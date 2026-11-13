//! Deterministic, untrusted ground instance search over a local VC handoff.

use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
};

use mizar_kernel::{
    certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
    checker::{
        FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy, KernelCheckStatus,
        KernelContextIdentityEntry, KernelContextIdentityPayload, KernelContextIdentitySource,
        KernelEvidenceCheckInput, KernelEvidenceCheckKind, KernelEvidenceCheckLimits,
        KernelFormulaProducerRef, KernelVcGeneratedFormulaId, check_kernel_evidence,
    },
    clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
    formula_evidence::{
        Formula, FormulaEvidenceEntry, FormulaEvidenceParseContext, FormulaSource,
        ParsedKernelEvidence, parse_formula_evidence,
    },
    rejection::TargetVcFingerprint,
};
use mizar_vc::{
    kernel_evidence_handoff::{
        KernelClauseTautologyPolicy, KernelContextIdentitySource as VcSource,
        VcKernelEvidenceHandoff,
    },
    vc_ir::VcFormulaRef,
};

pub use crate::backend::BackendCandidateEvidence as CandidateKernelEvidence;
use crate::{
    backend::{BackendCandidatePayload, EncodedBackendProblem},
    problem::{
        AtpDeclarationKind, AtpFormula, AtpFormulaTree, AtpProblem, AtpSourceRef, AtpTerm,
        ConcreteFormat,
    },
};

type WireSections = Vec<Vec<Vec<u8>>>;

/// Search limits count observed ground terms, premise substitutions (including
/// zero-variable premises), equality names, and kernel/SAT queries, respectively.
/// Missing limits disable search; zero limits are literal. Reports may only
/// select real premise labels. Unsupported or mismatched handoffs fail closed.
pub fn find_instance_evidence(
    problem: &AtpProblem,
    encoded: &EncodedBackendProblem,
    handoff: &VcKernelEvidenceHandoff,
    limits: Option<[u64; 4]>,
    used_premises: Option<&[String]>,
) -> Result<Option<CandidateKernelEvidence>, String> {
    if EncodedBackendProblem::encode(problem, encoded.concrete_format())? != *encoded {
        return Err(invalid());
    }
    let target = handoff.canonical_evidence().target_vc();
    if problem.target_binding().fingerprint().algorithm_id() != target.algorithm_id
        || problem.target_binding().fingerprint().digest() != target.digest
        || problem.target_binding().producer_binding().as_str()
            != format!(
                "mizar-vc-kernel-evidence-handoff:{}",
                handoff
                    .canonical_hash()
                    .as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            )
        || problem.type_context().guards().iter().any(|guard| {
            !problem
                .axioms()
                .iter()
                .any(|axiom| axiom.formula() == Some(guard.formula()))
        })
        || !problem.properties().is_empty()
    {
        return Err(invalid());
    }
    let base = handoff.encode_local_evidence(None)?;
    let profile = handoff.canonical_evidence().kernel_profile();
    let tautology = match profile.clause_tautology_policy {
        KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
        KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
        _ => return Err(invalid()),
    };
    let mut parse_context = FormulaEvidenceParseContext::v1(
        Fingerprint::new(target.algorithm_id, target.digest.clone()),
        KernelProfileRecord::v1(profile.profile_id, tautology),
    );
    parse_context.accepted_schema_versions.insert(2);
    let parsed = parse_formula_evidence(&base, &parse_context).map_err(|_| invalid())?;
    if !parsed.substitutions().is_empty() || parsed.formulas().len() != problem.axioms().len() {
        return Err(invalid());
    }
    let mut symbols = BTreeMap::new();
    let mut equalities = BTreeSet::new();
    let mut matched = BTreeSet::new();
    let mut selected = BTreeSet::new();
    let labels = match encoded.concrete_format() {
        ConcreteFormat::Tptp => {
            crate::tptp_encoder::encode_tptp(problem, crate::tptp_encoder::TptpDialect::Fof)
                .map_err(|error| error.to_string())?
                .formula_labels()
                .iter()
                .filter_map(|row| {
                    if let crate::tptp_encoder::TptpFormulaItem::Axiom(id) = row.item() {
                        Some((id, row.label().to_owned()))
                    } else {
                        None
                    }
                })
                .collect::<BTreeMap<_, _>>()
        }
        ConcreteFormat::SmtLib => crate::smtlib_encoder::encode_smtlib(
            problem,
            crate::smtlib_encoder::SmtLibDialect::Uninterpreted,
        )
        .map_err(|error| error.to_string())?
        .assertion_labels()
        .iter()
        .filter_map(|row| {
            if let crate::smtlib_encoder::SmtLibAssertionItem::Axiom(id) = row.item() {
                Some((id, row.label().to_owned()))
            } else {
                None
            }
        })
        .collect::<BTreeMap<_, _>>(),
    };
    if used_premises.is_some_and(|report| {
        report
            .iter()
            .any(|label| !labels.values().any(|real| real == label))
    }) {
        return Err(invalid());
    }
    let mut premises = Vec::new();
    for row in problem.axioms() {
        let provenance = provenance(problem, row)?;
        let entry = parsed
            .formulas()
            .iter()
            .find(|entry| source_matches(provenance.source(), &entry.source))
            .ok_or_else(invalid)?;
        if !matched.insert(entry.formula_id) {
            return Err(invalid());
        }
        authenticate_payload(provenance.payload().as_str(), entry.provenance_id, &parsed)?;
        let (tree, binders) = strip_universals(row.formula().ok_or_else(invalid)?)?;
        let mut variables = BTreeMap::new();
        match_formula(
            tree,
            &entry.formula,
            &mut symbols,
            &mut variables,
            &mut equalities,
        )?;
        if variables.keys().cloned().collect::<BTreeSet<_>>() != binders {
            return Err(invalid());
        }
        let label = labels.get(&row.id()).ok_or_else(invalid)?;
        if used_premises.is_none_or(|report| report.contains(label)) {
            selected.insert(entry.formula_id);
            premises.push((entry, variables.values().copied().collect::<BTreeSet<_>>()));
        }
    }
    let goal = problem.conjecture();
    let goal_provenance = provenance(problem, goal)?;
    if !matches!(goal_provenance.source(), AtpSourceRef::GeneratedVcFact(binding) if binding.as_str() == "goal:1")
    {
        return Err(invalid());
    }
    authenticate_payload(
        goal_provenance.payload().as_str(),
        parsed.final_goal().provenance_id,
        &parsed,
    )?;
    let mut variables = BTreeMap::new();
    match_formula(
        goal.formula().ok_or_else(invalid)?,
        &parsed.final_goal().formula,
        &mut symbols,
        &mut variables,
        &mut equalities,
    )?;
    if !variables.is_empty() || equalities.len() > 1 {
        return Err(invalid());
    }
    for (name, symbol) in &symbols {
        let declaration = problem
            .declarations()
            .iter()
            .find(|row| row.symbol().as_str() == name)
            .ok_or_else(invalid)?;
        if !matches!(
            (declaration.kind(), symbol.kind),
            (AtpDeclarationKind::Function, SymbolKind::FunctorPredicate)
                | (
                    AtpDeclarationKind::Predicate,
                    SymbolKind::Predicate
                        | SymbolKind::FunctorPredicate
                        | SymbolKind::BuiltinRelation
                )
        ) {
            return Err(invalid());
        }
    }
    let Some(limits) = limits else {
        return Ok(None);
    };
    let observed = problem.observed_ground_terms().map_err(|_| invalid())?;
    if observed.len() as u64 > limits[0] {
        return Ok(None);
    }
    let pool = observed
        .into_iter()
        .map(|term| translate_term(term, &symbols))
        .collect::<Result<Vec<_>, _>>()?;
    let (header, mut sections) = split_wire(&base)?;
    sections[2].retain(|item| selected.contains(&read_u32(item, 0)));
    sections[3].clear();
    sections.push(Vec::new());
    let context = kernel_context(handoff)?;
    let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
    let queries = Cell::new(0u64);
    let mut instances = premises
        .iter()
        .filter(|(_, variables)| variables.is_empty())
        .count() as u64;
    if instances > limits[1] {
        return Ok(None);
    }
    let mut counted_terms = pool
        .iter()
        .map(|term| (term.render(), term.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut ground_atoms = Vec::new();
    collect_atoms(&parsed.final_goal().formula, &mut ground_atoms);
    let query = |sections: &WireSections| -> Result<Option<Vec<u8>>, String> {
        if queries.get() == limits[3] {
            return Ok(None);
        }
        queries.set(queries.get() + 1);
        let wire = join_wire(&header, sections)?;
        let evidence = parse_formula_evidence(&wire, &parse_context).map_err(|_| invalid())?;
        let result = check_kernel_evidence(KernelEvidenceCheckInput {
            target_vc_fingerprint: &target,
            evidence: &evidence,
            formula_context: Some(&context),
            check_kind: KernelEvidenceCheckKind::ProofObligation,
            policy: KernelCheckPolicy::default(),
            limits: KernelEvidenceCheckLimits::default(),
        });
        Ok((result.status() == KernelCheckStatus::Accepted).then_some(wire))
    };
    let candidate = |wire| {
        CandidateKernelEvidence::new(
            "ground-instance-finder-v1",
            BackendCandidatePayload::FormulaSubstitutionBytes(wire),
            encoded.target_binding().clone(),
            encoded.input_hash(),
            encoded.provenance_hash(),
            encoded.formula_labels().to_vec(),
            encoded.symbol_bindings().to_vec(),
        )
        .map_err(|error| error.to_string())
    };
    if let Some(wire) = query(&sections)? {
        return candidate(wire).map(Some);
    }
    if queries.get() == limits[3] {
        return Ok(None);
    }
    // Each complete substitution is charged before it is materialized. Ground
    // premises are already in the wire and count as their empty substitution.
    for (entry, variables) in premises {
        if variables.is_empty() {
            collect_atoms(&entry.formula, &mut ground_atoms);
            continue;
        }
        if pool.is_empty() {
            continue;
        }
        let variables = variables.into_iter().collect::<Vec<_>>();
        let mut indices = vec![0; variables.len()];
        loop {
            if instances == limits[1] {
                return Ok(None);
            }
            instances += 1;
            let replacements = variables
                .iter()
                .copied()
                .zip(indices.iter().map(|i| pool[*i].clone()))
                .collect::<BTreeMap<_, _>>();
            let instantiated = instantiate(&entry.formula, &replacements)?;
            let mut atoms = Vec::new();
            collect_atoms(&instantiated, &mut atoms);
            for atom in &atoms {
                for term in &atom.arguments {
                    count_terms(term, &mut counted_terms);
                }
            }
            if counted_terms.len() as u64 > limits[0] {
                return Ok(None);
            }
            let id = sections[3].len() + 1;
            sections[3].push(substitution(entry, &replacements, id)?);
            ground_atoms.extend(atoms);
            if let Some(wire) = query(&sections)? {
                return candidate(wire).map(Some);
            }
            if queries.get() == limits[3] {
                return Ok(None);
            }
            let mut position = indices.len();
            while position > 0 {
                position -= 1;
                indices[position] += 1;
                if indices[position] < pool.len() {
                    break;
                }
                indices[position] = 0;
            }
            if position == 0 && indices[0] == 0 {
                break;
            }
        }
    }
    // Equality also uses the finitely generated instance subterms. They never
    // extend the substitution domain, so no unbounded term closure is formed.
    let pool = counted_terms.into_values().collect::<Vec<_>>();
    for equality in equalities {
        for schema in 1..=3u8 {
            if pool.is_empty() {
                continue;
            }
            let mut indices = vec![0; schema as usize];
            loop {
                if sections[6].len() as u64 == limits[2] {
                    return Ok(None);
                }
                let mut name = name_header(sections[6].len() + 1, schema, equality)?;
                for index in &indices {
                    put_term(&pool[*index], &mut name)?
                }
                sections[6].push(name);
                if let Some(wire) = query(&sections)? {
                    return candidate(wire).map(Some);
                }
                if queries.get() == limits[3] {
                    return Ok(None);
                }
                let mut position = indices.len();
                while position > 0 {
                    position -= 1;
                    indices[position] += 1;
                    if indices[position] < pool.len() {
                        break;
                    }
                    indices[position] = 0;
                }
                if position == 0 && indices[0] == 0 {
                    break;
                }
            }
        }
        let mut applications = pool
            .iter()
            .filter_map(|term| match term {
                Term::Application { symbol, arguments } => {
                    Some(Atom::new(*symbol, arguments.clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        applications.sort_by_key(Atom::render);
        ground_atoms.sort_by_key(Atom::render);
        ground_atoms.dedup();
        for (schema, atoms) in [(4, &applications), (5, &ground_atoms)] {
            for left in atoms {
                for right in atoms {
                    if left.symbol != right.symbol
                        || left.arguments.len() != right.arguments.len()
                        || (schema == 5 && left.symbol.kind == SymbolKind::Equality)
                    {
                        continue;
                    }
                    if sections[6].len() as u64 == limits[2] {
                        return Ok(None);
                    }
                    let mut name = name_header(sections[6].len() + 1, schema, equality)?;
                    name.push(kind_tag(left.symbol.kind)?);
                    put_u32(left.symbol.id.0 as usize, &mut name)?;
                    put_u32(left.arguments.len(), &mut name)?;
                    for side in [left, right] {
                        put_u32(side.arguments.len(), &mut name)?;
                        for term in &side.arguments {
                            put_term(term, &mut name)?
                        }
                    }
                    sections[6].push(name);
                    if let Some(wire) = query(&sections)? {
                        return candidate(wire).map(Some);
                    }
                    if queries.get() == limits[3] {
                        return Ok(None);
                    }
                }
            }
        }
    }
    Ok(None)
}

fn invalid() -> String {
    "instance_finder.unsupported_or_mismatched_input".to_owned()
}

fn provenance<'a>(
    problem: &'a AtpProblem,
    row: &AtpFormula,
) -> Result<&'a crate::problem::AtpProvenance, String> {
    problem
        .provenance()
        .iter()
        .find(|item| item.id() == row.provenance())
        .ok_or_else(invalid)
}

fn authenticate_payload(
    payload: &str,
    id: u32,
    parsed: &ParsedKernelEvidence,
) -> Result<(), String> {
    let source = parsed
        .provenance()
        .iter()
        .find(|item| item.provenance_id == id)
        .ok_or_else(invalid)?;
    let hex = source
        .payload
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if payload != format!("mizar-vc-handoff-provenance:{hex}") {
        return Err(invalid());
    }
    Ok(())
}

fn source_matches(source: &AtpSourceRef, kernel: &FormulaSource) -> bool {
    match (source, kernel) {
        (
            AtpSourceRef::LocalHypothesis(binding),
            FormulaSource::LocalHypothesis { local_context_id },
        ) => binding.as_str() == format!("local-context:{local_context_id}"),
        (AtpSourceRef::CitedPremise(binding), FormulaSource::CitedPremise { local_context_id }) => {
            binding.as_str() == format!("cited-premise:{local_context_id}")
        }
        (AtpSourceRef::GeneratedVcFact(binding), FormulaSource::GeneratedVcFact { vc_fact_id }) => {
            binding.as_str() == format!("generated:{vc_fact_id}")
        }
        _ => false,
    }
}

fn strip_universals(
    mut tree: &AtpFormulaTree,
) -> Result<(&AtpFormulaTree, BTreeSet<String>), String> {
    let mut variables = BTreeSet::new();
    while let AtpFormulaTree::Forall { binders, body } = tree {
        for binder in binders {
            if binder.sort().is_some() || !variables.insert(binder.variable().as_str().to_owned()) {
                return Err(invalid());
            }
        }
        tree = body;
    }
    Ok((tree, variables))
}

fn bind<K: Ord + Clone, V: Eq + Clone>(
    map: &mut BTreeMap<K, V>,
    key: K,
    value: V,
) -> Result<(), String> {
    if let Some(old) = map.get(&key) {
        if old != &value {
            return Err(invalid());
        }
    } else {
        if map.values().any(|old| old == &value) {
            return Err(invalid());
        }
        map.insert(key, value);
    }
    Ok(())
}

fn match_formula(
    tree: &AtpFormulaTree,
    formula: &Formula,
    symbols: &mut BTreeMap<String, SymbolKey>,
    variables: &mut BTreeMap<String, VariableId>,
    equalities: &mut BTreeSet<SymbolKey>,
) -> Result<(), String> {
    match (tree, formula) {
        (AtpFormulaTree::Atom(a), Formula::Atom(b))
            if a.arguments().len() == b.arguments.len()
                && b.symbol.kind != SymbolKind::Equality =>
        {
            bind(symbols, a.predicate().as_str().to_owned(), b.symbol)?;
            for (a, b) in a.arguments().iter().zip(&b.arguments) {
                match_term(a, b, symbols, variables)?
            }
        }
        (AtpFormulaTree::Equality { left, right }, Formula::Atom(atom))
            if atom.symbol.kind == SymbolKind::Equality && atom.arguments.len() == 2 =>
        {
            equalities.insert(atom.symbol);
            match_term(left, &atom.arguments[0], symbols, variables)?;
            match_term(right, &atom.arguments[1], symbols, variables)?;
        }
        (AtpFormulaTree::Not(a), Formula::Not(b)) => {
            match_formula(a, b, symbols, variables, equalities)?
        }
        (AtpFormulaTree::And(a), Formula::And(b)) | (AtpFormulaTree::Or(a), Formula::Or(b))
            if a.len() == b.len() =>
        {
            for (a, b) in a.iter().zip(b) {
                match_formula(a, b, symbols, variables, equalities)?
            }
        }
        (AtpFormulaTree::Implies(a, b), Formula::Or(children)) if children.len() == 2 => {
            let Formula::Not(left) = &children[0] else {
                return Err(invalid());
            };
            match_formula(a, left, symbols, variables, equalities)?;
            match_formula(b, &children[1], symbols, variables, equalities)?;
        }
        (AtpFormulaTree::True, Formula::And(children))
        | (AtpFormulaTree::False, Formula::Or(children))
            if children.is_empty() => {}
        _ => return Err(invalid()),
    }
    Ok(())
}

fn match_term(
    tree: &AtpTerm,
    term: &Term,
    symbols: &mut BTreeMap<String, SymbolKey>,
    variables: &mut BTreeMap<String, VariableId>,
) -> Result<(), String> {
    match (tree, term) {
        (AtpTerm::Variable(a), Term::Variable(b)) => bind(variables, a.as_str().to_owned(), *b),
        (
            AtpTerm::Function {
                function,
                arguments: a,
            },
            Term::Application {
                symbol,
                arguments: b,
            },
        ) if a.len() == b.len() && symbol.kind == SymbolKind::FunctorPredicate => {
            bind(symbols, function.as_str().to_owned(), *symbol)?;
            for (a, b) in a.iter().zip(b) {
                match_term(a, b, symbols, variables)?
            }
            Ok(())
        }
        _ => Err(invalid()),
    }
}

fn translate_term(term: &AtpTerm, symbols: &BTreeMap<String, SymbolKey>) -> Result<Term, String> {
    let AtpTerm::Function {
        function,
        arguments,
    } = term
    else {
        return Err(invalid());
    };
    Ok(Term::Application {
        symbol: *symbols.get(function.as_str()).ok_or_else(invalid)?,
        arguments: arguments
            .iter()
            .map(|term| translate_term(term, symbols))
            .collect::<Result<_, _>>()?,
    })
}

fn instantiate(
    formula: &Formula,
    replacements: &BTreeMap<VariableId, Term>,
) -> Result<Formula, String> {
    fn term(value: &Term, replacements: &BTreeMap<VariableId, Term>) -> Result<Term, String> {
        match value {
            Term::Variable(id) => replacements.get(id).cloned().ok_or_else(invalid),
            Term::Application { symbol, arguments } => Ok(Term::Application {
                symbol: *symbol,
                arguments: arguments
                    .iter()
                    .map(|value| term(value, replacements))
                    .collect::<Result<_, _>>()?,
            }),
            _ => Err(invalid()),
        }
    }
    Ok(match formula {
        Formula::Atom(atom) => Formula::Atom(Atom::new(
            atom.symbol,
            atom.arguments
                .iter()
                .map(|value| term(value, replacements))
                .collect::<Result<_, _>>()?,
        )),
        Formula::Not(child) => Formula::Not(Box::new(instantiate(child, replacements)?)),
        Formula::And(children) => Formula::And(
            children
                .iter()
                .map(|child| instantiate(child, replacements))
                .collect::<Result<_, _>>()?,
        ),
        Formula::Or(children) => Formula::Or(
            children
                .iter()
                .map(|child| instantiate(child, replacements))
                .collect::<Result<_, _>>()?,
        ),
        _ => return Err(invalid()),
    })
}

fn collect_atoms(formula: &Formula, atoms: &mut Vec<Atom>) {
    match formula {
        Formula::Atom(atom) => atoms.push(atom.clone()),
        Formula::Not(child) => collect_atoms(child, atoms),
        Formula::And(children) | Formula::Or(children) => {
            for child in children {
                collect_atoms(child, atoms)
            }
        }
        _ => {}
    }
}

fn count_terms(term: &Term, counted: &mut BTreeMap<String, Term>) {
    counted.entry(term.render()).or_insert_with(|| term.clone());
    if let Term::Application { arguments, .. } = term {
        for term in arguments {
            count_terms(term, counted);
        }
    }
}

fn substitution(
    entry: &FormulaEvidenceEntry,
    replacements: &BTreeMap<VariableId, Term>,
    id: usize,
) -> Result<Vec<u8>, String> {
    let mut binder = 1u16.to_be_bytes().to_vec();
    put_u32(0, &mut binder)?;
    put_u32(0, &mut binder)?;
    put_u32(replacements.len(), &mut binder)?;
    for variable in replacements.keys() {
        put_u32(variable.0 as usize, &mut binder)?
    }
    let mut bytes = Vec::new();
    for value in [id, entry.formula_id as usize, entry.provenance_id as usize] {
        put_u32(value, &mut bytes)?
    }
    put_bytes(&binder, &mut bytes)?;
    put_u32(id, &mut bytes)?;
    bytes.push(1);
    put_u32(0, &mut bytes)?;
    put_u32(replacements.len(), &mut bytes)?;
    for (variable, term) in replacements {
        put_u32(variable.0 as usize, &mut bytes)?;
        put_term(term, &mut bytes)?;
        bytes.push(1);
    }
    put_u32(0, &mut bytes)?;
    put_u32(0, &mut bytes)?;
    Ok(bytes)
}

fn name_header(id: usize, schema: u8, equality: SymbolKey) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    put_u32(id, &mut bytes)?;
    bytes.push(schema);
    put_u32(equality.id.0 as usize, &mut bytes)?;
    Ok(bytes)
}

fn kind_tag(kind: SymbolKind) -> Result<u8, String> {
    match kind {
        SymbolKind::Predicate => Ok(1),
        SymbolKind::FunctorPredicate => Ok(2),
        SymbolKind::Equality => Ok(3),
        SymbolKind::BuiltinRelation => Ok(4),
        _ => Err(invalid()),
    }
}

fn put_term(term: &Term, bytes: &mut Vec<u8>) -> Result<(), String> {
    match term {
        Term::Application { symbol, arguments } => {
            bytes.push(2);
            bytes.push(kind_tag(symbol.kind)?);
            put_u32(symbol.id.0 as usize, bytes)?;
            put_u32(arguments.len(), bytes)?;
            for term in arguments {
                put_term(term, bytes)?
            }
            Ok(())
        }
        _ => Err(invalid()),
    }
}

fn put_u32(value: usize, bytes: &mut Vec<u8>) -> Result<(), String> {
    bytes.extend_from_slice(&u32::try_from(value).map_err(|_| invalid())?.to_be_bytes());
    Ok(())
}

fn put_bytes(value: &[u8], bytes: &mut Vec<u8>) -> Result<(), String> {
    put_u32(value.len(), bytes)?;
    bytes.extend_from_slice(value);
    Ok(())
}
fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("validated local wire"),
    )
}

fn split_wire(bytes: &[u8]) -> Result<(Vec<u8>, WireSections), String> {
    // Only called on the successfully parsed local encoder output above.
    let count_offset = b"MIZAR_KERNEL_EVIDENCE\0".len()
        + 12
        + 5
        + read_u32(bytes, b"MIZAR_KERNEL_EVIDENCE\0".len() + 13) as usize;
    let count = read_u32(bytes, count_offset) as usize;
    if count != 6 {
        return Err(invalid());
    }
    let payload_offset = count_offset + 4 + count * 13;
    let mut sections = Vec::new();
    for index in 0..count {
        let directory = count_offset + 4 + index * 13;
        let mut offset = payload_offset + read_u32(bytes, directory + 5) as usize;
        let mut items = Vec::new();
        for _ in 0..read_u32(bytes, directory + 1) {
            let len = read_u32(bytes, offset + 2) as usize;
            items.push(bytes[offset + 6..offset + 6 + len].to_vec());
            offset += 6 + len;
        }
        sections.push(items);
    }
    Ok((bytes[..count_offset].to_vec(), sections))
}

fn join_wire(header: &[u8], sections: &WireSections) -> Result<Vec<u8>, String> {
    let mut bytes = header.to_vec();
    let schema_offset = b"MIZAR_KERNEL_EVIDENCE\0".len();
    bytes[schema_offset..schema_offset + 2].copy_from_slice(&2u16.to_be_bytes());
    put_u32(sections.len(), &mut bytes)?;
    let mut payload = Vec::new();
    for (index, section) in sections.iter().enumerate() {
        let tag = u8::try_from(index + 1).map_err(|_| invalid())?;
        let start = payload.len();
        for item in section {
            payload.extend_from_slice(&[tag, 1]);
            put_bytes(item, &mut payload)?
        }
        bytes.push(tag);
        put_u32(section.len(), &mut bytes)?;
        put_u32(start, &mut bytes)?;
        put_u32(payload.len() - start, &mut bytes)?;
    }
    bytes.extend(payload);
    Ok(bytes)
}

fn kernel_context(handoff: &VcKernelEvidenceHandoff) -> Result<FormulaEvidenceContext, String> {
    let source_target = handoff.canonical_evidence().target_vc();
    let target = TargetVcFingerprint::new(source_target.algorithm_id, source_target.digest.clone());
    let entries = handoff
        .context_identity()
        .entries()
        .iter()
        .map(|entry| {
            let source = match entry.source() {
                VcSource::LocalHypothesis { local_context_id } => {
                    KernelContextIdentitySource::LocalHypothesis { local_context_id }
                }
                VcSource::CitedPremise { local_context_id } => {
                    KernelContextIdentitySource::CitedPremise { local_context_id }
                }
                VcSource::GeneratedVcFact { vc_fact_id } => {
                    KernelContextIdentitySource::GeneratedVcFact { vc_fact_id }
                }
                _ => return Err(invalid()),
            };
            let producer = match entry.producer_formula_ref() {
                VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                VcFormulaRef::Generated(id) => {
                    KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(id.index()))
                }
                _ => return Err(invalid()),
            };
            Ok(KernelContextIdentityEntry::new(
                source,
                entry.formula_id(),
                Fingerprint::new(
                    entry.formula_fingerprint().algorithm_id,
                    entry.formula_fingerprint().digest.clone(),
                ),
                producer,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    FormulaEvidenceContext::with_context_identity(
        Some(handoff.context_identity_hash().as_bytes().to_vec()),
        Vec::new(),
        Vec::new(),
        Some(KernelContextIdentityPayload::new(
            target,
            handoff.canonical_hash(),
            handoff.context_identity_hash(),
            entries,
        )),
        ImportedFactContextLimits::default(),
    )
    .map_err(|_| invalid())
}

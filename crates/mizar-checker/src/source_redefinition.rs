//! Source-authenticated declaration roots and pending redefinition targets.

use crate::{binding_env::*, overload_resolution::*, type_checker::*, typed_ast::*};
use SurfaceNodeKind as K;
use SurfaceTokenKind as T;
use mizar_resolve::{
    declarations::DeclarationShellKind,
    env::{DefinitionKind, RelationKind, SymbolEnv, SymbolKind},
    names::{LocalTermScope, resolve_template_formal},
    resolved_ast::{SurfaceResolvedArena, SymbolId},
    symbols::{source_declaration_shells, validate_source_symbol_env},
};
use mizar_session::SourceAnchor;
use mizar_syntax::ast::{SurfaceFormulaConstant, SurfaceNodeKind, SurfaceTokenKind};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

// Borrowed original roles, never a stored syntax graph or caller input schema.
type Roles<'a> = (
    &'a SourceNode,
    &'a SourceNode,
    Vec<&'a SourceNode>,
    Vec<&'a SourceNode>,
    Option<&'a SourceNode>,
    &'a SourceNode,
    CandidateDeclarationKind,
    bool,
    String,
);
type Root = (
    SymbolId,
    TypedSiteRef,
    CandidateDeclarationKind,
    Vec<NormalizedTypeId>,
    Option<NormalizedTypeId>,
);

fn invalid() -> String {
    "source.redefinition.unsupported_source".to_owned()
}
fn children<'a>(rows: &'a [SourceNode], row: &SourceNode) -> Result<Vec<&'a SourceNode>, String> {
    let mut result = source_children(rows, row.site()).collect::<Vec<_>>();
    result.sort_by_key(|child| child.child_ordinal());
    if source_contiguous_child_count(result.iter().copied()) != Some(result.len()) {
        return Err(invalid());
    }
    Ok(result)
}
fn token(row: &SourceNode, text: &str) -> bool {
    matches!(row.kind(), K::Token(t) if t.text.as_ref()==text && t.kind==
        if text.chars().all(|c| c.is_ascii_alphabetic()) { T::ReservedWord } else { T::ReservedSymbol })
}
fn name(row: &SourceNode) -> bool {
    matches!(row.kind(), K::Token(t) if matches!(t.kind,T::Identifier|T::UserSymbol) && !t.text.is_empty())
}
fn only<'a>(rows: &'a [SourceNode], row: &SourceNode, kind: &K) -> Result<&'a SourceNode, String> {
    if row.kind() != kind {
        return Err(invalid());
    }
    let parts = children(rows, row)?;
    let [child] = parts.as_slice() else {
        return Err(invalid());
    };
    Ok(*child)
}
fn site(original: &TypedAst, row: &SourceNode) -> Result<TypedSiteRef, String> {
    let matches = original
        .nodes()
        .iter()
        .filter(|(_, n)| n.resolved_node == Some(row.site()))
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    let [id] = matches.as_slice() else {
        return Err(invalid());
    };
    Ok(TypedSiteRef::Node(*id))
}
fn syntax(native: &SurfaceResolvedArena) -> Result<TypedAst, String> {
    let nodes = TypedArena::try_new(
        Some(TypedNodeId::new(native.arena().root().index())),
        native
            .arena()
            .iter()
            .map(|(id, n)| {
                TypedNode::new("syntax", n.origin().anchor().clone())
                    .with_resolved_node(id)
                    .with_children(
                        n.children()
                            .iter()
                            .map(|c| TypedNodeId::new(c.index()))
                            .collect(),
                    )
            })
            .collect(),
    )
    .map_err(|_| invalid())?;
    let mut ast = TypedAst::try_new(TypedAstParts {
        source_id: native.source_id(),
        module_id: native.module().clone(),
        resolved_root: Some(native.arena().root()),
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes,
        contexts: Default::default(),
        types: Default::default(),
        facts: Default::default(),
        coercions: Default::default(),
        initial_obligations: Default::default(),
        diagnostics: Default::default(),
    })
    .map_err(|_| invalid())?;
    ast.install_source_nodes(native).map_err(|_| invalid())?;
    Ok(ast)
}
fn builtin(
    rows: &[SourceNode],
    original: &TypedAst,
    row: &SourceNode,
) -> Result<TypeExpressionInput, String> {
    let head = only(rows, row, &K::TypeExpression)?;
    let word = only(rows, head, &K::TypeHead)?;
    let radix = if token(word, "set") {
        TypeHeadInput::BuiltinSet
    } else if token(word, "object") {
        TypeHeadInput::BuiltinObject
    } else {
        return Err(invalid());
    };
    Ok(TypeExpressionInput::new(
        site(original, row)?,
        row.range(),
        row.spelling(),
        radix,
    ))
}
fn coherence(rows: &[SourceNode], row: &SourceNode) -> Result<(), String> {
    if row.kind() != &K::CoherenceCondition {
        return Err(invalid());
    }
    let parts = children(rows, row)?;
    let (word, proof, semi) = match parts.as_slice() {
        [word, proof, semi] => (*word, *proof, *semi),
        [word, with, label, proof, semi]
            if token(with, "with")
                && matches!(label.kind(), K::Token(t) if t.kind == T::Identifier && !t.text.is_empty()) =>
        {
            (*word, *proof, *semi)
        }
        _ => return Err(invalid()),
    };
    if !token(word, "coherence") || !token(semi, ";") || proof.kind() != &K::ProofBlock {
        return Err(invalid());
    }
    thesis_proof(rows, proof)
}
fn thesis_proof(rows: &[SourceNode], proof: &SourceNode) -> Result<(), String> {
    if proof.kind() != &K::ProofBlock {
        return Err(invalid());
    }
    let parts = children(rows, proof)?;
    let [word, claim, end] = parts.as_slice() else {
        return Err(invalid());
    };
    if !token(word, "proof") || !token(end, "end") || claim.kind() != &K::ConclusionStatement {
        return Err(invalid());
    }
    let parts = children(rows, claim)?;
    let [word, proposition, semi] = parts.as_slice() else {
        return Err(invalid());
    };
    if !token(word, "thus") || !token(semi, ";") {
        return Err(invalid());
    }
    let formula = only(rows, proposition, &K::Proposition)?;
    let thesis = only(rows, formula, &K::FormulaExpression)?;
    if !matches!(
        thesis.kind(),
        K::FormulaConstant(SurfaceFormulaConstant::Thesis)
    ) || !token(only(rows, thesis, thesis.kind())?, "thesis")
    {
        return Err(invalid());
    }
    Ok(())
}
fn roles<'a>(
    roots: &'a SourceOrdinaryRootHandoff,
    rows: &'a [SourceNode],
    block: &'a SourceNode,
) -> Result<Roles<'a>, String> {
    let frame = children(rows, block)?;
    let [word, content @ .., end, semi] = frame.as_slice() else {
        return Err(invalid());
    };
    let (parameters, decl, alias) = match content {
        [parameters @ .., decl, alias] if alias.kind() == &K::NotationAlias => {
            (parameters, *decl, true)
        }
        [parameters @ .., decl] => (parameters, *decl, false),
        _ => return Err(invalid()),
    };
    if block.kind() != &K::DefinitionBlockItem
        || parameters.is_empty()
        || !token(word, "definition")
        || !token(end, "end")
        || !token(semi, ";")
    {
        return Err(invalid());
    }
    let mut formals = Vec::new();
    let mut guards = Vec::new();
    for parameter in parameters {
        let parts = children(rows, parameter)?;
        let [word, segment, semi] = parts.as_slice() else {
            return Err(invalid());
        };
        if parameter.kind() != &K::DefinitionParameter
            || segment.kind() != &K::QualifiedVariableSegment
            || !token(word, "let")
            || !token(semi, ";")
        {
            return Err(invalid());
        }
        let parts = children(rows, segment)?;
        let (names, be, ty) = match parts.as_slice() {
            [x, be, ty] => (vec![*x], *be, *ty),
            [x, comma, y, be, ty] if token(comma, ",") && x.spelling() != y.spelling() => {
                (vec![*x, *y], *be, *ty)
            }
            _ => return Err(invalid()),
        };
        if !token(be, "be")
            || names.iter().any(|f| {
                !matches!(f.kind(),K::Token(t) if t.kind==T::Identifier)
                    || formals
                        .iter()
                        .any(|old: &&SourceNode| old.spelling() == f.spelling())
            })
        {
            return Err(invalid());
        }
        guards.extend(vec![ty; names.len()]);
        formals.extend(names);
    }
    let (kind, redefined, keyword) = match decl.kind() {
        K::AttributeDefinition => (CandidateDeclarationKind::Attribute, false, "attr"),
        K::AttributeRedefinition => (CandidateDeclarationKind::Attribute, true, "attr"),
        K::PredicateDefinition => (CandidateDeclarationKind::Predicate, false, "pred"),
        K::PredicateRedefinition => (CandidateDeclarationKind::Predicate, true, "pred"),
        K::FunctorDefinition => (CandidateDeclarationKind::Functor, false, "func"),
        K::FunctorRedefinition => (CandidateDeclarationKind::Functor, true, "func"),
        _ => return Err(invalid()),
    };
    if alias && redefined {
        return Err(invalid());
    }
    let parts = children(rows, decl)?;
    let skip = usize::from(redefined);
    if redefined && !token(parts.first().ok_or_else(invalid)?, "redefine") {
        return Err(invalid());
    }
    let parts = parts.get(skip..).ok_or_else(invalid)?;
    let count = if kind == CandidateDeclarationKind::Predicate {
        7
    } else {
        9
    };
    if parts.len() != count + skip
        || !token(parts[0], keyword)
        || !matches!(parts[1].kind(),K::Token(t) if t.kind==T::Identifier)
        || !token(parts[2], ":")
        || !token(parts[count - 1], ";")
    {
        return Err(invalid());
    }
    if redefined {
        coherence(rows, parts[count])?;
    }
    let (body, ret, notation) = match kind {
        CandidateDeclarationKind::Attribute => {
            source_attribute_redefinition_roles(roots, parts, &formals)?
        }
        CandidateDeclarationKind::Predicate => {
            source_predicate_redefinition_roles(roots, parts, &formals)?
        }
        CandidateDeclarationKind::Functor => {
            source_functor_redefinition_roles(roots, parts, &formals)?
        }
        _ => return Err(invalid()),
    };
    Ok((
        block, decl, formals, guards, ret, body, kind, redefined, notation,
    ))
}
fn intake(roots: &SourceOrdinaryRootHandoff) -> Result<Vec<Roles<'_>>, String> {
    let rows = roots.original.source_nodes().ok_or_else(invalid)?;
    let units = rows
        .iter()
        .filter(|r| r.kind() == &K::CompilationUnit)
        .collect::<Vec<_>>();
    let [unit] = units.as_slice() else {
        return Err(invalid());
    };
    let list = only(rows, unit, &K::CompilationUnit)?;
    if list.kind() != &K::ItemList {
        return Err(invalid());
    }
    let blocks = children(rows, list)?;
    if blocks.len() < 2 {
        return Err(invalid());
    }
    let mut result = Vec::new();
    let mut labels = BTreeSet::new();
    let mut blocks = blocks.as_slice();
    if let [declarations @ .., reserve, theorem] = blocks
        && reserve.kind() == &K::ReserveItem
        && theorem.kind() == &K::TheoremItem
    {
        blocks = declarations;
    }
    for block in blocks {
        let declaration = roles(roots, rows, block)?;
        let label = children(rows, declaration.1)?[1 + usize::from(declaration.7)].spelling();
        if !labels.insert(label) {
            return Err(invalid());
        }
        result.push(declaration);
    }
    if result.iter().filter(|r| r.7).count() != 1 {
        return Err(invalid());
    }
    // Every structural node belongs to the exact admitted compilation tree.
    let mut covered = BTreeSet::new();
    let mut pending = vec![*unit];
    while let Some(row) = pending.pop() {
        if !covered.insert(row.site()) {
            return Err(invalid());
        }
        pending.extend(children(rows, row)?);
    }
    if rows
        .iter()
        .any(|r| r.kind() != &K::Root && !covered.contains(&r.site()))
    {
        return Err(invalid());
    }
    Ok(result)
}

// The only admitted application is the final reflexive equality over a reserved
// Object. Its proof is syntax-checked but supplies no semantic evidence.
fn application(original: &TypedAst) -> Result<Option<[&SourceNode; 4]>, String> {
    let rows = original.source_nodes().ok_or_else(invalid)?;
    let units = rows
        .iter()
        .find(|r| r.kind() == &K::ItemList)
        .ok_or_else(invalid)?;
    let items = children(rows, units)?;
    if items
        .last()
        .is_some_and(|row| row.kind() == &K::DefinitionBlockItem)
    {
        return Ok(None);
    }
    let [_, .., reserve, theorem] = items.as_slice() else {
        return Err(invalid());
    };
    if reserve.kind() != &K::ReserveItem || theorem.kind() != &K::TheoremItem {
        return Err(invalid());
    }
    let reserve_parts = children(rows, reserve)?;
    let [word, segment, semi] = reserve_parts.as_slice() else {
        return Err(invalid());
    };
    if !token(word, "reserve") || !token(semi, ";") || segment.kind() != &K::ReserveSegment {
        return Err(invalid());
    }
    let segment_parts = children(rows, segment)?;
    let [variable, for_word, ty] = segment_parts.as_slice() else {
        return Err(invalid());
    };
    if !matches!(variable.kind(), K::Token(t) if t.kind == T::Identifier)
        || !token(for_word, "for")
        || !token(
            only(rows, only(rows, ty, &K::TypeExpression)?, &K::TypeHead)?,
            "object",
        )
    {
        return Err(invalid());
    }
    let theorem_parts = children(rows, theorem)?;
    let [word, label, colon, formula, proof, semi] = theorem_parts.as_slice() else {
        return Err(invalid());
    };
    if !token(word, "theorem")
        || !matches!(label.kind(), K::Token(t) if t.kind == T::Identifier)
        || !token(colon, ":")
        || !token(semi, ";")
    {
        return Err(invalid());
    }
    thesis_proof(rows, proof)?;
    let equality = only(rows, formula, &K::FormulaExpression)?;
    if equality.kind() != &K::BuiltinPredicateApplication {
        return Err(invalid());
    }
    let equation = children(rows, equality)?;
    let [left, equals, right] = equation.as_slice() else {
        return Err(invalid());
    };
    if !token(equals, "=") {
        return Err(invalid());
    }
    let prefix = only(rows, left, &K::TermExpression)?;
    let K::PrefixExpression(operator) = prefix.kind() else {
        return Err(invalid());
    };
    let call = children(rows, prefix)?;
    let [head, parenthesized] = call.as_slice() else {
        return Err(invalid());
    };
    if !name(head)
        || operator.spelling.as_ref() != head.spelling()
        || parenthesized.kind() != &K::ParenthesizedTerm
    {
        return Err(invalid());
    }
    let argument = children(rows, parenthesized)?;
    let [open, argument, close] = argument.as_slice() else {
        return Err(invalid());
    };
    if !token(open, "(") || !token(close, ")") {
        return Err(invalid());
    }
    for term in [*argument, *right] {
        let reference = only(rows, term, &K::TermExpression)?;
        let word = only(rows, reference, &K::TermReference)?;
        if !matches!(word.kind(), K::Token(t) if t.kind == T::Identifier)
            || word.spelling() != variable.spelling()
        {
            return Err(invalid());
        }
    }
    Ok(Some([ty, left, argument, head]))
}

fn relations(
    roots: &SourceOrdinaryRootHandoff,
) -> Result<Vec<(TypedSiteRef, RelationKind, SymbolId)>, String> {
    let rows = roots.original.source_nodes().ok_or_else(invalid)?;
    let mut out = Vec::new();
    for d in intake(roots)? {
        let block = children(rows, d.0)?;
        let Some(alias) = block.iter().find(|row| row.kind() == &K::NotationAlias) else {
            continue;
        };
        let parts = children(rows, alias)?;
        let [word, alternate, for_word, original, semi] = parts.as_slice() else {
            return Err(invalid());
        };
        if !token(for_word, "for")
            || !token(semi, ";")
            || alternate.kind() != &K::NotationPattern
            || original.kind() != &K::NotationPattern
        {
            return Err(invalid());
        }
        let alternate = children(rows, alternate)?;
        let original = children(rows, original)?;
        let kind = match (&d.6, alternate.as_slice(), original.as_slice()) {
            (CandidateDeclarationKind::Attribute, [alias], [base])
                if token(word, "synonym")
                    && matches!(alias.kind(), K::Token(t) if t.kind == T::Identifier)
                    && name(base)
                    && alias.spelling() != base.spelling()
                    && base.spelling() == d.8 =>
            {
                RelationKind::Synonym
            }
            (CandidateDeclarationKind::Predicate, [a, alias, b], [x, base, y])
                if token(word, "antonym")
                    && name(alias)
                    && name(base)
                    && alias.spelling() != base.spelling()
                    && base.spelling() == d.8 =>
            {
                for (locus, formal) in [a, b, x, y]
                    .into_iter()
                    .zip([d.2[0], d.2[1], d.2[0], d.2[1]])
                {
                    if resolve_template_formal(&roots.native, locus.site())? != formal.site() {
                        return Err(invalid());
                    }
                }
                RelationKind::Antonym
            }
            _ => return Err(invalid()),
        };
        out.push((
            site(&roots.original, alias)?,
            kind,
            symbol(roots, d.1, d.6, false)?,
        ));
    }
    Ok(out)
}
fn symbol(
    roots: &SourceOrdinaryRootHandoff,
    row: &SourceNode,
    kind: CandidateDeclarationKind,
    redefined: bool,
) -> Result<SymbolId, String> {
    let (sk, dk, arity) = if redefined {
        (SymbolKind::Redefinition, DefinitionKind::Redefinition, None)
    } else {
        match kind {
            CandidateDeclarationKind::Attribute => {
                (SymbolKind::Attribute, DefinitionKind::Attribute, Some(0))
            }
            CandidateDeclarationKind::Predicate => {
                (SymbolKind::Predicate, DefinitionKind::Predicate, Some(2))
            }
            CandidateDeclarationKind::Functor => {
                (SymbolKind::Functor, DefinitionKind::Functor, Some(1))
            }
            _ => return Err(invalid()),
        }
    };
    let entries = roots
        .symbols
        .symbols()
        .iter()
        .filter(|e| e.kind() == sk && e.origin().anchor() == &SourceAnchor::Range(row.range()))
        .collect::<Vec<_>>();
    let [entry] = entries.as_slice() else {
        return Err(invalid());
    };
    let defs = roots
        .symbols
        .definitions()
        .iter()
        .filter(|d| {
            d.symbol() == entry.symbol()
                && d.kind() == dk
                && d.origin() == entry.origin()
                && d.arity() == arity
                && d.contribution() == entry.contribution()
                && d.conflict().is_none()
        })
        .collect::<Vec<_>>();
    if defs.len() != 1 || entry.symbol().module() != roots.original.module_id() {
        return Err(invalid());
    }
    Ok(entry.symbol().clone())
}
fn normalized(
    roots: &SourceOrdinaryRootHandoff,
    row: &SourceNode,
) -> Result<NormalizedTypeId, String> {
    let owner = site(&roots.original, row)?;
    let entries = roots
        .normalization
        .type_entries()
        .iter()
        .filter(|(_, e)| e.owner == owner && e.status == TypeStatus::Known)
        .collect::<Vec<_>>();
    let [(_, entry)] = entries.as_slice() else {
        return Err(invalid());
    };
    let TypeEntryActual::Known(id) = entry.actual else {
        return Err(invalid());
    };
    Ok(id)
}

/// Immutable source-owned ordinary declaration-root prerequisite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceOrdinaryRootHandoff {
    original: Arc<TypedAst>,
    native: Arc<SurfaceResolvedArena>,
    symbols: Arc<SymbolEnv>,
    normalization: TypeNormalizationOutput,
    roots: Vec<Root>,
    collection: OverloadCollectionOutput,
    expansion: TemplateExpansionOutput,
    viability: CandidateViabilityOutput,
    specificity: SpecificityGraphOutput,
}
impl SourceOrdinaryRootHandoff {
    pub fn build(
        original: Arc<TypedAst>,
        native: Arc<SurfaceResolvedArena>,
        symbols: Arc<SymbolEnv>,
    ) -> Result<Self, String> {
        validate_source_symbol_env(&native, &symbols)?;
        if *original != syntax(&native)? {
            return Err(invalid());
        }
        let normalization = TypeNormalizer::default().normalize(&symbols, []);
        let collection = OverloadCollectionOutput::collect([], []);
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability = CandidateViabilityOutput::filter(&expansion, []);
        let specificity = SpecificityGraphOutput::build(&viability, []);
        let mut out = Self {
            original,
            native,
            symbols,
            normalization,
            roots: Vec::new(),
            collection,
            expansion,
            viability,
            specificity,
        };
        let declarations = intake(&out)?;
        let rows = out.original.source_nodes().ok_or_else(invalid)?;
        let mut inputs = Vec::new();
        for d in &declarations {
            symbol(&out, d.1, d.6.clone(), d.7)?;
            for guard in &d.3 {
                let owner = site(&out.original, guard)?;
                if !inputs
                    .iter()
                    .any(|input: &TypeExpressionInput| input.site == owner)
                {
                    inputs.push(builtin(rows, &out.original, guard)?);
                }
            }
            if let Some(ret) = d.4 {
                inputs.push(builtin(rows, &out.original, ret)?);
            }
        }
        let application = application(&out.original)?;
        if let Some([ty, ..]) = application {
            inputs.push(builtin(rows, &out.original, ty)?);
        }
        if inputs.len()
            != rows
                .iter()
                .filter(|r| r.kind() == &K::TypeExpression)
                .count()
        {
            return Err(invalid());
        }
        let normalization = TypeNormalizer::default().normalize(&out.symbols, inputs);
        if !normalization.diagnostics().is_empty()
            || normalization.normalized_types().iter().any(|(_, t)| {
                t.status != NormalizedTypeStatus::Known
                    || !t.args.is_empty()
                    || !t.attributes.positive().is_empty()
                    || !t.attributes.negative().is_empty()
            })
        {
            return Err(invalid());
        }
        out.normalization = normalization;
        let declarations = intake(&out)?;
        let roots = declarations
            .iter()
            .filter(|d| !d.7)
            .map(|d| {
                Ok((
                    symbol(&out, d.1, d.6.clone(), false)?,
                    site(&out.original, d.1)?,
                    d.6.clone(),
                    d.3.iter()
                        .map(|ty| normalized(&out, ty))
                        .collect::<Result<Vec<_>, _>>()?,
                    d.4.map(|r| normalized(&out, r)).transpose()?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut unique = BTreeSet::new();
        for (d, root) in declarations.iter().filter(|d| !d.7).zip(&roots) {
            if !unique.insert((d.6.clone(), d.8.clone(), root.3.clone())) {
                return Err(invalid());
            }
        }
        for d in &declarations {
            if let Some(ret) = d.4 {
                let expected = if d.7 {
                    TypeHeadRef::BuiltinSet
                } else {
                    TypeHeadRef::BuiltinObject
                };
                if out
                    .normalization
                    .normalized_types()
                    .get(normalized(&out, ret)?)
                    .ok_or_else(invalid)?
                    .head
                    != expected
                {
                    return Err(invalid());
                }
            }
        }
        if roots.is_empty() {
            return Err(invalid());
        }
        let Some([ty, call, argument, head]) = application else {
            out.roots = roots;
            return Ok(out);
        };
        let actual = normalized(&out, ty)?;
        let matches = roots
            .iter()
            .filter(|root| {
                root.2 == CandidateDeclarationKind::Functor
                    && root.3 == [actual]
                    && declarations.iter().any(|d| {
                        !d.7 && d.8 == head.spelling()
                            && site(&out.original, d.1).ok().as_ref() == Some(&root.1)
                    })
            })
            .collect::<Vec<_>>();
        let [root] = matches.as_slice() else {
            return Err(invalid());
        };
        let declaration = declarations
            .iter()
            .find(|d| site(&out.original, d.1).ok().as_ref() == Some(&root.1))
            .ok_or_else(invalid)?;
        let key = OverloadSiteKey::new(format!("source.redefinition.call:{}", call.site().index()));
        let collection = OverloadCollectionOutput::collect(
            [OverloadSiteInput {
                key: key.clone(),
                owner: site(&out.original, call)?,
                source_range: call.range(),
                kind: OverloadSiteKind::FunctorApplication,
                name: OverloadNameKey::new(head.spelling()),
                arguments: vec![site(&out.original, argument)?],
                expected: None,
                source_qua: vec![],
                recovery: OverloadSiteRecovery::Normal,
            }],
            [OverloadCandidateInput {
                site: key,
                symbol: root.0.clone(),
                ordinary_root: root.0.clone(),
                declaration_kind: root.2.clone(),
                parameters: root.3.clone(),
                result: root.4,
                origin: CandidateOrigin::Ordinary,
                template: None,
                coherence: None,
                provenance: CandidateProvenance {
                    stable_key: CandidateProvenanceKey::new(format!(
                        "source.ordinary:{}",
                        declaration.1.site().index()
                    )),
                    source_range: Some(declaration.1.range()),
                    scope: CandidateScope::Local,
                    declaration_order: declaration.1.preorder_ordinal(),
                },
            }],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability = CandidateViabilityOutput::filter(
            &expansion,
            expansion
                .candidates()
                .iter()
                .map(|(candidate, _)| CandidateViabilityInput {
                    candidate,
                    arguments: vec![ArgumentViabilityEvidence::Exact { actual }],
                }),
        );
        let specificity = SpecificityGraphOutput::build(&viability, []);
        if !collection.diagnostics().is_empty()
            || !expansion.diagnostics().is_empty()
            || !viability.diagnostics().is_empty()
            || !specificity.diagnostics().is_empty()
        {
            return Err(invalid());
        }
        out.roots = roots;
        out.collection = collection;
        out.expansion = expansion;
        out.viability = viability;
        out.specificity = specificity;
        Ok(out)
    }
    pub fn roots(
        &self,
    ) -> impl Iterator<
        Item = (
            &SymbolId,
            &TypedSiteRef,
            CandidateDeclarationKind,
            &[NormalizedTypeId],
            Option<NormalizedTypeId>,
        ),
    > {
        self.roots
            .iter()
            .map(|r| (&r.0, &r.1, r.2.clone(), r.3.as_slice(), r.4))
    }
    pub const fn normalization(&self) -> &TypeNormalizationOutput {
        &self.normalization
    }
    pub const fn collection(&self) -> &OverloadCollectionOutput {
        &self.collection
    }
    pub const fn expansion(&self) -> &TemplateExpansionOutput {
        &self.expansion
    }
    pub const fn viability(&self) -> &CandidateViabilityOutput {
        &self.viability
    }
    pub const fn specificity(&self) -> &SpecificityGraphOutput {
        &self.specificity
    }
}
fn target(roots: &SourceOrdinaryRootHandoff, d: &Roles<'_>) -> Result<SymbolId, String> {
    for ty in &d.3 {
        if roots
            .normalization
            .normalized_types()
            .get(normalized(roots, ty)?)
            .ok_or_else(invalid)?
            .head
            != TypeHeadRef::BuiltinSet
        {
            return Err(invalid());
        }
    }
    let declarations = intake(roots)?;
    let rows = roots.original.source_nodes().ok_or_else(invalid)?;
    let declaration = children(rows, d.1)?;
    let clause = children(rows, declaration.last().ok_or_else(invalid)?)?;
    let explicit = (clause.len() == 5).then(|| clause[2].spelling());
    let mut found = Vec::new();
    for root in roots.roots() {
        let original = declarations
            .iter()
            .find(|r| site(&roots.original, r.1).ok().as_ref() == Some(root.1))
            .ok_or_else(invalid)?;
        if root.2 != d.6
            || original.8 != d.8
            || root.3.len() != d.2.len()
            || original.1.range().end >= d.1.range().start
        {
            continue;
        }
        if let Some(label) = explicit {
            let frame = children(rows, original.1)?;
            if frame[1].spelling() != label {
                continue;
            }
        }
        let mut strict = false;
        for id in root.3 {
            match roots
                .normalization
                .normalized_types()
                .get(*id)
                .ok_or_else(invalid)?
                .head
            {
                TypeHeadRef::BuiltinObject => strict = true,
                TypeHeadRef::BuiltinSet => {}
                _ => return Err(invalid()),
            }
        }
        if strict {
            found.push(root.0.clone());
        }
    }
    let [root] = found.as_slice() else {
        return Err(invalid());
    };
    Ok(root.clone())
}
fn source_attribute_redefinition_roles<'a>(
    roots: &'a SourceOrdinaryRootHandoff,
    p: &[&'a SourceNode],
    formals: &[&SourceNode],
) -> Result<(&'a SourceNode, Option<&'a SourceNode>, String), String> {
    if formals.len() != 1
        || !matches!(p[3].kind(),K::Token(t) if t.kind==T::Identifier)
        || p[3].spelling() != formals[0].spelling()
        || !token(p[4], "is")
        || !token(p[6], "means")
    {
        return Err(invalid());
    }
    let head = only(
        roots.original.source_nodes().ok_or_else(invalid)?,
        p[5],
        &K::AttributePattern,
    )?;
    if !name(head) {
        return Err(invalid());
    }
    Ok((p[7], None, head.spelling().to_owned()))
}
fn source_predicate_redefinition_roles<'a>(
    roots: &'a SourceOrdinaryRootHandoff,
    p: &[&'a SourceNode],
    formals: &[&SourceNode],
) -> Result<(&'a SourceNode, Option<&'a SourceNode>, String), String> {
    if formals.len() != 2 || !token(p[4], "means") || p[3].kind() != &K::PredicatePattern {
        return Err(invalid());
    }
    let frame = children(roots.original.source_nodes().ok_or_else(invalid)?, p[3])?;
    let [left, head, right] = frame.as_slice() else {
        return Err(invalid());
    };
    if !name(head)
        || resolve_template_formal(&roots.native, left.site())? != formals[0].site()
        || resolve_template_formal(&roots.native, right.site())? != formals[1].site()
    {
        return Err(invalid());
    }
    Ok((p[5], None, head.spelling().to_owned()))
}
fn source_functor_redefinition_roles<'a>(
    roots: &'a SourceOrdinaryRootHandoff,
    p: &[&'a SourceNode],
    formals: &[&SourceNode],
) -> Result<(&'a SourceNode, Option<&'a SourceNode>, String), String> {
    if formals.len() != 1
        || !token(p[4], "->")
        || !token(p[6], "equals")
        || p[3].kind() != &K::FunctorPattern
    {
        return Err(invalid());
    }
    let frame = children(roots.original.source_nodes().ok_or_else(invalid)?, p[3])?;
    let [head, locus] = frame.as_slice() else {
        return Err(invalid());
    };
    if !name(head) || resolve_template_formal(&roots.native, locus.site())? != formals[0].site() {
        return Err(invalid());
    }
    Ok((p[7], Some(p[5]), head.spelling().to_owned()))
}

/// Ordered source-derived intake; caller values are checked by fresh derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRedefinitionInput {
    pub relations: Vec<(TypedSiteRef, RelationKind, SymbolId)>,
    pub coherence_intents: Vec<(SymbolId, TypedSiteRef, Option<TypedSiteRef>)>,
    pub targets: Vec<(SymbolId, SymbolId, TypedSiteRef, InitialObligationId)>,
    pub target_diagnostics: Vec<(TypedSiteRef, SymbolId)>,
    pub refinements: Vec<OverloadCandidateInput>,
    pub coherence_inputs: Vec<(InitialObligationId, CoherenceStatus)>,
}
impl SourceRedefinitionInput {
    pub fn from_source(roots: &SourceOrdinaryRootHandoff) -> Result<Self, String> {
        Ok(SourceRedefinitionHandoff::build(Arc::new(roots.clone()))?.input)
    }
}

/// Immutable typed root/target association and pending coherence transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRedefinitionHandoff {
    roots: Arc<SourceOrdinaryRootHandoff>,
    binding_env: BindingEnv,
    inference: TermFormulaInferenceOutput,
    obligations: InitialObligationTable,
    input: SourceRedefinitionInput,
    selection: OverloadSelectionOutput,
}
impl SourceRedefinitionHandoff {
    pub fn build_with_input(
        roots: Arc<SourceOrdinaryRootHandoff>,
        input: SourceRedefinitionInput,
    ) -> Result<Self, String> {
        let expected = Self::build(roots)?;
        if input != expected.input {
            return Err(invalid());
        }
        Ok(expected)
    }
    pub fn build(roots: Arc<SourceOrdinaryRootHandoff>) -> Result<Self, String> {
        if *roots
            != SourceOrdinaryRootHandoff::build(
                roots.original.clone(),
                roots.native.clone(),
                roots.symbols.clone(),
            )?
        {
            return Err(invalid());
        }
        let declarations = intake(&roots)?;
        let rows = roots.original.source_nodes().ok_or_else(invalid)?;
        let shells = source_declaration_shells(&roots.native, &roots.symbols)?;
        let mut contexts = BindingContextTable::new();
        let mut bindings = BindingTable::new();
        let module = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![],
            visible_bindings: vec![],
            recovery: BindingContextRecovery::Normal,
        });
        let mut identity = BTreeMap::new();
        let mut terms = Vec::new();
        let mut formulas = Vec::new();
        let mut obligations = InitialObligationTable::new();
        let mut targets = Vec::new();
        let mut coherence_intents = Vec::new();
        let mut declaration_sites = BTreeSet::new();
        for d in &declarations {
            let mut pending = vec![d.0];
            while let Some(row) = pending.pop() {
                if !declaration_sites.insert(row.site()) {
                    return Err(invalid());
                }
                pending.extend(children(rows, row)?);
            }
            let shell = shells
                .declarations()
                .iter()
                .find(|s| {
                    s.kind() == DeclarationShellKind::DefinitionBlock && s.range() == d.0.range()
                })
                .ok_or_else(invalid)?;
            let context = BindingContextId::new(contexts.len());
            let scope = LocalTermScope::new(vec![
                u32::try_from(d.0.site().index()).map_err(|_| invalid())?,
            ]);
            let mut ids = Vec::new();
            for (formal, guard) in d.2.iter().zip(&d.3) {
                let id = bindings.insert(BindingDraft {
                    spelling: formal.spelling().to_owned(),
                    kind: BindingKind::DefinitionParameter,
                    identity: BinderIdentity::ResolverLocal {
                        scope: scope.clone(),
                        ordinal: formal.preorder_ordinal(),
                        declaration_range: formal.range(),
                    },
                    owner_context: context,
                    declaration_range: formal.range(),
                    visible_after_ordinal: formal.preorder_ordinal(),
                    type_site: BindingTypeSite::Source(guard.range()),
                    status: BindingStatus::Active,
                    captured: CapturedFreeVariables::default(),
                    diagnostics: vec![],
                    recovery: BindingRecoveryState::Normal,
                });
                identity.insert(formal.site(), (id, context, *guard));
                ids.push(id);
            }
            contexts.insert(BindingContextDraft {
                owner: BindingContextOwner::DeclarationShell(shell.id()),
                parent: Some(module),
                layer: BindingContextLayer::Declaration,
                lexical_scope: Some(scope),
                bindings: ids.clone(),
                visible_bindings: ids,
                recovery: BindingContextRecovery::Normal,
            });
            let operands = if d.6 == CandidateDeclarationKind::Functor {
                vec![only(rows, d.5, &K::TermDefiniens)?]
            } else {
                let formula = only(
                    rows,
                    only(rows, d.5, &K::FormulaDefiniens)?,
                    &K::FormulaExpression,
                )?;
                if formula.kind() != &K::BuiltinPredicateApplication {
                    return Err(invalid());
                }
                let frame = children(rows, formula)?;
                let [left, equals, right] = frame.as_slice() else {
                    return Err(invalid());
                };
                if !token(equals, "=") {
                    return Err(invalid());
                }
                formulas.push(
                    FormulaInput::new(
                        site(&roots.original, formula)?,
                        context,
                        formula.range(),
                        FormulaKind::Equality,
                    )
                    .with_terms(vec![
                        site(&roots.original, left)?,
                        site(&roots.original, right)?,
                    ]),
                );
                vec![*left, *right]
            };
            for term in operands {
                let reference = only(rows, term, &K::TermExpression)?;
                let word = only(rows, reference, &K::TermReference)?;
                let formal = resolve_template_formal(&roots.native, word.site())?;
                if !d.2.iter().any(|f| f.site() == formal) {
                    return Err(invalid());
                }
                let (binding, owner, ty) = identity.get(&formal).ok_or_else(invalid)?;
                let mut input = builtin(rows, &roots.original, ty)?;
                input.site = site(&roots.original, term)?;
                input.source_range = term.range();
                terms.push(
                    TermInput::new(input.site.clone(), *owner, term.range(), TermKind::Variable)
                        .with_reference(TermReference::Binding(*binding))
                        .with_result_type(input),
                );
            }
            if d.7 {
                let root = target(&roots, d)?;
                let frame = children(rows, d.1)?;
                let clause = *frame.last().ok_or_else(invalid)?;
                let kind = match d.6 {
                    CandidateDeclarationKind::Attribute => {
                        InitialObligationKind::AttributeRedefinitionCoherence
                    }
                    CandidateDeclarationKind::Predicate => {
                        InitialObligationKind::PredicateRedefinitionCoherence
                    }
                    _ => InitialObligationKind::FunctorRedefinitionCoherence,
                };
                let id = obligations.insert(InitialObligationDraft {
                    kind,
                    owner: site(&roots.original, clause)?,
                    source_range: clause.range(),
                    assumptions: vec![],
                    goal: InitialObligationGoal::new("source.redefinition.coherence"),
                    provenance: InitialObligationProvenance::new("source.redefinition.pending"),
                    status: InitialObligationStatus::Pending,
                });
                targets.push((
                    symbol(&roots, d.1, d.6.clone(), true)?,
                    root,
                    site(&roots.original, clause)?,
                    id,
                ));
                let parts = children(rows, clause)?;
                coherence_intents.push((
                    targets.last().ok_or_else(invalid)?.0.clone(),
                    site(&roots.original, clause)?,
                    if parts.len() == 5 {
                        Some(site(&roots.original, parts[2])?)
                    } else {
                        None
                    },
                ));
            }
        }
        let env = BindingEnv::try_new(BindingEnvParts {
            source_id: roots.original.source_id(),
            module_id: roots.original.module_id().clone(),
            contexts,
            bindings,
            diagnostics: Default::default(),
        })
        .map_err(|_| invalid())?;
        if terms
            .iter()
            .map(|term| term.site.clone())
            .collect::<BTreeSet<_>>()
            != rows
                .iter()
                .filter(|r| r.kind() == &K::TermExpression && declaration_sites.contains(&r.site()))
                .map(|r| site(&roots.original, r))
                .collect::<Result<BTreeSet<_>, _>>()?
            || formulas
                .iter()
                .map(|formula| formula.site.clone())
                .collect::<BTreeSet<_>>()
                != rows
                    .iter()
                    .filter(|r| {
                        r.kind() == &K::BuiltinPredicateApplication
                            && declaration_sites.contains(&r.site())
                    })
                    .map(|r| site(&roots.original, r))
                    .collect::<Result<BTreeSet<_>, _>>()?
        {
            return Err(invalid());
        }
        let inference = TermFormulaChecker::default().infer(&roots.symbols, &env, terms, formulas);
        if !inference.diagnostics().is_empty()
            || !inference.facts().is_empty()
            || !inference.candidate_sets().is_empty()
            || inference
                .terms()
                .iter()
                .any(|(_, t)| t.status != TermStatus::Inferred || !t.deferred.is_empty())
            || inference
                .formulas()
                .iter()
                .any(|(_, f)| f.status != FormulaStatus::Checked)
        {
            return Err(invalid());
        }
        let mut refinements = Vec::new();
        for target in &targets {
            let declaration = declarations
                .iter()
                .find(|d| {
                    d.7 && symbol(&roots, d.1, d.6.clone(), true).ok().as_ref() == Some(&target.0)
                })
                .ok_or_else(invalid)?;
            for (_, call) in roots.collection.sites().iter() {
                if !roots.collection.candidates().iter().any(|(_, candidate)| {
                    candidate.site == call.id && candidate.ordinary_root == target.1
                }) {
                    continue;
                }
                refinements.push(OverloadCandidateInput {
                    site: call.key.clone(),
                    symbol: target.0.clone(),
                    ordinary_root: target.1.clone(),
                    declaration_kind: declaration.6.clone(),
                    parameters: declaration
                        .3
                        .iter()
                        .map(|r| normalized(&roots, r))
                        .collect::<Result<Vec<_>, _>>()?,
                    result: declaration.4.map(|r| normalized(&roots, r)).transpose()?,
                    origin: CandidateOrigin::Redefinition {
                        refined: target.1.clone(),
                    },
                    template: None,
                    coherence: Some(CoherenceStatus::Pending),
                    provenance: CandidateProvenance {
                        stable_key: CandidateProvenanceKey::new(format!(
                            "source.redefinition:{}",
                            declaration.1.site().index()
                        )),
                        source_range: Some(declaration.1.range()),
                        scope: CandidateScope::Local,
                        declaration_order: declaration.1.preorder_ordinal(),
                    },
                });
            }
        }
        let input = SourceRedefinitionInput {
            relations: relations(&roots)?,
            coherence_intents,
            target_diagnostics: targets.iter().map(|t| (t.2.clone(), t.1.clone())).collect(),
            coherence_inputs: targets
                .iter()
                .map(|t| (t.3, CoherenceStatus::Pending))
                .collect(),
            targets,
            refinements,
        };
        // Pending requests reach this intake but do not enter the active set.
        // The sealed ordinary graph is reused without rerunning candidate stages.
        if input
            .refinements
            .iter()
            .any(|candidate| candidate.coherence != Some(CoherenceStatus::Pending))
        {
            return Err(invalid());
        }
        let selection = OverloadSelectionOutput::resolve(
            &roots.specificity,
            roots
                .collection
                .sites()
                .iter()
                .map(|(site, _)| {
                    let ordinary = roots
                        .collection
                        .candidates()
                        .iter()
                        .find(|(_, candidate)| candidate.site == site)
                        .map(|(_, candidate)| candidate)
                        .ok_or_else(invalid)?;
                    Ok(OverloadSiteResolutionInput {
                        site,
                        refinements: vec![],
                        inserted_views: vec![],
                        refinement_join: RefinementJoinPayload {
                            status: RefinementJoinStatus::Compatible,
                            exposed_result: Some(ExposedResultPayload {
                                result: ordinary.result,
                                source: ExposedResultSource::SelectedRoot,
                                evidence: vec![],
                            }),
                        },
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        );
        if !selection.diagnostics().is_empty() {
            return Err(invalid());
        }
        Ok(Self {
            input,
            selection,
            roots,
            binding_env: env,
            inference,
            obligations,
        })
    }
    pub fn roots(&self) -> &SourceOrdinaryRootHandoff {
        &self.roots
    }
    pub const fn input(&self) -> &SourceRedefinitionInput {
        &self.input
    }
    pub const fn selection(&self) -> &OverloadSelectionOutput {
        &self.selection
    }
    pub fn targets(
        &self,
    ) -> impl Iterator<Item = (&SymbolId, &SymbolId, &TypedSiteRef, InitialObligationId)> {
        self.input.targets.iter().map(|r| (&r.0, &r.1, &r.2, r.3))
    }
    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }
    pub const fn inference(&self) -> &TermFormulaInferenceOutput {
        &self.inference
    }
    pub(crate) fn install(&self, receiver: &TypedAst) -> Result<TypedAst, String> {
        if receiver != self.roots.original.as_ref() || *self != Self::build(self.roots.clone())? {
            return Err(invalid());
        }
        let roots = &self.roots;
        let inference = &self.inference;
        let obligations = &self.obligations;
        let rows = roots.original.source_nodes().ok_or_else(invalid)?;
        let shells = source_declaration_shells(&roots.native, &roots.symbols)?;
        let mut local = LocalTypeContextTable::new();
        let mut local_ids = BTreeMap::new();
        let mut block_context = BTreeMap::new();
        let mut module_context = None;
        for (id, context) in self.binding_env.contexts().iter() {
            let (owner, layer) = match context.owner {
                BindingContextOwner::Module => (
                    TypedSiteRef::Node(receiver.nodes().root().ok_or_else(invalid)?),
                    TypeContextLayer::Module,
                ),
                BindingContextOwner::DeclarationShell(shell) => {
                    let block = shells.declaration(shell).ok_or_else(invalid)?;
                    let row = rows
                        .iter()
                        .find(|r| r.kind() == &K::DefinitionBlockItem && r.range() == block.range())
                        .ok_or_else(invalid)?;
                    block_context.insert(row.site(), LocalTypeContextId::new(local.len()));
                    (site(receiver, row)?, TypeContextLayer::Declaration)
                }
                _ => return Err(invalid()),
            };
            let mut sites = Vec::new();
            for binding in &context.bindings {
                let b = self
                    .binding_env
                    .bindings()
                    .get(*binding)
                    .ok_or_else(invalid)?;
                let row = rows
                    .iter()
                    .find(|r| {
                        r.range() == b.declaration_range
                            && matches!(r.kind(),K::Token(t) if t.kind==T::Identifier)
                    })
                    .ok_or_else(invalid)?;
                sites.push(BindingTypeRef::Site(site(receiver, row)?));
            }
            let parent = context
                .parent
                .map(|p| local_ids.get(&p).copied().ok_or_else(invalid))
                .transpose()?;
            let local_id = local.insert(LocalTypeContextDraft {
                owner,
                parent,
                layer,
                bindings: sites,
                introduced_assumptions: vec![],
                visible_facts: vec![],
                recovery: ContextRecoveryState::Normal,
            });
            if context.owner == BindingContextOwner::Module {
                module_context = Some(local_id);
            }
            local_ids.insert(id, local_id);
        }
        let mut types = roots.normalization.type_entries().clone();
        for (_, term) in inference.terms().iter() {
            let entry = inference
                .type_entries()
                .get(term.type_entry)
                .ok_or_else(invalid)?;
            let TypeEntryActual::Known(actual) = entry.actual else {
                return Err(invalid());
            };
            let ty = inference
                .normalized_types()
                .get(actual)
                .ok_or_else(invalid)?;
            let id = roots
                .normalization
                .normalized_types()
                .iter()
                .find(|(_, other)| {
                    other.head == ty.head
                        && other.args == ty.args
                        && other.attributes == ty.attributes
                        && other.status == ty.status
                })
                .map(|(id, _)| id)
                .ok_or_else(invalid)?;
            types.insert(TypeEntryDraft {
                owner: term.site.clone(),
                actual: TypeEntryActual::Known(id),
                expected: None,
                status: TypeStatus::Known,
                provenance: entry.provenance.clone(),
            });
        }
        let mut nodes = roots
            .original
            .nodes()
            .iter()
            .map(|(_, n)| n.clone())
            .collect::<Vec<_>>();
        for (id, entry) in types.iter() {
            let node = nodes
                .get_mut(entry.owner.node().index())
                .ok_or_else(invalid)?;
            node.links.type_entry = Some(id);
            let mut source = receiver
                .nodes()
                .node(entry.owner.node())
                .ok_or_else(invalid)?
                .resolved_node
                .ok_or_else(invalid)?;
            node.links.context = Some(loop {
                if let Some(context) = block_context.get(&source) {
                    break *context;
                }
                let row = rows
                    .iter()
                    .find(|r| r.site() == source)
                    .ok_or_else(invalid)?;
                let Some(parent) = row.parent() else {
                    break module_context.ok_or_else(invalid)?;
                };
                source = parent;
            });
            node.typing = TypingState::Successful;
        }
        for (id, obligation) in obligations.iter() {
            let node = nodes
                .get_mut(obligation.owner.node().index())
                .ok_or_else(invalid)?;
            node.kind = "CoherenceCondition".into();
            node.links.initial_obligations.push(id);
        }
        let nodes =
            TypedArena::try_new(roots.original.nodes().root(), nodes).map_err(|_| invalid())?;
        let mut ast = TypedAst::try_new(TypedAstParts {
            source_id: receiver.source_id(),
            module_id: receiver.module_id().clone(),
            resolved_root: receiver.resolved_root(),
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes,
            contexts: local,
            types,
            facts: Default::default(),
            coercions: Default::default(),
            initial_obligations: self.obligations.clone(),
            diagnostics: Default::default(),
        })
        .map_err(|_| invalid())?;
        ast.install_source_nodes(&self.roots.native)
            .map_err(|_| invalid())?;
        Ok(ast)
    }
    pub(crate) fn validate_installation(
        ast: &TypedAst,
        selection: &OverloadSelectionOutput,
    ) -> Result<(), String> {
        let Some(handoff) = ast.source_redefinition() else {
            if ast.source_node_entries().is_some_and(|mut rows| {
                rows.any(|row| {
                    matches!(
                        row.kind(),
                        K::AttributeRedefinition
                            | K::PredicateRedefinition
                            | K::FunctorRedefinition
                    )
                })
            }) && selection.results().iter().any(|(_, result)| {
                matches!(
                    result.status,
                    OverloadResultStatus::Resolved {
                        exposed_result: Some(_),
                        ..
                    }
                )
            }) {
                return Err(invalid());
            }
            return Ok(());
        };
        let expected = handoff
            .roots
            .original
            .as_ref()
            .clone()
            .with_source_redefinition(handoff.clone())
            .map_err(|_| invalid())?;
        if *ast != expected {
            return Err(invalid());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cluster_trace::ClusterFactTable,
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstInputs},
    };
    use mizar_frontend::{
        lexical_env::*,
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector, env::NamespacePath,
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath,
        PackageId, SourceInput, SourceOriginInput,
    };
    struct NoImports;
    impl LexicalSummaryProvider for NoImports {
        fn resolve_imports(
            &self,
            r: &LexicalEnvironmentRequest<'_>,
        ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
            assert!(r.import_stubs.is_empty());
            Ok(ResolvedImports {
                imports: vec![],
                summaries: vec![],
                diagnostics: vec![],
            })
        }
    }
    fn fixture(form: &str, variant: u8) -> Result<SourceRedefinitionHandoff, String> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let directory = match form {
            "attribute" => "attributes",
            "predicate" => "predicates",
            _ => "functors",
        };
        let family = if matches!(variant, 3 | 6) {
            "explicit"
        } else {
            "intrinsic"
        };
        let source = if variant == 5 {
            root.join(
                "tests/miz/pass/overload/pass_advanced_semantics_redefinition_pending_001.miz",
            )
        } else {
            root.join(format!(
                "tests/miz/pass/{directory}/pass_type_elaboration_redefinition_{family}_{form}_001.miz"
            ))
        };
        static FIXTURE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let fixture_id = FIXTURE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "checker279-private-{}-{form}-{fixture_id}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("src")).unwrap();
        let path = root.join("src/fixture.miz");
        let mut text = std::fs::read_to_string(source).unwrap();
        if variant == 2 {
            text = text.replace("Refined", "Base");
        }
        if variant == 6 {
            text = text.replace("coherence with PRight", "coherence");
        }
        if variant == 1 {
            let (first, tail) = text.split_once("end;").unwrap();
            std::fs::write(
                &path,
                format!(
                    "{first}end;\n{}end;{tail}",
                    first.replace("FBase", "FConflict")
                ),
            )
            .unwrap();
        } else {
            std::fs::write(&path, text).unwrap();
        }
        let output = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            NoImports,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                    "mizar-session-build-snapshot-v1:{:064x}",
                    1
                ))
                .unwrap(),
                input: SourceInput {
                    package_id: PackageId::new("private-redefinitions"),
                    module_path: ModulePath::new(form),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert!(output.diagnostics.is_empty());
        let ast = output.ast.unwrap();
        let module = mizar_resolve::resolved_ast::ModuleId::new(
            PackageId::new("private-redefinitions"),
            ModulePath::new(form),
        );
        let shells = DeclarationShellCollector::new(&ast, &module).collect();
        let collected = SignatureProjectionExtractor::new(&ast, &shells, NamespacePath::new(form))
            .collect(&module);
        if variant == 1 {
            assert!(collected.diagnostics().iter().any(|d| d.class()
                == mizar_resolve::symbols::SymbolDiagnosticClass::SameSignatureDefinitionConflict));
        } else {
            assert!(collected.diagnostics().is_empty());
        }
        let native = Arc::new(SurfaceResolvedArena::lower(&ast, &module).unwrap());
        let roots = SourceOrdinaryRootHandoff::build(
            Arc::new(syntax(&native).unwrap()),
            native,
            Arc::new(collected.into_env()),
        )?;
        SourceRedefinitionHandoff::build(Arc::new(roots))
    }
    #[test]
    fn task279a_private_fresh_replay_rejects_coherent_and_structural_corruptions() {
        assert!(fixture("functor", 1).is_err());
        for form in ["attribute", "predicate", "functor"] {
            assert!(fixture(form, 2).is_err());
            let valid = fixture(form, 0).unwrap();
            let original = valid.roots.original.as_ref();
            let mut rows = original.source_nodes().unwrap().to_vec();
            let clause = rows
                .iter()
                .find(|r| r.kind() == &K::CoherenceCondition)
                .unwrap();
            coherence(&rows, clause).unwrap();
            rows.iter_mut()
                .find(|r| matches!(r.kind(), K::FormulaConstant(_)))
                .unwrap()
                .kind = K::FormulaConstant(SurfaceFormulaConstant::Contradiction);
            let clause = rows
                .iter()
                .find(|r| r.kind() == &K::CoherenceCondition)
                .unwrap();
            assert!(rows.iter().any(|r| token(r, "thesis")));
            assert!(
                rows.iter()
                    .all(|r| r.recovery() == NodeRecoveryState::Normal)
            );
            assert!(coherence(&rows, clause).is_err());

            let installed = original
                .clone()
                .with_source_redefinition(valid.clone())
                .unwrap();
            let collection = OverloadCollectionOutput::collect(vec![], vec![]);
            let expansion = TemplateExpansionOutput::expand(&collection);
            let viability = CandidateViabilityOutput::filter(&expansion, vec![]);
            let specificity = SpecificityGraphOutput::build(&viability, vec![]);
            let selection = OverloadSelectionOutput::resolve(&specificity, vec![]);
            let facts = ClusterFactTable::new();
            let resolve =
                |ast: &TypedAst, hints: Vec<crate::resolved_typed_ast::ResolvedNodeKindHint>| {
                    ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                        typed_ast: ast,
                        cluster_facts: &facts,
                        overload_collection: &collection,
                        template_expansion: &expansion,
                        viability: &viability,
                        specificity: &specificity,
                        overload_selection: &selection,
                        expressions: vec![],
                        node_hints: hints,
                        statement_semantics: None,
                        statement_proofs: None,
                    })
                };
            assert!(resolve(&installed, vec![]).is_ok());
            assert!(
                installed
                    .clone()
                    .with_source_redefinition(valid.clone())
                    .is_err()
            );
            let reject = |bad: SourceRedefinitionHandoff| {
                assert!(
                    original
                        .clone()
                        .with_source_redefinition(bad.clone())
                        .is_err()
                );
                let mut forged = installed.clone();
                forged.inject_source_redefinition_for_test(bad);
                assert!(resolve(&forged, vec![]).is_err());
            };
            for fault in 0..6 {
                let mut bad = valid.clone();
                let roots = Arc::make_mut(&mut bad.roots);
                match fault {
                    0 => roots.roots[0].0 = valid.input.targets[0].0.clone(),
                    1 => roots.roots[0].1 = valid.input.targets[0].2.clone(),
                    2 => roots.roots[0].2 = CandidateDeclarationKind::Mode,
                    3 => roots.roots[0].3.clear(),
                    4 => roots.roots[0].4 = roots.roots[0].3.first().copied(),
                    _ => {
                        roots.roots[0].3[0] = roots
                            .normalization
                            .normalized_types()
                            .iter()
                            .find(|(_, t)| t.head == TypeHeadRef::BuiltinSet)
                            .unwrap()
                            .0
                    }
                }
                if fault == 4 && form == "functor" {
                    roots.roots[0].4 = None;
                }
                reject(bad);
            }
            for fault in 0..6 {
                let mut bad = valid.clone();
                match fault {
                    0 => bad.input.targets[0].1 = bad.input.targets[0].0.clone(),
                    1 => bad.input.targets[0].2 = bad.roots.roots[0].1.clone(),
                    2 => bad.input.targets[0].3 = InitialObligationId::new(1),
                    3 => {
                        bad.input.targets.push(bad.input.targets[0].clone());
                    }
                    4 => {
                        let b = bad
                            .binding_env
                            .binding_mut_for_test(BindingId::new(0))
                            .unwrap();
                        b.owner_context = BindingContextId::new(2);
                    }
                    _ => {
                        let b = bad
                            .binding_env
                            .binding_mut_for_test(BindingId::new(0))
                            .unwrap();
                        b.visible_after_ordinal += 1;
                    }
                }
                reject(bad);
            }
            let mut bad = valid.clone();
            let mut obligations = InitialObligationTable::new();
            for (_, o) in bad.obligations.iter() {
                obligations.insert(InitialObligationDraft {
                    kind: o.kind,
                    owner: o.owner.clone(),
                    source_range: o.source_range,
                    assumptions: vec![],
                    goal: o.goal.clone(),
                    provenance: o.provenance.clone(),
                    status: InitialObligationStatus::Blocked,
                });
            }
            bad.obligations = obligations;
            reject(bad);
            let forged_terms = valid
                .inference
                .terms()
                .iter()
                .map(|(_, t)| {
                    let SourceAnchor::Range(range) =
                        original.nodes().node(t.site.node()).unwrap().anchor
                    else {
                        panic!("actual source term");
                    };
                    TermInput::new(t.site.clone(), t.context, range, t.kind)
                        .with_reference(t.reference.clone().unwrap())
                        .with_result_type(TypeExpressionInput::new(
                            t.site.clone(),
                            range,
                            "object",
                            TypeHeadInput::BuiltinObject,
                        ))
                })
                .collect::<Vec<_>>();
            let forged_formulas = valid
                .inference
                .formulas()
                .iter()
                .map(|(_, f)| {
                    FormulaInput::new(f.site.clone(), f.context, f.source_range, f.kind)
                        .with_terms(f.terms.clone())
                })
                .collect::<Vec<_>>();
            let mut bad = valid.clone();
            bad.inference = TermFormulaChecker::default().infer(
                &valid.roots.symbols,
                &valid.binding_env,
                forged_terms,
                forged_formulas,
            );
            assert!(bad.inference.diagnostics().is_empty());
            assert!(
                bad.inference
                    .terms()
                    .iter()
                    .all(|(_, t)| t.status == TermStatus::Inferred)
            );
            assert!(
                bad.inference
                    .formulas()
                    .iter()
                    .all(|(_, f)| f.status == FormulaStatus::Checked)
            );
            reject(bad);
            let mut bad = valid.clone();
            let roots = Arc::make_mut(&mut bad.roots);
            let inputs = roots
                .normalization
                .type_entries()
                .iter()
                .map(|(_, e)| {
                    TypeExpressionInput::new(
                        e.owner.clone(),
                        match roots.original.nodes().node(e.owner.node()).unwrap().anchor {
                            SourceAnchor::Range(r) => r,
                            _ => unreachable!(),
                        },
                        "object",
                        TypeHeadInput::BuiltinObject,
                    )
                })
                .collect::<Vec<_>>();
            roots.normalization = TypeNormalizer::default().normalize(&roots.symbols, inputs);
            assert!(roots.normalization.diagnostics().is_empty());
            assert!(
                roots
                    .normalization
                    .type_entries()
                    .iter()
                    .all(|(_, e)| e.status == TypeStatus::Known)
            );
            reject(bad);
            for fault in 0..4 {
                let mut nodes = installed
                    .nodes()
                    .iter()
                    .map(|(_, n)| n.clone())
                    .collect::<Vec<_>>();
                let mut types = TypeTable::new();
                let object = valid
                    .roots
                    .normalization
                    .normalized_types()
                    .iter()
                    .find(|(_, t)| t.head == TypeHeadRef::BuiltinObject)
                    .unwrap()
                    .0;
                for (_, e) in installed.types().iter() {
                    types.insert(TypeEntryDraft {
                        owner: e.owner.clone(),
                        expected: e.expected,
                        actual: if fault == 0 {
                            TypeEntryActual::Known(object)
                        } else {
                            e.actual
                        },
                        status: e.status,
                        provenance: e.provenance.clone(),
                    });
                }
                let row = nodes
                    .iter_mut()
                    .find(|n| n.links.type_entry.is_some())
                    .unwrap();
                if fault == 1 {
                    row.links.context = Some(LocalTypeContextId::new(2));
                }
                if fault == 2 {
                    row.kind = "wrong-original-role".into();
                }
                let mut obligations = InitialObligationTable::new();
                for (_, o) in installed.initial_obligations().iter() {
                    obligations.insert(InitialObligationDraft {
                        kind: if fault == 3 {
                            InitialObligationKind::Narrowing
                        } else {
                            o.kind
                        },
                        owner: o.owner.clone(),
                        source_range: o.source_range,
                        assumptions: vec![],
                        goal: o.goal.clone(),
                        provenance: o.provenance.clone(),
                        status: o.status,
                    });
                }
                let mut forged = TypedAst::try_new(TypedAstParts {
                    source_id: installed.source_id(),
                    module_id: installed.module_id().clone(),
                    resolved_root: installed.resolved_root(),
                    source_context: None,
                    source_type: None,
                    source_attribute: None,
                    nodes: TypedArena::try_new(installed.nodes().root(), nodes).unwrap(),
                    contexts: installed.contexts().clone(),
                    types,
                    facts: Default::default(),
                    coercions: Default::default(),
                    initial_obligations: obligations,
                    diagnostics: Default::default(),
                })
                .unwrap();
                forged.install_source_nodes(&valid.roots.native).unwrap();
                forged.inject_source_redefinition_for_test(valid.clone());
                assert!(
                    forged
                        .types()
                        .iter()
                        .all(|(_, e)| e.status == TypeStatus::Known)
                );
                assert!(resolve(&forged, vec![]).is_err());
            }
            let hints = vec![crate::resolved_typed_ast::ResolvedNodeKindHint {
                typed_node: installed.nodes().root().unwrap(),
                kind: crate::resolved_typed_ast::ResolvedNodeKindHintKind::SourcePreserved {
                    role: crate::resolved_typed_ast::SourceNodeRole::new("forged-source"),
                },
            }];
            let plain = valid.install(original).unwrap();
            assert!(resolve(&plain, hints.clone()).is_ok());
            assert!(resolve(&installed, hints).is_err());
            let explicit = fixture(form, 3).unwrap();
            let rows = explicit.roots.original.source_nodes().unwrap();
            let clause = rows
                .iter()
                .find(|r| r.kind() == &K::CoherenceCondition)
                .unwrap();
            for ordinal in [1, 2] {
                let mut bad = rows.to_vec();
                let row = bad
                    .iter_mut()
                    .find(|r| r.parent() == Some(clause.site()) && r.child_ordinal() == ordinal)
                    .unwrap();
                if let K::Token(t) = &mut row.kind {
                    t.kind = T::ReservedSymbol;
                }
                let owner = bad.iter().find(|r| r.site() == clause.site()).unwrap();
                assert!(coherence(&bad, owner).is_err());
            }
            if form == "predicate" {
                assert_eq!(explicit.roots.roots.len(), 2);
                assert_eq!(explicit.input.targets[0].1, explicit.roots.roots[1].0);
                for root in &explicit.roots.roots {
                    let heads = root
                        .3
                        .iter()
                        .map(|id| {
                            explicit
                                .roots
                                .normalization
                                .normalized_types()
                                .get(*id)
                                .unwrap()
                                .head
                                .clone()
                        })
                        .collect::<Vec<_>>();
                    assert!(
                        heads == [TypeHeadRef::BuiltinObject, TypeHeadRef::BuiltinSet]
                            || heads == [TypeHeadRef::BuiltinSet, TypeHeadRef::BuiltinObject]
                    );
                }
                let mut bad = explicit.clone();
                bad.input.targets[0].1 = explicit.roots.roots[0].0.clone();
                let base = explicit.roots.original.as_ref();
                assert!(base.clone().with_source_redefinition(bad.clone()).is_err());
                let mut forged = base.clone().with_source_redefinition(explicit).unwrap();
                assert!(resolve(&forged, vec![]).is_ok());
                forged.inject_source_redefinition_for_test(bad);
                assert!(resolve(&forged, vec![]).is_err());
            }
            let foreign = fixture(
                if form == "attribute" {
                    "predicate"
                } else {
                    "attribute"
                },
                0,
            )
            .unwrap();
            assert!(
                SourceOrdinaryRootHandoff::build(
                    valid.roots.original.clone(),
                    foreign.roots.native.clone(),
                    valid.roots.symbols.clone()
                )
                .is_err()
            );
            assert!(
                SourceOrdinaryRootHandoff::build(
                    valid.roots.original.clone(),
                    valid.roots.native.clone(),
                    foreign.roots.symbols.clone()
                )
                .is_err()
            );
            let mut bad = valid.clone();
            Arc::make_mut(&mut bad.roots).original = foreign.roots.original.clone();
            reject(bad);
        }
    }

    #[test]
    fn task279_refuses_ambiguous_or_unbound_source_targets() {
        assert!(fixture("predicate", 6).is_err());

        let explicit = fixture("predicate", 3).unwrap();
        assert_eq!(explicit.roots.roots.len(), 2);
        let mut ambiguous = explicit.input().clone();
        let chosen = ambiguous.targets[0].1.clone();
        let competing = explicit
            .roots
            .roots
            .iter()
            .map(|root| &root.0)
            .find(|root| **root != chosen)
            .unwrap()
            .clone();
        ambiguous.targets[0].1 = competing;
        assert!(
            SourceRedefinitionHandoff::build_with_input(
                Arc::new(explicit.roots().clone()),
                ambiguous
            )
            .is_err()
        );

        let valid = fixture("functor", 5).unwrap();
        let reject = |input| {
            assert!(
                SourceRedefinitionHandoff::build_with_input(Arc::new(valid.roots().clone()), input)
                    .is_err()
            );
        };
        let mut missing_target = valid.input().clone();
        missing_target.targets.clear();
        reject(missing_target);
        let foreign = fixture("attribute", 0).unwrap();
        let mut missing_root = valid.input().clone();
        missing_root.targets[0].1 = foreign.roots.roots[0].0.clone();
        reject(missing_root);
        let mut missing_anchor = valid.input().clone();
        missing_anchor.targets[0].2 =
            TypedSiteRef::Node(valid.roots.original.nodes().root().unwrap());
        reject(missing_anchor);
    }

    #[test]
    fn task279_refuses_fabricated_input_and_unbound_refinement() {
        let valid = fixture("functor", 5).unwrap();
        let reject = |input| {
            assert!(
                SourceRedefinitionHandoff::build_with_input(Arc::new(valid.roots().clone()), input)
                    .is_err()
            );
        };

        let mut accepted = valid.input().clone();
        accepted.coherence_inputs[0].1 = CoherenceStatus::Accepted;
        reject(accepted);

        let mut alias_semantics = valid.input().clone();
        alias_semantics.relations.push((
            alias_semantics.targets[0].2.clone(),
            RelationKind::Redefinition,
            alias_semantics.targets[0].1.clone(),
        ));
        reject(alias_semantics);

        assert!(!valid.input().refinements.is_empty());
        for corrupt in 0..5 {
            let mut bad = valid.input().clone();
            let candidate = &mut bad.refinements[0];
            match corrupt {
                0 => candidate.coherence = Some(CoherenceStatus::Accepted),
                1 => candidate.origin = CandidateOrigin::Ordinary,
                2 => candidate.result = None,
                3 => {
                    candidate.ordinary_root =
                        fixture("attribute", 0).unwrap().input.targets[0].1.clone()
                }
                _ => candidate.provenance.source_range = None,
            }
            reject(bad);
        }

        let mut unbound_refinement = valid.input().clone();
        unbound_refinement.targets.clear();
        reject(unbound_refinement);

        let mut missing_intent_label = valid.input().clone();
        assert!(missing_intent_label.coherence_intents[0].2.is_some());
        missing_intent_label.coherence_intents[0].2 = None;
        reject(missing_intent_label);

        let mut wrong_diagnostic_anchor = valid.input().clone();
        wrong_diagnostic_anchor.target_diagnostics[0].0 =
            TypedSiteRef::Node(valid.roots.original.nodes().root().unwrap());
        reject(wrong_diagnostic_anchor);
    }

    #[test]
    fn task279_refuses_duplicate_targets_and_reordered_source_roots() {
        let valid = fixture("predicate", 3).unwrap();
        let mut duplicate = valid.input().clone();
        duplicate.targets.push(duplicate.targets[0].clone());
        assert!(
            SourceRedefinitionHandoff::build_with_input(Arc::new(valid.roots().clone()), duplicate)
                .is_err()
        );

        let mut reordered = valid.roots().clone();
        assert!(reordered.roots.len() > 1);
        reordered.roots.swap(0, 1);
        assert!(SourceRedefinitionHandoff::build(Arc::new(reordered)).is_err());
    }

    #[test]
    fn task279_refuses_exposed_application_views_without_the_sealed_handoff() {
        let valid = fixture("functor", 5).unwrap();
        let roots = valid.roots();
        let facts = ClusterFactTable::new();
        let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: roots.original.as_ref(),
            cluster_facts: &facts,
            overload_collection: roots.collection(),
            template_expansion: roots.expansion(),
            viability: roots.viability(),
            specificity: roots.specificity(),
            overload_selection: valid.selection(),
            expressions: vec![],
            node_hints: vec![],
            statement_semantics: None,
            statement_proofs: None,
        });
        assert!(resolved.is_err());

        let mut foreign_selection = valid.clone();
        foreign_selection.selection = OverloadSelectionOutput::resolve(roots.specificity(), []);
        assert!(foreign_selection.install(roots.original.as_ref()).is_err());

        let installed = roots
            .original
            .as_ref()
            .clone()
            .with_source_redefinition(valid.clone())
            .unwrap();
        let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: &installed,
            cluster_facts: &facts,
            overload_collection: roots.collection(),
            template_expansion: roots.expansion(),
            viability: roots.viability(),
            specificity: roots.specificity(),
            overload_selection: valid.selection(),
            expressions: vec![],
            node_hints: vec![],
            statement_semantics: None,
            statement_proofs: None,
        });
        assert!(resolved.is_err());

        assert!(
            roots
                .specificity()
                .graphs()
                .iter()
                .all(|(_, graph)| graph.comparisons.is_empty() && graph.edges.is_empty())
        );

        let mut forged_obligation = valid.clone();
        let mut obligations = InitialObligationTable::new();
        for (_, obligation) in forged_obligation.obligations.iter() {
            obligations.insert(InitialObligationDraft {
                kind: obligation.kind,
                owner: obligation.owner.clone(),
                source_range: obligation.source_range,
                assumptions: obligation.assumptions.clone(),
                goal: obligation.goal.clone(),
                provenance: obligation.provenance.clone(),
                status: InitialObligationStatus::Blocked,
            });
        }
        forged_obligation.obligations = obligations;
        assert!(forged_obligation.install(roots.original.as_ref()).is_err());
    }
}

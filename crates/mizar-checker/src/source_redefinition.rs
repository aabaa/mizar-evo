//! Source-authenticated declaration roots and pending redefinition targets.

use crate::{
    binding_env::*, overload_resolution::CandidateDeclarationKind, type_checker::*, typed_ast::*,
};
use SurfaceNodeKind as K;
use SurfaceTokenKind as T;
use mizar_resolve::{
    declarations::DeclarationShellKind,
    env::{DefinitionKind, SymbolEnv, SymbolKind},
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
    let [word, parameters @ .., decl, end, semi] = frame.as_slice() else {
        return Err(invalid());
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
        block, *decl, formals, guards, ret, body, kind, redefined, notation,
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
        let mut out = Self {
            original,
            native,
            symbols,
            normalization,
            roots: Vec::new(),
        };
        let declarations = intake(&out)?;
        if out.symbols.symbols().len() != declarations.len()
            || out.symbols.definitions().len() != declarations.len()
        {
            return Err(invalid());
        }
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
        out.roots = roots;
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

/// Immutable typed root/target association and pending coherence transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRedefinitionHandoff {
    roots: Arc<SourceOrdinaryRootHandoff>,
    binding_env: BindingEnv,
    inference: TermFormulaInferenceOutput,
    targets: Vec<(SymbolId, SymbolId, TypedSiteRef, InitialObligationId)>,
    obligations: InitialObligationTable,
}
impl SourceRedefinitionHandoff {
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
        for d in &declarations {
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
        if terms.len()
            != rows
                .iter()
                .filter(|r| r.kind() == &K::TermExpression)
                .count()
            || formulas.len()
                != rows
                    .iter()
                    .filter(|r| r.kind() == &K::BuiltinPredicateApplication)
                    .count()
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
        Ok(Self {
            roots,
            binding_env: env,
            inference,
            targets,
            obligations,
        })
    }
    pub fn roots(&self) -> &SourceOrdinaryRootHandoff {
        &self.roots
    }
    pub fn targets(
        &self,
    ) -> impl Iterator<Item = (&SymbolId, &SymbolId, &TypedSiteRef, InitialObligationId)> {
        self.targets.iter().map(|r| (&r.0, &r.1, &r.2, r.3))
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
            while !block_context.contains_key(&source) {
                source = rows
                    .iter()
                    .find(|r| r.site() == source)
                    .and_then(SourceNode::parent)
                    .ok_or_else(invalid)?;
            }
            node.links.context = Some(block_context[&source]);
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
    pub(crate) fn validate_installation(&self, ast: &TypedAst) -> Result<(), String> {
        let expected = self
            .roots
            .original
            .as_ref()
            .clone()
            .with_source_redefinition(self.clone())
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
        overload_resolution::*,
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
        let family = if variant == 3 {
            "explicit"
        } else {
            "intrinsic"
        };
        let source = root.join(format!(
            "tests/miz/pass/{directory}/pass_type_elaboration_redefinition_{family}_{form}_001.miz"
        ));
        let root =
            std::env::temp_dir().join(format!("checker279-private-{}-{form}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        let path = root.join("src/fixture.miz");
        let mut text = std::fs::read_to_string(source).unwrap();
        if variant == 2 {
            text = text.replace("Refined", "Base");
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
                    0 => roots.roots[0].0 = valid.targets[0].0.clone(),
                    1 => roots.roots[0].1 = valid.targets[0].2.clone(),
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
                    0 => bad.targets[0].1 = bad.targets[0].0.clone(),
                    1 => bad.targets[0].2 = bad.roots.roots[0].1.clone(),
                    2 => bad.targets[0].3 = InitialObligationId::new(1),
                    3 => {
                        bad.targets.push(bad.targets[0].clone());
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
                assert_eq!(explicit.targets[0].1, explicit.roots.roots[1].0);
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
                bad.targets[0].1 = explicit.roots.roots[0].0.clone();
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
}

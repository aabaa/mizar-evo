use crate::recovery::SyntaxRecoveryKind;
use std::fmt::Write as _;

use super::{SurfaceAst, SurfaceNodeKind, SurfaceNodeView};

pub(super) fn snapshot_text(ast: &SurfaceAst) -> String {
    let mut output = String::from("surface-ast-snapshot-v1\n");
    output.push_str("root:\n");
    match ast.root_view() {
        Some(root) => write_snapshot_node(&mut output, root, 1),
        None => output.push_str("  <none>\n"),
    }
    output.push_str("expression_root:\n");
    match ast.expression_view() {
        Some(expression) => write_snapshot_node(&mut output, expression, 1),
        None => output.push_str("  <none>\n"),
    }
    output.push_str("token_nodes:\n");
    let mut token_count = 0;
    for token in ast.token_views() {
        token_count += 1;
        write_snapshot_node(&mut output, token, 1);
    }
    if token_count == 0 {
        output.push_str("  <none>\n");
    }
    output
}

fn write_snapshot_node(output: &mut String, view: SurfaceNodeView<'_>, indent: usize) {
    write_snapshot_indent(output, indent);
    match view.kind() {
        SurfaceNodeKind::Root => output.push_str("Root"),
        SurfaceNodeKind::CompilationUnit => output.push_str("CompilationUnit"),
        SurfaceNodeKind::ItemList => output.push_str("ItemList"),
        SurfaceNodeKind::PlaceholderItem => output.push_str("PlaceholderItem"),
        SurfaceNodeKind::ImportItem => output.push_str("ImportItem"),
        SurfaceNodeKind::ImportAliasDecl => output.push_str("ImportAliasDecl"),
        SurfaceNodeKind::ModuleBranchImport => output.push_str("ModuleBranchImport"),
        SurfaceNodeKind::ExportItem => output.push_str("ExportItem"),
        SurfaceNodeKind::VisibilityMarker => output.push_str("VisibilityMarker"),
        SurfaceNodeKind::VisibleItem => output.push_str("VisibleItem"),
        SurfaceNodeKind::ReserveItem => output.push_str("ReserveItem"),
        SurfaceNodeKind::ReserveSegment => output.push_str("ReserveSegment"),
        SurfaceNodeKind::TypeExpression => output.push_str("TypeExpression"),
        SurfaceNodeKind::AttributeChain => output.push_str("AttributeChain"),
        SurfaceNodeKind::AttributeRef => output.push_str("AttributeRef"),
        SurfaceNodeKind::ParameterPrefix => output.push_str("ParameterPrefix"),
        SurfaceNodeKind::TypeHead => output.push_str("TypeHead"),
        SurfaceNodeKind::TypeArguments => output.push_str("TypeArguments"),
        SurfaceNodeKind::TemplateLoci => output.push_str("TemplateLoci"),
        SurfaceNodeKind::TemplateLocus => output.push_str("TemplateLocus"),
        SurfaceNodeKind::TemplateArguments => output.push_str("TemplateArguments"),
        SurfaceNodeKind::TemplateArgument => output.push_str("TemplateArgument"),
        SurfaceNodeKind::AlgorithmDefinition => output.push_str("AlgorithmDefinition"),
        SurfaceNodeKind::AlgorithmParameters => output.push_str("AlgorithmParameters"),
        SurfaceNodeKind::AlgorithmBody => output.push_str("AlgorithmBody"),
        SurfaceNodeKind::AlgorithmStatementList => output.push_str("AlgorithmStatementList"),
        SurfaceNodeKind::VariableDeclaration => output.push_str("VariableDeclaration"),
        SurfaceNodeKind::VariableBinding => output.push_str("VariableBinding"),
        SurfaceNodeKind::AssignmentStatement => output.push_str("AssignmentStatement"),
        SurfaceNodeKind::Lvalue => output.push_str("Lvalue"),
        SurfaceNodeKind::SnapshotStatement => output.push_str("SnapshotStatement"),
        SurfaceNodeKind::ReturnStatement => output.push_str("ReturnStatement"),
        SurfaceNodeKind::ClaimBlockItem => output.push_str("ClaimBlockItem"),
        SurfaceNodeKind::IfStatement => output.push_str("IfStatement"),
        SurfaceNodeKind::WhileStatement => output.push_str("WhileStatement"),
        SurfaceNodeKind::ForRangeStatement => output.push_str("ForRangeStatement"),
        SurfaceNodeKind::ForCollectionStatement => {
            output.push_str("ForCollectionStatement");
        }
        SurfaceNodeKind::MatchStatement => output.push_str("MatchStatement"),
        SurfaceNodeKind::MatchCase => output.push_str("MatchCase"),
        SurfaceNodeKind::MatchEnding => output.push_str("MatchEnding"),
        SurfaceNodeKind::BreakStatement => output.push_str("BreakStatement"),
        SurfaceNodeKind::ContinueStatement => output.push_str("ContinueStatement"),
        SurfaceNodeKind::AlgorithmTerminationClause => {
            output.push_str("AlgorithmTerminationClause");
        }
        SurfaceNodeKind::AlgorithmRequiresClause => output.push_str("AlgorithmRequiresClause"),
        SurfaceNodeKind::AlgorithmEnsuresClause => output.push_str("AlgorithmEnsuresClause"),
        SurfaceNodeKind::AlgorithmDecreasingClause => {
            output.push_str("AlgorithmDecreasingClause");
        }
        SurfaceNodeKind::LoopInvariantClause => output.push_str("LoopInvariantClause"),
        SurfaceNodeKind::LoopDecreasingClause => output.push_str("LoopDecreasingClause"),
        SurfaceNodeKind::AssertStatement => output.push_str("AssertStatement"),
        SurfaceNodeKind::TermList => output.push_str("TermList"),
        SurfaceNodeKind::Annotation => output.push_str("Annotation"),
        SurfaceNodeKind::LibraryAnnotation => output.push_str("LibraryAnnotation"),
        SurfaceNodeKind::AnnotationLabelList => output.push_str("AnnotationLabelList"),
        SurfaceNodeKind::AnnotationLabel => output.push_str("AnnotationLabel"),
        SurfaceNodeKind::AnnotationArgumentList => output.push_str("AnnotationArgumentList"),
        SurfaceNodeKind::AnnotationArgument => output.push_str("AnnotationArgument"),
        SurfaceNodeKind::ProofHintOptionList => output.push_str("ProofHintOptionList"),
        SurfaceNodeKind::ProofHintOption => output.push_str("ProofHintOption"),
        SurfaceNodeKind::StandaloneDiagnosticAnnotation => {
            output.push_str("StandaloneDiagnosticAnnotation");
        }
        SurfaceNodeKind::AnnotatedStatement => output.push_str("AnnotatedStatement"),
        SurfaceNodeKind::AnnotatedAlgorithmStatement => {
            output.push_str("AnnotatedAlgorithmStatement");
        }
        SurfaceNodeKind::AnnotatedDefinitionContent => {
            output.push_str("AnnotatedDefinitionContent");
        }
        SurfaceNodeKind::AnnotatedRegistrationContent => {
            output.push_str("AnnotatedRegistrationContent");
        }
        SurfaceNodeKind::TermPlaceholder => output.push_str("TermPlaceholder"),
        SurfaceNodeKind::TermExpression => output.push_str("TermExpression"),
        SurfaceNodeKind::TermReference => output.push_str("TermReference"),
        SurfaceNodeKind::NumeralTerm => output.push_str("NumeralTerm"),
        SurfaceNodeKind::ItTerm => output.push_str("ItTerm"),
        SurfaceNodeKind::ParenthesizedTerm => output.push_str("ParenthesizedTerm"),
        SurfaceNodeKind::ChoiceTerm => output.push_str("ChoiceTerm"),
        SurfaceNodeKind::ApplicationTerm => output.push_str("ApplicationTerm"),
        SurfaceNodeKind::StructureConstructor => output.push_str("StructureConstructor"),
        SurfaceNodeKind::FieldArgument => output.push_str("FieldArgument"),
        SurfaceNodeKind::SetEnumeration => output.push_str("SetEnumeration"),
        SurfaceNodeKind::SetComprehension => output.push_str("SetComprehension"),
        SurfaceNodeKind::ComprehensionVariableSegment => {
            output.push_str("ComprehensionVariableSegment");
        }
        SurfaceNodeKind::StatementItem => output.push_str("StatementItem"),
        SurfaceNodeKind::LetStatement => output.push_str("LetStatement"),
        SurfaceNodeKind::QualifiedVariableSegment => {
            output.push_str("QualifiedVariableSegment");
        }
        SurfaceNodeKind::AssumptionStatement => output.push_str("AssumptionStatement"),
        SurfaceNodeKind::Proposition => output.push_str("Proposition"),
        SurfaceNodeKind::ConditionList => output.push_str("ConditionList"),
        SurfaceNodeKind::GivenStatement => output.push_str("GivenStatement"),
        SurfaceNodeKind::TakeStatement => output.push_str("TakeStatement"),
        SurfaceNodeKind::Witness => output.push_str("Witness"),
        SurfaceNodeKind::SetStatement => output.push_str("SetStatement"),
        SurfaceNodeKind::Equating => output.push_str("Equating"),
        SurfaceNodeKind::CompactStatement => output.push_str("CompactStatement"),
        SurfaceNodeKind::JustificationClause => output.push_str("JustificationClause"),
        SurfaceNodeKind::ReferenceList => output.push_str("ReferenceList"),
        SurfaceNodeKind::Reference => output.push_str("Reference"),
        SurfaceNodeKind::QualifiedReference => output.push_str("QualifiedReference"),
        SurfaceNodeKind::GroupedReference => output.push_str("GroupedReference"),
        SurfaceNodeKind::GroupedReferenceItem => output.push_str("GroupedReferenceItem"),
        SurfaceNodeKind::BulkReference => output.push_str("BulkReference"),
        SurfaceNodeKind::ComputationJustification => {
            output.push_str("ComputationJustification");
        }
        SurfaceNodeKind::ComputationOption => output.push_str("ComputationOption"),
        SurfaceNodeKind::ConsiderStatement => output.push_str("ConsiderStatement"),
        SurfaceNodeKind::ReconsiderStatement => output.push_str("ReconsiderStatement"),
        SurfaceNodeKind::ReconsiderItem => output.push_str("ReconsiderItem"),
        SurfaceNodeKind::ConclusionStatement => output.push_str("ConclusionStatement"),
        SurfaceNodeKind::ThenStatement => output.push_str("ThenStatement"),
        SurfaceNodeKind::IterativeEqualityStatement => {
            output.push_str("IterativeEqualityStatement");
        }
        SurfaceNodeKind::IterativeEqualityStep => output.push_str("IterativeEqualityStep"),
        SurfaceNodeKind::NowStatement => output.push_str("NowStatement"),
        SurfaceNodeKind::HerebyStatement => output.push_str("HerebyStatement"),
        SurfaceNodeKind::CaseReasoningStatement => {
            output.push_str("CaseReasoningStatement");
        }
        SurfaceNodeKind::CaseItem => output.push_str("CaseItem"),
        SurfaceNodeKind::SupposeItem => output.push_str("SupposeItem"),
        SurfaceNodeKind::InlineFunctorDefinition => {
            output.push_str("InlineFunctorDefinition");
        }
        SurfaceNodeKind::InlinePredicateDefinition => {
            output.push_str("InlinePredicateDefinition");
        }
        SurfaceNodeKind::TypedParameter => output.push_str("TypedParameter"),
        SurfaceNodeKind::TheoremItem => output.push_str("TheoremItem"),
        SurfaceNodeKind::LemmaItem => output.push_str("LemmaItem"),
        SurfaceNodeKind::ProofBlock => output.push_str("ProofBlock"),
        SurfaceNodeKind::DefinitionBlockItem => output.push_str("DefinitionBlockItem"),
        SurfaceNodeKind::PropertyImplementation => output.push_str("PropertyImplementation"),
        SurfaceNodeKind::DefinitionParameter => output.push_str("DefinitionParameter"),
        SurfaceNodeKind::TemplateParameter => output.push_str("TemplateParameter"),
        SurfaceNodeKind::AttributeDefinition => output.push_str("AttributeDefinition"),
        SurfaceNodeKind::AttributePattern => output.push_str("AttributePattern"),
        SurfaceNodeKind::FormulaDefiniens => output.push_str("FormulaDefiniens"),
        SurfaceNodeKind::FormulaCase => output.push_str("FormulaCase"),
        SurfaceNodeKind::CorrectnessCondition => output.push_str("CorrectnessCondition"),
        SurfaceNodeKind::PredicateDefinition => output.push_str("PredicateDefinition"),
        SurfaceNodeKind::PredicatePattern => output.push_str("PredicatePattern"),
        SurfaceNodeKind::FunctorDefinition => output.push_str("FunctorDefinition"),
        SurfaceNodeKind::FunctorPattern => output.push_str("FunctorPattern"),
        SurfaceNodeKind::TermDefiniens => output.push_str("TermDefiniens"),
        SurfaceNodeKind::TermCase => output.push_str("TermCase"),
        SurfaceNodeKind::ModeDefinition => output.push_str("ModeDefinition"),
        SurfaceNodeKind::ModePattern => output.push_str("ModePattern"),
        SurfaceNodeKind::ModeProperty => output.push_str("ModeProperty"),
        SurfaceNodeKind::AttributeRedefinition => output.push_str("AttributeRedefinition"),
        SurfaceNodeKind::PredicateRedefinition => output.push_str("PredicateRedefinition"),
        SurfaceNodeKind::FunctorRedefinition => output.push_str("FunctorRedefinition"),
        SurfaceNodeKind::CoherenceCondition => output.push_str("CoherenceCondition"),
        SurfaceNodeKind::NotationAlias => output.push_str("NotationAlias"),
        SurfaceNodeKind::NotationPattern => output.push_str("NotationPattern"),
        SurfaceNodeKind::PropertyClause => output.push_str("PropertyClause"),
        SurfaceNodeKind::StructureDefinition => output.push_str("StructureDefinition"),
        SurfaceNodeKind::StructurePattern => output.push_str("StructurePattern"),
        SurfaceNodeKind::StructureField => output.push_str("StructureField"),
        SurfaceNodeKind::StructureProperty => output.push_str("StructureProperty"),
        SurfaceNodeKind::InheritanceDefinition => output.push_str("InheritanceDefinition"),
        SurfaceNodeKind::InheritanceTarget => output.push_str("InheritanceTarget"),
        SurfaceNodeKind::FieldRedefinition => output.push_str("FieldRedefinition"),
        SurfaceNodeKind::PropertyRedefinition => output.push_str("PropertyRedefinition"),
        SurfaceNodeKind::RegistrationBlockItem => output.push_str("RegistrationBlockItem"),
        SurfaceNodeKind::RegistrationParameter => output.push_str("RegistrationParameter"),
        SurfaceNodeKind::ExistentialRegistration => output.push_str("ExistentialRegistration"),
        SurfaceNodeKind::ConditionalRegistration => output.push_str("ConditionalRegistration"),
        SurfaceNodeKind::FunctorialRegistration => output.push_str("FunctorialRegistration"),
        SurfaceNodeKind::ReductionRegistration => output.push_str("ReductionRegistration"),
        SurfaceNodeKind::SelectorAccess => output.push_str("SelectorAccess"),
        SurfaceNodeKind::StructureUpdate => output.push_str("StructureUpdate"),
        SurfaceNodeKind::FieldUpdate => output.push_str("FieldUpdate"),
        SurfaceNodeKind::QuaExpression => output.push_str("QuaExpression"),
        SurfaceNodeKind::FormulaExpression => output.push_str("FormulaExpression"),
        SurfaceNodeKind::BuiltinPredicateApplication => {
            output.push_str("BuiltinPredicateApplication");
        }
        SurfaceNodeKind::IsAssertion => output.push_str("IsAssertion"),
        SurfaceNodeKind::AttributeTestChain => output.push_str("AttributeTestChain"),
        SurfaceNodeKind::PredicateApplication => output.push_str("PredicateApplication"),
        SurfaceNodeKind::PredicateSegment => output.push_str("PredicateSegment"),
        SurfaceNodeKind::PredicateHead => output.push_str("PredicateHead"),
        SurfaceNodeKind::InlinePredicateApplication => {
            output.push_str("InlinePredicateApplication");
        }
        SurfaceNodeKind::PrefixFormula(operator) => {
            let _ = write!(
                output,
                "PrefixFormula operator={}",
                operator.snapshot_name()
            );
        }
        SurfaceNodeKind::BinaryFormula(operator) => {
            let _ = write!(
                output,
                "BinaryFormula connective={} repeated={}",
                operator.connective.snapshot_name(),
                operator.repeated
            );
        }
        SurfaceNodeKind::ParenthesizedFormula => output.push_str("ParenthesizedFormula"),
        SurfaceNodeKind::QuantifiedFormula(quantifier) => {
            let _ = write!(
                output,
                "QuantifiedFormula quantifier={}",
                quantifier.snapshot_name()
            );
        }
        SurfaceNodeKind::QuantifierVariableSegment => {
            output.push_str("QuantifierVariableSegment");
        }
        SurfaceNodeKind::FormulaConstant(constant) => {
            let _ = write!(
                output,
                "FormulaConstant constant={}",
                constant.snapshot_name()
            );
        }
        SurfaceNodeKind::ModulePath => output.push_str("ModulePath"),
        SurfaceNodeKind::NamespacePath => output.push_str("NamespacePath"),
        SurfaceNodeKind::QualifiedSymbol => output.push_str("QualifiedSymbol"),
        SurfaceNodeKind::PathSegment => output.push_str("PathSegment"),
        SurfaceNodeKind::RelativePrefix => output.push_str("RelativePrefix"),
        SurfaceNodeKind::Token(token) => {
            let _ = write!(
                output,
                "Token kind={} text=\"{}\"",
                token.kind.snapshot_name(),
                SnapshotEscaped(token.text.as_ref())
            );
        }
        SurfaceNodeKind::InfixExpression(operator) => {
            let _ = write!(
                output,
                "InfixExpression spelling=\"{}\" precedence={} associativity={}",
                SnapshotEscaped(operator.spelling.as_ref()),
                operator.precedence,
                operator.associativity.snapshot_name()
            );
        }
        SurfaceNodeKind::PrefixExpression(operator) => {
            let _ = write!(
                output,
                "PrefixExpression spelling=\"{}\" precedence={}",
                SnapshotEscaped(operator.spelling.as_ref()),
                operator.precedence
            );
        }
        SurfaceNodeKind::PostfixExpression(operator) => {
            let _ = write!(
                output,
                "PostfixExpression spelling=\"{}\" precedence={}",
                SnapshotEscaped(operator.spelling.as_ref()),
                operator.precedence
            );
        }
        SurfaceNodeKind::ErrorRecovery(kind) => {
            let _ = write!(
                output,
                "ErrorRecovery kind={}",
                recovery_snapshot_name(*kind)
            );
        }
    }
    let range = view.range();
    let _ = writeln!(
        output,
        " range={}..{} recovered={}",
        range.start,
        range.end,
        view.is_recovered()
    );
    for child in view.child_views() {
        write_snapshot_node(output, child, indent + 1);
    }
}

fn write_snapshot_indent(output: &mut String, indent: usize) {
    for _ in 0..indent {
        output.push_str("  ");
    }
}

pub(super) fn recovery_snapshot_name(kind: SyntaxRecoveryKind) -> &'static str {
    match kind {
        SyntaxRecoveryKind::ErrorToken => "ErrorToken",
        SyntaxRecoveryKind::MissingEnd => "MissingEnd",
        SyntaxRecoveryKind::MissingStringLiteral => "MissingStringLiteral",
        SyntaxRecoveryKind::MissingItem => "MissingItem",
        SyntaxRecoveryKind::MissingTypeExpression => "MissingTypeExpression",
        SyntaxRecoveryKind::MissingTerm => "MissingTerm",
        SyntaxRecoveryKind::MissingFormula => "MissingFormula",
        SyntaxRecoveryKind::MissingStatement => "MissingStatement",
        SyntaxRecoveryKind::MissingProofStep => "MissingProofStep",
        SyntaxRecoveryKind::MissingAnnotationArgument => "MissingAnnotationArgument",
        SyntaxRecoveryKind::SkippedToken => "SkippedToken",
        SyntaxRecoveryKind::UnmatchedOpeningDelimiter => "UnmatchedOpeningDelimiter",
        SyntaxRecoveryKind::UnmatchedClosingDelimiter => "UnmatchedClosingDelimiter",
        SyntaxRecoveryKind::MalformedAnnotation => "MalformedAnnotation",
    }
}

struct SnapshotEscaped<'a>(&'a str);

impl std::fmt::Display for SnapshotEscaped<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for character in self.0.chars() {
            for escaped in character.escape_default() {
                formatter.write_char(escaped)?;
            }
        }
        Ok(())
    }
}

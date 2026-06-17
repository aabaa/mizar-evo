# mizar-syntax: Surface AST

Status: rowan-backed storage boundary and task-12 compatibility views implemented; full AST coverage planned.

## Purpose

This module defines the source-shaped `SurfaceAst` produced by `mizar-parser`.
`SurfaceAst` is backed by an immutable rowan green tree. The current
`SurfaceNode`/`SurfaceNodeId` surface remains as a compatibility view while the
parser and frontend migrate from the task-12 minimal tree shape.

## Responsibilities

- define `SurfaceAst`, rowan syntax kinds, compatibility syntax node ids, and
  parser-facing construction APIs;
- preserve source order, source ranges, and recovery nodes;
- represent modules, items, terms, formulas, statements, proofs, algorithms, and annotations;
- avoid resolved symbol ids, inferred types, overload resolution results, cluster facts, and proof obligations.

## Public API

### Storage Boundary

`SurfaceAst` owns a rowan green tree. Rowan is the storage backend for syntax
shape and deterministic sharing; it is not the semantic identity surface of the
compiler. Consumers should use the typed accessors on `SurfaceAst` and
`SurfaceNodeView` unless they are explicitly testing the storage boundary.
The raw rowan root is available through `SurfaceAst::rowan_root`, and the green
node through `SurfaceAst::green_node`, for infrastructure tests and carefully
documented integrations.

The task-12 compatibility data (`SurfaceNode`, `SurfaceNodeId`, `token_nodes`,
`root`, and `expression_root`) is backed by private fields inside `SurfaceAst`,
but parts of that surface remain public during migration: compatibility types,
read-only accessors, and `SurfaceNode` constructors/fields are still exported so
`mizar-parser`, `mizar-frontend`, and existing tests can assert the current
minimal shapes. This is a public compatibility API, not the storage backend and
not a stable artifact schema. New consumers should prefer `SurfaceNodeView` and
typed accessors. Compatibility ids and nodes must not be serialized as
cross-run identities, and consumers cannot mutate them independently of the
green tree.

### Syntax Kind Mapping

`SyntaxKind` is the raw rowan kind vocabulary. Node kinds currently map as:

| Surface role | Raw kind |
|---|---|
| root node | `SyntaxKind::Root` |
| compatibility token node | `SyntaxKind::Token` |
| compilation unit node | `SyntaxKind::CompilationUnit` |
| top-level item list node | `SyntaxKind::ItemList` |
| parser task-5 placeholder item node | `SyntaxKind::PlaceholderItem` |
| module path node | `SyntaxKind::ModulePath` |
| namespace path node | `SyntaxKind::NamespacePath` |
| qualified symbol node | `SyntaxKind::QualifiedSymbol` |
| path segment node | `SyntaxKind::PathSegment` |
| relative import prefix node | `SyntaxKind::RelativePrefix` |
| concrete export item node | `SyntaxKind::ExportItem` |
| visibility marker node | `SyntaxKind::VisibilityMarker` |
| visible item wrapper node | `SyntaxKind::VisibleItem` |
| reserve item node | `SyntaxKind::ReserveItem` |
| reserve segment node | `SyntaxKind::ReserveSegment` |
| type expression node | `SyntaxKind::TypeExpression` |
| attribute chain node | `SyntaxKind::AttributeChain` |
| attribute reference node | `SyntaxKind::AttributeRef` |
| parameter prefix node | `SyntaxKind::ParameterPrefix` |
| generic type head node | `SyntaxKind::TypeHead` |
| type arguments node | `SyntaxKind::TypeArguments` |
| legacy bracket term placeholder node | `SyntaxKind::TermPlaceholder` |
| term expression node | `SyntaxKind::TermExpression` |
| term reference node | `SyntaxKind::TermReference` |
| numeral term node | `SyntaxKind::NumeralTerm` |
| `it` term node | `SyntaxKind::ItTerm` |
| parenthesized term node | `SyntaxKind::ParenthesizedTerm` |
| choice term node | `SyntaxKind::ChoiceTerm` |
| application term node | `SyntaxKind::ApplicationTerm` |
| structure constructor node | `SyntaxKind::StructureConstructor` |
| field argument node | `SyntaxKind::FieldArgument` |
| set enumeration node | `SyntaxKind::SetEnumeration` |
| set comprehension node | `SyntaxKind::SetComprehension` |
| comprehension variable segment node | `SyntaxKind::ComprehensionVariableSegment` |
| statement item wrapper node | `SyntaxKind::StatementItem` |
| let statement node | `SyntaxKind::LetStatement` |
| qualified variable segment node | `SyntaxKind::QualifiedVariableSegment` |
| assumption statement node | `SyntaxKind::AssumptionStatement` |
| proposition node | `SyntaxKind::Proposition` |
| condition list node | `SyntaxKind::ConditionList` |
| given statement node | `SyntaxKind::GivenStatement` |
| take statement node | `SyntaxKind::TakeStatement` |
| witness node | `SyntaxKind::Witness` |
| set statement node | `SyntaxKind::SetStatement` |
| equating node | `SyntaxKind::Equating` |
| consider statement node | `SyntaxKind::ConsiderStatement` |
| reconsider statement node | `SyntaxKind::ReconsiderStatement` |
| reconsider item node | `SyntaxKind::ReconsiderItem` |
| conclusion statement node | `SyntaxKind::ConclusionStatement` |
| sequential `then` wrapper node | `SyntaxKind::ThenStatement` |
| iterative equality statement node | `SyntaxKind::IterativeEqualityStatement` |
| iterative equality step node | `SyntaxKind::IterativeEqualityStep` |
| now statement node | `SyntaxKind::NowStatement` |
| hereby statement node | `SyntaxKind::HerebyStatement` |
| case reasoning statement node | `SyntaxKind::CaseReasoningStatement` |
| case item node | `SyntaxKind::CaseItem` |
| suppose item node | `SyntaxKind::SupposeItem` |
| inline functor definition node | `SyntaxKind::InlineFunctorDefinition` |
| inline predicate definition node | `SyntaxKind::InlinePredicateDefinition` |
| typed parameter node | `SyntaxKind::TypedParameter` |
| theorem item node | `SyntaxKind::TheoremItem` |
| lemma item node | `SyntaxKind::LemmaItem` |
| proof block node | `SyntaxKind::ProofBlock` |
| definition block item node | `SyntaxKind::DefinitionBlockItem` |
| definition parameter node | `SyntaxKind::DefinitionParameter` |
| attribute definition node | `SyntaxKind::AttributeDefinition` |
| attribute pattern node | `SyntaxKind::AttributePattern` |
| formula definiens node | `SyntaxKind::FormulaDefiniens` |
| formula case node | `SyntaxKind::FormulaCase` |
| correctness condition node | `SyntaxKind::CorrectnessCondition` |
| predicate definition node | `SyntaxKind::PredicateDefinition` |
| predicate pattern node | `SyntaxKind::PredicatePattern` |
| functor definition node | `SyntaxKind::FunctorDefinition` |
| functor pattern node | `SyntaxKind::FunctorPattern` |
| term definiens node | `SyntaxKind::TermDefiniens` |
| term case node | `SyntaxKind::TermCase` |
| mode definition node | `SyntaxKind::ModeDefinition` |
| mode pattern node | `SyntaxKind::ModePattern` |
| mode property node | `SyntaxKind::ModeProperty` |
| attribute redefinition node | `SyntaxKind::AttributeRedefinition` |
| predicate redefinition node | `SyntaxKind::PredicateRedefinition` |
| functor redefinition node | `SyntaxKind::FunctorRedefinition` |
| coherence condition node | `SyntaxKind::CoherenceCondition` |
| notation alias node | `SyntaxKind::NotationAlias` |
| notation pattern node | `SyntaxKind::NotationPattern` |
| property clause node | `SyntaxKind::PropertyClause` |
| structure definition node | `SyntaxKind::StructureDefinition` |
| structure pattern node | `SyntaxKind::StructurePattern` |
| structure field node | `SyntaxKind::StructureField` |
| structure property node | `SyntaxKind::StructureProperty` |
| inheritance definition node | `SyntaxKind::InheritanceDefinition` |
| inheritance target node | `SyntaxKind::InheritanceTarget` |
| field redefinition node | `SyntaxKind::FieldRedefinition` |
| property redefinition node | `SyntaxKind::PropertyRedefinition` |
| registration block item node | `SyntaxKind::RegistrationBlockItem` |
| registration parameter node | `SyntaxKind::RegistrationParameter` |
| existential registration node | `SyntaxKind::ExistentialRegistration` |
| conditional registration node | `SyntaxKind::ConditionalRegistration` |
| functorial registration node | `SyntaxKind::FunctorialRegistration` |
| reduction registration node | `SyntaxKind::ReductionRegistration` |
| template parameter node | `SyntaxKind::TemplateParameter` |
| template loci list node | `SyntaxKind::TemplateLoci` |
| template locus node | `SyntaxKind::TemplateLocus` |
| template arguments list node | `SyntaxKind::TemplateArguments` |
| template argument node | `SyntaxKind::TemplateArgument` |
| algorithm definition node | `SyntaxKind::AlgorithmDefinition` |
| algorithm parameter list node | `SyntaxKind::AlgorithmParameters` |
| algorithm body node | `SyntaxKind::AlgorithmBody` |
| algorithm statement list node | `SyntaxKind::AlgorithmStatementList` |
| variable declaration node | `SyntaxKind::VariableDeclaration` |
| variable binding node | `SyntaxKind::VariableBinding` |
| assignment statement node | `SyntaxKind::AssignmentStatement` |
| lvalue node | `SyntaxKind::Lvalue` |
| snapshot statement node | `SyntaxKind::SnapshotStatement` |
| return statement node | `SyntaxKind::ReturnStatement` |
| claim block item node | `SyntaxKind::ClaimBlockItem` |
| algorithm `if` statement node | `SyntaxKind::IfStatement` |
| algorithm `while` statement node | `SyntaxKind::WhileStatement` |
| algorithm range-loop statement node | `SyntaxKind::ForRangeStatement` |
| algorithm collection-loop statement node | `SyntaxKind::ForCollectionStatement` |
| algorithm `match` statement node | `SyntaxKind::MatchStatement` |
| algorithm match-case node | `SyntaxKind::MatchCase` |
| algorithm match-ending node | `SyntaxKind::MatchEnding` |
| algorithm `break` statement node | `SyntaxKind::BreakStatement` |
| algorithm `continue` statement node | `SyntaxKind::ContinueStatement` |
| `qua` expression node | `SyntaxKind::QuaExpression` |
| infix expression node | `SyntaxKind::InfixExpression` |
| prefix expression node | `SyntaxKind::PrefixExpression` |
| postfix expression node | `SyntaxKind::PostfixExpression` |
| formula expression node | `SyntaxKind::FormulaExpression` |
| built-in predicate application node | `SyntaxKind::BuiltinPredicateApplication` |
| generic `is` assertion node | `SyntaxKind::IsAssertion` |
| attribute-test chain node | `SyntaxKind::AttributeTestChain` |
| user predicate application node | `SyntaxKind::PredicateApplication` |
| predicate segment node | `SyntaxKind::PredicateSegment` |
| predicate head node | `SyntaxKind::PredicateHead` |
| inline predicate application node | `SyntaxKind::InlinePredicateApplication` |
| prefix formula node | `SyntaxKind::PrefixFormula` |
| binary formula node | `SyntaxKind::BinaryFormula` |
| parenthesized formula node | `SyntaxKind::ParenthesizedFormula` |
| quantified formula node | `SyntaxKind::QuantifiedFormula` |
| quantifier variable segment node | `SyntaxKind::QuantifierVariableSegment` |
| formula constant node | `SyntaxKind::FormulaConstant` |
| recovery node | `SyntaxKind::ErrorRecovery` |

Token roles are separate raw kinds: identifier, reserved word, reserved symbol,
numeral, lexeme run, user symbol, string literal, error-recovery token, and
unknown token. The rowan tree is source-shaped: each token appears once as a
rowan token leaf in source order. Compatibility side tables may retain token
payloads for the task-12 API, but they must not cause duplicated token leaves
or repeated text in the rowan tree.

The current raw discriminants are part of the rowan boundary for this phase:

| Raw value | `SyntaxKind` | Role |
|---:|---|---|
| 0 | `Unknown` | fallback for unrecognized raw rowan kinds |
| 1 | `Root` | root node |
| 2 | `Token` | compatibility token wrapper node |
| 3 | `InfixExpression` | infix expression node |
| 4 | `ErrorRecovery` | recovery node |
| 5 | `ModulePath` | module import/export path node |
| 6 | `NamespacePath` | citation/reference namespace path node |
| 7 | `QualifiedSymbol` | dotted active user symbol node, including attribute-ref structure prefixes |
| 8 | `PathSegment` | single identifier or user-symbol segment wrapper |
| 9 | `RelativePrefix` | `.` / `..` import-relative prefix wrapper |
| 10 | `CompilationUnit` | module file skeleton node |
| 11 | `ItemList` | top-level item list node |
| 12 | `PlaceholderItem` | task-5 keyword-dispatched placeholder item node |
| 13 | `ImportItem` | task-6 concrete `import` item node |
| 14 | `ImportAliasDecl` | task-6 simple import or alias declaration node |
| 15 | `ModuleBranchImport` | task-6 branch import declaration node |
| 16 | `ExportItem` | task-7 concrete `export` item node |
| 17 | `VisibilityMarker` | task-7 `private` / `public` token wrapper |
| 18 | `VisibleItem` | task-7 visible top-level item wrapper |
| 19 | `ReserveItem` | task-8 concrete top-level `reserve` host item |
| 20 | `ReserveSegment` | task-8 `identifier_list "for" type_expression` segment |
| 21 | `TypeExpression` | task-8 `attribute_chain type_head` node |
| 22 | `AttributeChain` | task-8 non-empty sequence of attribute references |
| 23 | `AttributeRef` | task-8 optional `non` plus syntactic attribute reference |
| 24 | `ParameterPrefix` | task-8 attribute parameter-prefix wrapper |
| 25 | `TypeHead` | task-8 generic radix-or-mode type head |
| 26 | `TypeArguments` | task-8 `of` / `over` / bracket argument wrapper |
| 27 | `TermPlaceholder` | legacy task-8 bracket `qua_arg` stub; no longer produced by the task-11 parser path |
| 28 | `TermExpression` | task-9 current term-expression wrapper |
| 29 | `TermReference` | task-9 identifier or qualified-symbol term reference |
| 30 | `NumeralTerm` | task-9 numeral term |
| 31 | `ItTerm` | task-9 `it` term |
| 32 | `ParenthesizedTerm` | task-9 parenthesized term |
| 33 | `ChoiceTerm` | task-9 `"the" type_expression` term |
| 34 | `ApplicationTerm` | task-9 parenthesized or reserved-bracket application term |
| 35 | `StructureConstructor` | task-9 named-field structure-constructor surface |
| 36 | `FieldArgument` | task-9 structure-constructor field argument |
| 37 | `SetEnumeration` | task-9 set-enumeration term |
| 38 | `SelectorAccess` | task-10 selector postfix and selector-call surface |
| 39 | `StructureUpdate` | task-10 functional structure-update postfix |
| 40 | `FieldUpdate` | task-10 structure-update field assignment |
| 41 | `QuaExpression` | task-11 `term "qua" type_expression` qualification surface |
| 42 | `PrefixExpression` | task-12 prefix operator expression surface |
| 43 | `PostfixExpression` | task-12 postfix operator expression surface |
| 44 | `FormulaExpression` | task-13/14 formula wrapper for one formula child |
| 45 | `BuiltinPredicateApplication` | task-13 `term_expression builtin_pred term_expression` atomic formula |
| 46 | `IsAssertion` | task-13 generic `term_expression "is" ...` assertion |
| 47 | `AttributeTestChain` | task-13 attribute-only `is_assertion_body` chain |
| 48 | `PredicateApplication` | task-13 syntax-only user predicate application or chain |
| 49 | `PredicateSegment` | task-13 user predicate segment |
| 50 | `PredicateHead` | task-13 predicate symbol wrapper |
| 51 | `InlinePredicateApplication` | task-13 inline predicate call shape |
| 52 | `PrefixFormula` | task-14 fixed prefix formula shape |
| 53 | `BinaryFormula` | task-14 fixed binary connective formula shape |
| 54 | `ParenthesizedFormula` | task-14 parenthesized formula operand |
| 55 | `QuantifiedFormula` | task-14 universal or existential formula |
| 56 | `QuantifierVariableSegment` | task-14 quantified variable segment |
| 57 | `FormulaConstant` | task-14 `thesis` or `contradiction` formula constant |
| 58 | `SetComprehension` | task-15 set-comprehension / Fraenkel term |
| 59 | `ComprehensionVariableSegment` | task-15 typed generator segment |
| 60 | `StatementItem` | task-16 temporary item host for concrete statements |
| 61 | `LetStatement` | task-16 `let` generalization statement |
| 62 | `QualifiedVariableSegment` | task-16 statement-level qualified variable segment |
| 63 | `AssumptionStatement` | task-16 `assume` / `assume that` statement |
| 64 | `Proposition` | task-16 optional label plus formula proposition |
| 65 | `ConditionList` | task-16 `that` / `and` statement-level condition list |
| 66 | `GivenStatement` | task-16 existential assumption statement |
| 67 | `TakeStatement` | task-16 witness introduction statement |
| 68 | `Witness` | task-16 named or unnamed witness item |
| 69 | `SetStatement` | task-16 local constant-definition statement |
| 70 | `Equating` | task-16 `set` equating item |
| 71 | `CompactStatement` | task-17 minimal explicit-justification compact proposition host |
| 72 | `JustificationClause` | task-17 `by` citation or computation proof clause |
| 73 | `ReferenceList` | task-17 comma-separated citation list |
| 74 | `Reference` | task-17 local reference citation |
| 75 | `QualifiedReference` | task-17 namespace-qualified reference citation |
| 76 | `GroupedReference` | task-17 `namespace_path ".{" ... "}"` grouped citation |
| 77 | `GroupedReferenceItem` | task-17 grouped citation item |
| 78 | `BulkReference` | task-17 `namespace_path ".*"` bulk citation |
| 79 | `ComputationJustification` | task-17 `by computation(...)` justification payload |
| 80 | `ComputationOption` | task-17 `steps` / `timeout` / `nest` computation option |
| 81 | `ConsiderStatement` | task-18 `consider ... such that ... by ...` choice statement |
| 82 | `ReconsiderStatement` | task-18 `reconsider ... as ... by ...` type-changing statement |
| 83 | `ReconsiderItem` | task-18 bare or equated reconsider item |
| 84 | `ConclusionStatement` | task-19 `thus` / `hence` conclusion statement |
| 85 | `ThenStatement` | task-19 `then` modifier wrapper around a linkable statement |
| 86 | `IterativeEqualityStatement` | task-19 equality chain with at least one `.=` continuation |
| 87 | `IterativeEqualityStep` | task-19 `.=` equality-chain continuation step |
| 88 | `NowStatement` | task-20 labelled reasoning block |
| 89 | `HerebyStatement` | task-20 diffuse conclusion block |
| 90 | `CaseReasoningStatement` | task-20 `per cases` reasoning block |
| 91 | `CaseItem` | task-20 `case ... end;` branch |
| 92 | `SupposeItem` | task-20 `suppose ... end;` branch |
| 93 | `InlineFunctorDefinition` | task-21 `deffunc ... equals ...;` local definition |
| 94 | `InlinePredicateDefinition` | task-21 `defpred ... means ...;` local definition |
| 95 | `TypedParameter` | task-21 inline-definition typed parameter |
| 96 | `TheoremItem` | task-22 theorem declaration item |
| 97 | `LemmaItem` | task-22 lemma declaration item |
| 98 | `ProofBlock` | task-22 `proof ... end` justification block |
| 100 | `TokenIdentifier` | identifier token leaf |
| 101 | `TokenReservedWord` | reserved-word token leaf |
| 102 | `TokenReservedSymbol` | reserved-symbol token leaf |
| 103 | `TokenNumeral` | numeral token leaf |
| 104 | `TokenLexemeRun` | lexeme-run token leaf |
| 105 | `TokenUserSymbol` | user-symbol token leaf |
| 106 | `TokenStringLiteral` | string-literal token leaf |
| 107 | `TokenErrorRecovery` | lexer recovery token leaf |
| 108 | `TokenUnknown` | unknown token leaf |
| 109 | `DefinitionBlockItem` | task-23 `definition ... end;` item |
| 110 | `DefinitionParameter` | task-23 ordinary definition `let` parameter |
| 111 | `AttributeDefinition` | task-23 `attr ... means ...;` definition |
| 112 | `AttributePattern` | task-23 attribute pattern head |
| 113 | `FormulaDefiniens` | task-23 formula definiens body |
| 114 | `FormulaCase` | task-23 conditional formula definiens case |
| 115 | `CorrectnessCondition` | task-23 correctness-condition clause |
| 116 | `PredicateDefinition` | task-24 `pred ... means ...;` definition |
| 117 | `PredicatePattern` | task-24 raw predicate definition pattern |
| 118 | `FunctorDefinition` | task-25 `func ... means|equals ...;` definition |
| 119 | `FunctorPattern` | task-25 raw functor definition pattern |
| 120 | `TermDefiniens` | task-25 term definiens body |
| 121 | `TermCase` | task-25 conditional term definiens case |
| 122 | `ModeDefinition` | task-26 `mode ... is ...;` definition |
| 123 | `ModePattern` | task-26 raw mode definition pattern |
| 124 | `ModeProperty` | task-26 `sethood` property attached to a mode definition |
| 125 | `AttributeRedefinition` | task-27 `redefine attr ... means ...; coherence ...;` definition |
| 126 | `PredicateRedefinition` | task-27 `redefine pred ... means ...; coherence ...;` definition |
| 127 | `FunctorRedefinition` | task-27 `redefine func ... -> ... means|equals ...; coherence ...;` definition |
| 128 | `CoherenceCondition` | task-27 mandatory redefinition coherence proof tail |
| 129 | `NotationAlias` | task-27 `synonym` / `antonym` notation alias declaration |
| 130 | `NotationPattern` | task-27 raw notation alias pattern span |
| 131 | `PropertyClause` | task-28 predicate/functor/standalone mode property clause |
| 132 | `StructureDefinition` | task-29 `struct ... where ... end;` definition |
| 133 | `StructurePattern` | task-29 raw structure definition name and parameters |
| 134 | `StructureField` | task-29 structure `field` member |
| 135 | `StructureProperty` | task-29 structure `property` member |
| 136 | `InheritanceDefinition` | task-29 `inherit ... extends ...` definition |
| 137 | `InheritanceTarget` | task-29 raw inheritance child/parent target |
| 138 | `FieldRedefinition` | task-29 inherited field mapping |
| 139 | `PropertyRedefinition` | task-29 inherited property mapping |
| 140 | `RegistrationBlockItem` | task-30 `registration ... end;` block item |
| 141 | `RegistrationParameter` | task-30 registration-local `let` parameter |
| 142 | `ExistentialRegistration` | task-30 existential `cluster ... existence ...;` registration |
| 143 | `ConditionalRegistration` | task-30 conditional `cluster ... -> ... coherence ...;` registration |
| 144 | `FunctorialRegistration` | task-30 functorial `cluster term -> ... coherence ...;` registration |
| 145 | `ReductionRegistration` | task-30 `reduce ... to ... reducibility ...;` registration |
| 146 | `TemplateParameter` | task-31 template definition `let` parameter |
| 147 | `TemplateLoci` | task-31 pattern-side `[` locus list `]` wrapper |
| 148 | `TemplateLocus` | task-31 single pattern-side template locus |
| 149 | `TemplateArguments` | task-31 call/reference-side template argument list |
| 150 | `TemplateArgument` | task-31 single call/reference-side template argument |
| 151 | `AlgorithmDefinition` | task-32 `algorithm ... do ... end;` definition content |
| 152 | `AlgorithmParameters` | task-32 algorithm formal parameter list |
| 153 | `AlgorithmBody` | task-32 algorithm `do ... end` body |
| 154 | `AlgorithmStatementList` | task-32 source-ordered algorithm statement list |
| 155 | `VariableDeclaration` | task-32 `var` / `const` / ghost declaration statement |
| 156 | `VariableBinding` | task-32 single declaration binding |
| 157 | `AssignmentStatement` | task-32 assignment or ghost assignment statement |
| 158 | `Lvalue` | task-32 syntactic assignment target |
| 159 | `SnapshotStatement` | task-32 `snapshot` statement |
| 160 | `ReturnStatement` | task-32 `return` statement |
| 161 | `ClaimBlockItem` | task-32 top-level `claim ... do ... end;` item |
| 162 | `IfStatement` | task-33 algorithm `if ... do ... [else ...] end;` statement |
| 163 | `WhileStatement` | task-33 algorithm `while ... do ... end;` statement |
| 164 | `ForRangeStatement` | task-33 algorithm `for i = ... to|downto ... [step ...] do ... end;` statement |
| 165 | `ForCollectionStatement` | task-33 algorithm `for x in S [processed V] do ... end;` statement |
| 166 | `MatchStatement` | task-33 algorithm `match ... do ... end;` statement |
| 167 | `MatchCase` | task-33 `case pattern do ... end;` branch inside algorithm `match` |
| 168 | `MatchEnding` | task-33 `otherwise ... end;` or `exhaustive [justification];` match ending |
| 169 | `BreakStatement` | task-33 algorithm `break;` statement |
| 170 | `ContinueStatement` | task-33 algorithm `continue;` statement |
| 171 | `AlgorithmTerminationClause` | task-34 `terminating` algorithm modifier |
| 172 | `AlgorithmRequiresClause` | task-34 header `requires formula` clause |
| 173 | `AlgorithmEnsuresClause` | task-34 header `ensures formula` clause |
| 174 | `AlgorithmDecreasingClause` | task-34 header `decreasing term_list` clause |
| 175 | `LoopInvariantClause` | task-34 leading loop `invariant formula [justification];` clause |
| 176 | `LoopDecreasingClause` | task-34 leading while-loop `decreasing term_list [justification];` clause |
| 177 | `AssertStatement` | task-34 algorithm `assert formula [justification];` statement |
| 178 | `TermList` | task-34 comma-separated decreasing-measure term list |

`SyntaxKind::from_raw` maps any unknown raw value to `Unknown`.
`SyntaxKind::is_node_kind` is true for every structural node raw kind listed
above, including `Root` through task-22 `ProofBlock`, task-23
`DefinitionBlockItem` through task-34 `TermList`, the compatibility
`Token` wrapper, and `ErrorRecovery`; `is_token_kind` is true only for token
leaf raw kinds `TokenIdentifier` through `TokenUnknown`. Future raw values should be
appended or assigned into a documented reserved range so existing snapshots and
rowan tests fail loudly when the raw vocabulary changes.

### Current Surface Vocabulary

The current implemented surface node vocabulary is deliberately small:

| Public surface kind | Payload | Raw rowan node kind | Notes |
|---|---|---|---|
| `SurfaceNodeKind::Root` | none | `SyntaxKind::Root` | top-level compatibility root |
| `SurfaceNodeKind::Token(SurfaceToken)` | token kind and interned text | `SyntaxKind::Token` with one token leaf of the token raw kind | compatibility wrapper around a rowan token leaf |
| `SurfaceNodeKind::CompilationUnit` | none | `SyntaxKind::CompilationUnit` | parser task-5 module file skeleton; one `ItemList` child and no semantic module identity |
| `SurfaceNodeKind::ItemList` | none | `SyntaxKind::ItemList` | source-order list of top-level item placeholders, temporary `StatementItem` hosts, and item-level recovery markers |
| `SurfaceNodeKind::PlaceholderItem` | none | `SyntaxKind::PlaceholderItem` | keyword-dispatched top-level item placeholder used until later tasks replace it with concrete item nodes |
| `SurfaceNodeKind::ImportItem` | none | `SyntaxKind::ImportItem` | parser task-6 concrete `import_stmt`; owns the `import` token, import declaration nodes separated by comma tokens, optional malformed-tail recovery, and optional semicolon token |
| `SurfaceNodeKind::ImportAliasDecl` | none | `SyntaxKind::ImportAliasDecl` | parser task-6 `module_path ["as" module_identifier]`; owns a `ModulePath`, optional `as` token, optional alias `PathSegment`, and optional malformed-tail recovery |
| `SurfaceNodeKind::ModuleBranchImport` | none | `SyntaxKind::ModuleBranchImport` | parser task-6 `module_path ".{" module_identifier { "," module_identifier } "}"`; owns a base `ModulePath`, `.{` token, branch `PathSegment`s separated by comma tokens, optional malformed-tail recovery, and optional `}` |
| `SurfaceNodeKind::ExportItem` | none | `SyntaxKind::ExportItem` | parser task-7 concrete `export_stmt`; owns the `export` token, exported `ModulePath` nodes separated by comma tokens, optional malformed-tail recovery, and optional semicolon token |
| `SurfaceNodeKind::VisibilityMarker` | none | `SyntaxKind::VisibilityMarker` | parser task-7 wrapper for exactly one `private` or `public` token |
| `SurfaceNodeKind::VisibleItem` | none | `SyntaxKind::VisibleItem` | parser task-7 top-level visibility wrapper; owns annotation-prefix tokens when present, one `VisibilityMarker`, and the current target item node |
| `SurfaceNodeKind::ReserveItem` | none | `SyntaxKind::ReserveItem` | parser task-8 concrete top-level `reserve_decl` host item; owns the `reserve` token, one `ReserveSegment`, optional malformed-tail recovery, and optional semicolon token |
| `SurfaceNodeKind::ReserveSegment` | none | `SyntaxKind::ReserveSegment` | parser task-8 `identifier_list "for" type_expression`; owns identifier tokens separated by comma tokens, the `for` token, and a `TypeExpression` or missing-type recovery |
| `SurfaceNodeKind::TypeExpression` | none | `SyntaxKind::TypeExpression` | parser task-8 `attribute_chain type_head`; owns an optional non-empty `AttributeChain` followed by a generic `TypeHead` |
| `SurfaceNodeKind::AttributeChain` | none | `SyntaxKind::AttributeChain` | parser task-8 non-empty source-ordered sequence of `AttributeRef` nodes |
| `SurfaceNodeKind::AttributeRef` | none | `SyntaxKind::AttributeRef` | parser task-8 optional `non` token, optional `ParameterPrefix`, syntactic `QualifiedSymbol`, and optional parenthesized term arguments |
| `SurfaceNodeKind::ParameterPrefix` | none | `SyntaxKind::ParameterPrefix` | parser task-8 attribute parameter prefix, either `parameter "-"` or `"(" parameter_list ")" "-"` |
| `SurfaceNodeKind::TypeHead` | none | `SyntaxKind::TypeHead` | parser task-8 generic radix-or-mode head; owns builtin `object`/`set` token or `QualifiedSymbol`, plus optional `TypeArguments` |
| `SurfaceNodeKind::TypeArguments` | none | `SyntaxKind::TypeArguments` | parser task-8 type argument wrapper for `of`, `over`, or bracket syntax; task 9 replaces `of`/`over` placeholders with `TermExpression` arguments, and task 11 replaces bracket `qua_arg` placeholders with `TermExpression` / `QuaExpression` surfaces |
| `SurfaceNodeKind::TermPlaceholder` | none | `SyntaxKind::TermPlaceholder` | legacy parser task-8 syntax-only term-entry stub retained for raw-kind compatibility; the task-11 parser no longer emits it for bracket `qua_arg` forms |
| `SurfaceNodeKind::TermExpression` | none | `SyntaxKind::TermExpression` | parser task-9 current term-expression wrapper; owns exactly one current term-shape child, which may be a primary term, postfix chain, `QuaExpression`, or later operator expression |
| `SurfaceNodeKind::TermReference` | none | `SyntaxKind::TermReference` | parser task-9 identifier token or shared `QualifiedSymbol` in term position, with optional task-31 `TemplateArguments` before parenthesized application and no semantic classification |
| `SurfaceNodeKind::NumeralTerm` | none | `SyntaxKind::NumeralTerm` | parser task-9 numeral term wrapper |
| `SurfaceNodeKind::ItTerm` | none | `SyntaxKind::ItTerm` | parser task-9 `it` keyword term wrapper |
| `SurfaceNodeKind::ParenthesizedTerm` | none | `SyntaxKind::ParenthesizedTerm` | parser task-9 parenthesized term; owns `(`, a `TermExpression` or `MissingTerm`, and optional `)` |
| `SurfaceNodeKind::ChoiceTerm` | none | `SyntaxKind::ChoiceTerm` | parser task-9 `"the" TypeExpression` choice term; missing type operands use `MissingTypeExpression` recovery |
| `SurfaceNodeKind::ApplicationTerm` | none | `SyntaxKind::ApplicationTerm` | parser task-9 ordinary parenthesized application or reserved-bracket functor application; owns delimiters and source-ordered term arguments |
| `SurfaceNodeKind::StructureConstructor` | none | `SyntaxKind::StructureConstructor` | parser task-9 syntax-only structure-constructor surface when named field arguments are visible |
| `SurfaceNodeKind::FieldArgument` | none | `SyntaxKind::FieldArgument` | parser task-9 `identifier ":" term_expression` field argument |
| `SurfaceNodeKind::SetEnumeration` | none | `SyntaxKind::SetEnumeration` | parser task-9 set-enumeration term |
| `SurfaceNodeKind::SetComprehension` | none | `SyntaxKind::SetComprehension` | parser task-15 set-comprehension / Fraenkel term; owns `{`, a mapper `TermExpression`, `where`, generator segments, optional condition formula, and `}` or delimiter recovery |
| `SurfaceNodeKind::ComprehensionVariableSegment` | none | `SyntaxKind::ComprehensionVariableSegment` | parser task-15 typed generator segment; owns identifier or `MissingTerm` recovery, optional `is`, and `TypeExpression` or `MissingTypeExpression` recovery when `is` is present |
| `SurfaceNodeKind::StatementItem` | none | `SyntaxKind::StatementItem` | parser task-16 temporary module-level statement host; owns exactly one concrete parser-owned statement node from the currently implemented S-013 / S-014 statement vocabulary and no statement-level annotation payload |
| `SurfaceNodeKind::LetStatement` | none | `SyntaxKind::LetStatement` | parser task-16/17 `let` generalization; owns `let`, qualified-variable segments with separator commas, optional `such` plus `ConditionList`, optional task-17 simple `JustificationClause`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::QualifiedVariableSegment` | none | `SyntaxKind::QualifiedVariableSegment` | parser task-16 statement-level variable segment; owns identifier tokens separated by comma tokens, optional `be` / `being`, and optional `TypeExpression` or `MissingTypeExpression` recovery |
| `SurfaceNodeKind::AssumptionStatement` | none | `SyntaxKind::AssumptionStatement` | parser task-16 `assume` or `assume that`; owns `assume` plus either one `Proposition` or one `ConditionList`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::Proposition` | none | `SyntaxKind::Proposition` | parser task-16 proposition surface; owns optional label identifier plus colon and one `FormulaExpression` or `MissingFormula` recovery |
| `SurfaceNodeKind::ConditionList` | none | `SyntaxKind::ConditionList` | parser task-16 statement-level conditions; owns `that`, one or more `Proposition` children separated by `and` tokens, and optional recovery |
| `SurfaceNodeKind::GivenStatement` | none | `SyntaxKind::GivenStatement` | parser task-16 existential assumption; owns `given`, qualified-variable segments with separator commas, optional `such` plus `ConditionList`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::TakeStatement` | none | `SyntaxKind::TakeStatement` | parser task-16 witness introduction; owns `take`, one or more `Witness` children separated by comma tokens, optional recovery, and optional semicolon |
| `SurfaceNodeKind::Witness` | none | `SyntaxKind::Witness` | parser task-16 witness item; owns either one `TermExpression` or identifier, `=`, and a `TermExpression` / `MissingTerm` recovery |
| `SurfaceNodeKind::SetStatement` | none | `SyntaxKind::SetStatement` | parser task-16 local constant definition; owns `set`, one or more `Equating` children separated by comma tokens, optional recovery, and optional semicolon |
| `SurfaceNodeKind::Equating` | none | `SyntaxKind::Equating` | parser task-16 equating item; owns identifier or `MissingTerm` recovery, `=` when present, and a `TermExpression` or `MissingTerm` recovery |
| `SurfaceNodeKind::ConsiderStatement` | none | `SyntaxKind::ConsiderStatement` | parser task-18 choice statement; owns `consider`, qualified-variable segments with separator commas, `such`, `ConditionList` or condition recovery, simple `JustificationClause` or missing-justification recovery, optional recovery, and optional semicolon |
| `SurfaceNodeKind::ReconsiderStatement` | none | `SyntaxKind::ReconsiderStatement` | parser task-18 type-changing statement; owns `reconsider`, reconsider items with separator commas, `as`, `TypeExpression` or `MissingTypeExpression`, simple `JustificationClause` or missing-justification recovery, optional recovery, and optional semicolon |
| `SurfaceNodeKind::ReconsiderItem` | none | `SyntaxKind::ReconsiderItem` | parser task-18 reconsider item; owns either identifier or identifier, `=`, and `TermExpression` / `MissingTerm` recovery |
| `SurfaceNodeKind::ConclusionStatement` | none | `SyntaxKind::ConclusionStatement` | parser task-19 conclusion statement plus parser task-22 full proof justification; owns `thus` or `hence`, one `Proposition`, an optional explicit `JustificationClause` or `ProofBlock`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::ThenStatement` | none | `SyntaxKind::ThenStatement` | parser task-19 sequential modifier wrapper; owns the `then` token and exactly one linkable statement child or `MissingStatement` recovery |
| `SurfaceNodeKind::IterativeEqualityStatement` | none | `SyntaxKind::IterativeEqualityStatement` | parser task-19 equality chain; owns optional label identifier/colon, first `TermExpression`, `=`, second `TermExpression`, optional simple `JustificationClause`, one or more `IterativeEqualityStep` children, optional recovery, and optional semicolon |
| `SurfaceNodeKind::IterativeEqualityStep` | none | `SyntaxKind::IterativeEqualityStep` | parser task-19 `.=` continuation; owns `.=` token, a `TermExpression` or `MissingTerm`, and optional simple `JustificationClause` |
| `SurfaceNodeKind::NowStatement` | none | `SyntaxKind::NowStatement` | parser task-20 `now ... end;` reasoning block; owns optional label identifier/colon, `now`, nested statement nodes, optional recovery including `MissingEnd`, `end` when present, and optional semicolon |
| `SurfaceNodeKind::HerebyStatement` | none | `SyntaxKind::HerebyStatement` | parser task-20 `hereby ... end;` diffuse conclusion block; owns `hereby`, nested statement nodes, optional recovery including `MissingEnd`, `end` when present, and optional semicolon |
| `SurfaceNodeKind::CaseReasoningStatement` | none | `SyntaxKind::CaseReasoningStatement` | parser task-20 `per cases` block; owns `per`, `cases`, optional simple `JustificationClause`, header semicolon, and either source-ordered homogeneous `CaseItem` children or source-ordered homogeneous `SupposeItem` children |
| `SurfaceNodeKind::CaseItem` | none | `SyntaxKind::CaseItem` | parser task-20 `case ... end;` branch; owns `case`, either `Proposition` or `ConditionList`, header semicolon, nested statement nodes, optional recovery including `MissingEnd`, `end` when present, and optional semicolon |
| `SurfaceNodeKind::SupposeItem` | none | `SyntaxKind::SupposeItem` | parser task-20 `suppose ... end;` branch; owns `suppose`, either `Proposition` or `ConditionList`, header semicolon, nested statement nodes, optional recovery including `MissingEnd`, `end` when present, and optional semicolon |
| `SurfaceNodeKind::InlineFunctorDefinition` | none | `SyntaxKind::InlineFunctorDefinition` | parser task-21 standalone `deffunc` definition; owns `deffunc`, a name identifier or `MissingTerm` recovery, parameter parentheses, zero or more `TypedParameter` children separated by comma tokens, `->`, a return `TypeExpression` or `MissingTypeExpression`, `equals`, a body `TermExpression` or `MissingTerm`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::InlinePredicateDefinition` | none | `SyntaxKind::InlinePredicateDefinition` | parser task-21 standalone `defpred` definition; owns `defpred`, a name identifier or `MissingTerm` recovery, parameter parentheses, zero or more `TypedParameter` children separated by comma tokens, `means`, a body `FormulaExpression` or `MissingFormula`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::TypedParameter` | none | `SyntaxKind::TypedParameter` | parser task-21 inline-definition parameter; owns the parameter identifier when present, optional `be` or `being` when written, and a `TypeExpression` or `MissingTypeExpression` recovery |
| `SurfaceNodeKind::TheoremItem` | none | `SyntaxKind::TheoremItem` | parser task-22 theorem declaration; owns optional status tokens preserved syntactically, `theorem`, a label identifier or `MissingTerm`, `:`, a `FormulaExpression` or `MissingFormula`, optional `JustificationClause` or `ProofBlock`, optional recovery, and the final semicolon when present |
| `SurfaceNodeKind::LemmaItem` | none | `SyntaxKind::LemmaItem` | parser task-22 lemma declaration with the same source-order children as `TheoremItem`, selected by the `lemma` role token |
| `SurfaceNodeKind::ProofBlock` | none | `SyntaxKind::ProofBlock` | parser task-22 full proof justification block; owns `proof`, nested statement nodes from the reasoning body, optional recovery including `MissingEnd`, and `end` when present; the enclosing theorem or statement owns the following semicolon |
| `SurfaceNodeKind::DefinitionBlockItem` | none | `SyntaxKind::DefinitionBlockItem` | parser task-23 `definition ... end;` block item; owns `definition`, source-ordered concrete definition content or placeholder content, optional `MissingEnd`, `end` when present, and the final semicolon when present |
| `SurfaceNodeKind::DefinitionParameter` | none | `SyntaxKind::DefinitionParameter` | parser task-23 ordinary definition parameter; owns `let`, one or more qualified-variable segments, optional conditions or justification, optional recovery, and optional semicolon |
| `SurfaceNodeKind::AttributeDefinition` | none | `SyntaxKind::AttributeDefinition` | parser task-23 `attr` definition; owns `attr`, a label identifier or `MissingTerm`, `:`, a subject identifier or `MissingTerm`, `is`, an `AttributePattern`, `means`, a `FormulaDefiniens`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::AttributePattern` | none | `SyntaxKind::AttributePattern` | parser task-23 attribute pattern head; owns an optional `ParameterPrefix` plus an identifier or user-symbol name, or `MissingTerm` when the name is absent |
| `SurfaceNodeKind::FormulaDefiniens` | none | `SyntaxKind::FormulaDefiniens` | parser task-23 formula definiens; owns either one `FormulaExpression` or source-ordered `FormulaCase` children separated by comma tokens plus optional `otherwise FormulaExpression` |
| `SurfaceNodeKind::FormulaCase` | none | `SyntaxKind::FormulaCase` | parser task-23 conditional formula definiens case; owns the value `FormulaExpression`, `if`, and the condition `FormulaExpression`, with `MissingFormula` recovery for absent value or condition formulas |
| `SurfaceNodeKind::CorrectnessCondition` | none | `SyntaxKind::CorrectnessCondition` | parser task-23 correctness condition; owns the condition keyword, optional general justification (`by`, `by computation`, or `proof`), optional recovery, and optional semicolon |
| `SurfaceNodeKind::PredicateDefinition` | none | `SyntaxKind::PredicateDefinition` | parser task-24 `pred` definition; owns `pred`, a label identifier or `MissingTerm`, `:`, a raw `PredicatePattern`, `means`, a `FormulaDefiniens`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::PredicatePattern` | none | `SyntaxKind::PredicatePattern` | parser task-24 predicate definition pattern; owns source-ordered raw pattern tokens accepted by `pred_pattern` (`loci`, one `def_predicate_symbol`, optional `template_loci`, optional trailing `loci`) plus `MissingTerm` recovery when no grammar-shaped split exists; it does not encode which identifier is the predicate symbol |
| `SurfaceNodeKind::FunctorDefinition` | none | `SyntaxKind::FunctorDefinition` | parser task-25 `func` definition; owns `func`, a label identifier or `MissingTerm`, `:`, a raw `FunctorPattern`, `->`, a return `TypeExpression` or `MissingTypeExpression`, `means` plus `FormulaDefiniens` or `equals` plus `TermDefiniens`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::FunctorPattern` | none | `SyntaxKind::FunctorPattern` | parser task-25 functor definition pattern; owns source-ordered raw pattern tokens accepted by canonical single-symbol `func_pattern` or by the documented two-symbol circumfix surface shape, plus `MissingTerm` recovery when no grammar-shaped split exists; it does not encode which token is a functor symbol |
| `SurfaceNodeKind::TermDefiniens` | none | `SyntaxKind::TermDefiniens` | parser task-25 term definiens; owns either one `TermExpression` or source-ordered `TermCase` children separated by comma tokens plus optional `otherwise TermExpression` |
| `SurfaceNodeKind::TermCase` | none | `SyntaxKind::TermCase` | parser task-25 conditional term definiens case; owns the value `TermExpression`, `if`, and the condition `FormulaExpression`, with `MissingTerm` or `MissingFormula` recovery for absent value or condition |
| `SurfaceNodeKind::ModeDefinition` | none | `SyntaxKind::ModeDefinition` | parser task-26 `mode` definition; owns `mode`, a label identifier or `MissingTerm`, `:`, a raw `ModePattern`, `is`, a body `TypeExpression` or `MissingTypeExpression`, the first semicolon when present, and optional `ModeProperty` |
| `SurfaceNodeKind::ModePattern` | none | `SyntaxKind::ModePattern` | parser task-26 mode definition pattern; owns source-ordered raw tokens accepted by `mode_def_name [ type_params ]` plus `MissingTerm` recovery when no grammar-shaped split exists; it does not encode semantic parameter roles |
| `SurfaceNodeKind::ModeProperty` | none | `SyntaxKind::ModeProperty` | parser task-26 `sethood` property immediately following a mode definition; owns `sethood`, a required general justification (`JustificationClause` or `ProofBlock`) when present, optional recovery, and the property semicolon when present |
| `SurfaceNodeKind::AttributeRedefinition` | none | `SyntaxKind::AttributeRedefinition` | parser task-27 `redefine attr`; owns `redefine`, `attr`, a label identifier or `MissingTerm`, `:`, a subject identifier or `MissingTerm`, `is`, an `AttributePattern`, `means`, a `FormulaDefiniens`, the first semicolon when present, and a mandatory `CoherenceCondition` |
| `SurfaceNodeKind::PredicateRedefinition` | none | `SyntaxKind::PredicateRedefinition` | parser task-27 `redefine pred`; owns `redefine`, `pred`, a raw `PredicatePattern`, `means`, a `FormulaDefiniens`, the first semicolon when present, and a mandatory `CoherenceCondition` |
| `SurfaceNodeKind::FunctorRedefinition` | none | `SyntaxKind::FunctorRedefinition` | parser task-27 `redefine func`; owns `redefine`, `func`, a label identifier or `MissingTerm`, `:`, a raw `FunctorPattern`, `->`, a return `TypeExpression` or `MissingTypeExpression`, `means FormulaDefiniens` or `equals TermDefiniens`, the first semicolon when present, and a mandatory `CoherenceCondition` |
| `SurfaceNodeKind::CoherenceCondition` | none | `SyntaxKind::CoherenceCondition` | parser task-27 redefinition coherence tail; owns `coherence`, optional `with` plus a label identifier or `MissingProofStep`, a required general justification when present, optional recovery, and the coherence semicolon when present |
| `SurfaceNodeKind::NotationAlias` | none | `SyntaxKind::NotationAlias` | parser task-27 `synonym` or `antonym` declaration; owns the alias keyword, an alternate `NotationPattern`, `for`, an original `NotationPattern`, optional recovery, and the final semicolon when present |
| `SurfaceNodeKind::NotationPattern` | none | `SyntaxKind::NotationPattern` | parser task-27 raw notation alias pattern; owns source-ordered raw tokens from one side of `for` plus `MissingTerm` recovery when the side is empty or cannot be delimited; it does not classify the pattern as predicate, functor, mode, or attribute |
| `SurfaceNodeKind::PropertyClause` | none | `SyntaxKind::PropertyClause` | parser task-28 property item; owns a canonical predicate/functor property keyword or standalone `sethood`, a required general justification (`JustificationClause` or `ProofBlock`) when present, optional recovery, and the property semicolon when present |
| `SurfaceNodeKind::StructureDefinition` | none | `SyntaxKind::StructureDefinition` | parser task-29 `struct` definition; owns `struct`, a raw `StructurePattern`, `where`, one or more `StructureField` / `StructureProperty` members, `end`, and the final semicolon when present |
| `SurfaceNodeKind::StructurePattern` | none | `SyntaxKind::StructurePattern` | parser task-29 structure definition pattern; owns source-ordered raw tokens for `struct_def_name [ type_params ]` plus `MissingTerm` recovery when the pattern is empty or malformed; it does not encode semantic structure identity |
| `SurfaceNodeKind::StructureField` | none | `SyntaxKind::StructureField` | parser task-29 structure field member; owns `field`, a field identifier or `MissingTerm`, `->`, a `TypeExpression` or `MissingTypeExpression`, optional `:= TermExpression`, and the member semicolon when present |
| `SurfaceNodeKind::StructureProperty` | none | `SyntaxKind::StructureProperty` | parser task-29 structure property member; owns `property`, a property identifier or `MissingTerm`, `->`, a `TypeExpression` or `MissingTypeExpression`, and the member semicolon when present |
| `SurfaceNodeKind::InheritanceDefinition` | none | `SyntaxKind::InheritanceDefinition` | parser task-29 inheritance definition; owns `inherit`, child and parent `InheritanceTarget` nodes around `extends`, optional explicit `where ... end` member block, and the final semicolon when present |
| `SurfaceNodeKind::InheritanceTarget` | none | `SyntaxKind::InheritanceTarget` | parser task-29 raw inheritance child/parent target; preserves source-order tokens for a structure-like reference plus optional raw type arguments, or the parent `set` token, without resolving semantic structure identity |
| `SurfaceNodeKind::FieldRedefinition` | none | `SyntaxKind::FieldRedefinition` | parser task-29 explicit inheritance field mapping; owns `field`, a child field identifier or `MissingTerm`, optional narrowed `-> TypeExpression`, required `from`, an identifier or `it` source when present, and the member semicolon when present |
| `SurfaceNodeKind::PropertyRedefinition` | none | `SyntaxKind::PropertyRedefinition` | parser task-29 explicit inheritance property mapping; owns `property`, a child property identifier or `MissingTerm`, optional narrowed `-> TypeExpression`, required `from`, an identifier source when present, and the member semicolon when present |
| `SurfaceNodeKind::RegistrationBlockItem` | none | `SyntaxKind::RegistrationBlockItem` | parser task-30 `registration ... end;` block item; owns `registration`, source-ordered registration content, optional recovery, `MissingEnd` or `end` when present, and the final semicolon when present |
| `SurfaceNodeKind::RegistrationParameter` | none | `SyntaxKind::RegistrationParameter` | parser task-30 registration-local `let` parameter; owns qualified variable segments, a `TypeExpression` or recovery, optional condition list and syntax-level `by` references, and the parameter semicolon when present |
| `SurfaceNodeKind::ExistentialRegistration` | none | `SyntaxKind::ExistentialRegistration` | parser task-30 existential cluster registration; owns `cluster`, label, colon, attributed `TypeExpression`, header semicolon, and an `existence` correctness condition |
| `SurfaceNodeKind::ConditionalRegistration` | none | `SyntaxKind::ConditionalRegistration` | parser task-30 conditional cluster registration; owns antecedent registration adjectives, `->`, consequent registration adjectives, `for`, target `TypeExpression`, header semicolon, and a `coherence` correctness condition |
| `SurfaceNodeKind::FunctorialRegistration` | none | `SyntaxKind::FunctorialRegistration` | parser task-30 functorial cluster registration; owns an unambiguous application/operator/bracket payload term, `->`, consequent registration adjectives, `for`, target `TypeExpression`, header semicolon, and a `coherence` correctness condition |
| `SurfaceNodeKind::ReductionRegistration` | none | `SyntaxKind::ReductionRegistration` | parser task-30 reduction registration; owns `reduce`, label, colon, left `TermExpression`, `to`, right `TermExpression`, header semicolon, and a `reducibility` correctness condition |
| `SurfaceNodeKind::TemplateParameter` | none | `SyntaxKind::TemplateParameter` | parser task-31 leading template-block `let` parameter; owns ordinary value/type/predicate/functor parameter tokens, optional constraints or `by`/proof tails, and the parameter semicolon when present |
| `SurfaceNodeKind::TemplateLoci` | none | `SyntaxKind::TemplateLoci` | parser task-31 predicate/functor pattern-side `[` locus list `]`; owns delimiters, comma tokens, and `TemplateLocus` children or missing-locus recovery |
| `SurfaceNodeKind::TemplateLocus` | none | `SyntaxKind::TemplateLocus` | parser task-31 single pattern-side template locus identifier or missing-term recovery |
| `SurfaceNodeKind::TemplateArguments` | none | `SyntaxKind::TemplateArguments` | parser task-31 call/reference-side `[` template argument list `]`; owns delimiters, comma tokens, and `TemplateArgument` children or recovery |
| `SurfaceNodeKind::TemplateArgument` | none | `SyntaxKind::TemplateArgument` | parser task-31 single template actual, wrapping a `TypeExpression`, `TermExpression` / `QuaExpression`, or missing-type recovery |
| `SurfaceNodeKind::AlgorithmDefinition` | none | `SyntaxKind::AlgorithmDefinition` | parser task-32 definition content for `algorithm name [template_loci] (parameters) [-> type_expression] do ... end;`, extended by task 34 to accept an optional leading `AlgorithmTerminationClause` and ordered header verification clauses before `AlgorithmBody` |
| `SurfaceNodeKind::AlgorithmParameters` | none | `SyntaxKind::AlgorithmParameters` | parser task-32 algorithm formal list; owns `(`, comma-separated identifier tokens or missing-term recovery, and `)` / delimiter recovery |
| `SurfaceNodeKind::AlgorithmBody` | none | `SyntaxKind::AlgorithmBody` | parser task-32 `do ... end` algorithm body; owns the `do` token when present, an `AlgorithmStatementList`, and `end` / missing-end recovery |
| `SurfaceNodeKind::AlgorithmStatementList` | none | `SyntaxKind::AlgorithmStatementList` | parser task-32 source-ordered body statements, extended by task 33 to include control-flow statements and by task 34 to include `AssertStatement`; annotations remain parser task 35 |
| `SurfaceNodeKind::VariableDeclaration` | none | `SyntaxKind::VariableDeclaration` | parser task-32 `var` / `const` declaration, optionally prefixed by `ghost`; owns one or more `VariableBinding` children, optional shared `as TypeExpression`, optional syntax-level justification, and the semicolon or recovery |
| `SurfaceNodeKind::VariableBinding` | none | `SyntaxKind::VariableBinding` | parser task-32 declaration binding; owns a binding identifier plus optional `:= TermExpression` initializer or missing-term recovery |
| `SurfaceNodeKind::AssignmentStatement` | none | `SyntaxKind::AssignmentStatement` | parser task-32 assignment statement, optionally prefixed by `ghost`; owns an `Lvalue`, `:=`, assigned `TermExpression` or recovery, and the semicolon or recovery |
| `SurfaceNodeKind::Lvalue` | none | `SyntaxKind::Lvalue` | parser task-32 syntactic assignment target; owns an identifier and optional dotted identifier segments without resolving selector versus namespace roles |
| `SurfaceNodeKind::SnapshotStatement` | none | `SyntaxKind::SnapshotStatement` | parser task-32 `snapshot` statement; owns the snapshot identifier and semicolon or recovery |
| `SurfaceNodeKind::ReturnStatement` | none | `SyntaxKind::ReturnStatement` | parser task-32 `return` statement; owns optional returned `TermExpression`, optional syntax-level justification, and semicolon or recovery |
| `SurfaceNodeKind::ClaimBlockItem` | none | `SyntaxKind::ClaimBlockItem` | parser task-32 top-level `claim algorithm_name do ... end;` item; owns bare theorem/lemma children and defers claim-local annotations to task 35 |
| `SurfaceNodeKind::IfStatement` | none | `SyntaxKind::IfStatement` | parser task-33 algorithm conditional; owns `if`, a `FormulaExpression` or missing-formula recovery, `do`, the then `AlgorithmStatementList`, and either `end;`, `else` plus nested `IfStatement`, or `else` plus an else `AlgorithmStatementList` and `end;` |
| `SurfaceNodeKind::WhileStatement` | none | `SyntaxKind::WhileStatement` | parser task-33 algorithm while loop, extended by task 34 to own leading `LoopInvariantClause` and `LoopDecreasingClause` children between `do` and the body list |
| `SurfaceNodeKind::ForRangeStatement` | none | `SyntaxKind::ForRangeStatement` | parser task-33 algorithm range loop, extended by task 34 to own leading `LoopInvariantClause` children between `do` and the body list; `decreasing` is still rejected for `for` loops |
| `SurfaceNodeKind::ForCollectionStatement` | none | `SyntaxKind::ForCollectionStatement` | parser task-33 algorithm collection loop, extended by task 34 to own leading `LoopInvariantClause` children between `do` and the body list; `decreasing` is still rejected for `for` loops |
| `SurfaceNodeKind::MatchStatement` | none | `SyntaxKind::MatchStatement` | parser task-33 algorithm structural match; owns `match`, scrutinee `TermExpression`, `do`, one or more `MatchCase` nodes or missing-statement recovery, a `MatchEnding`, and final `end;` / recovery |
| `SurfaceNodeKind::MatchCase` | none | `SyntaxKind::MatchCase` | parser task-33 `case term_pattern do ... end;` branch; owns the pattern as `TermExpression`, branch statement list, and branch `end;` / recovery |
| `SurfaceNodeKind::MatchEnding` | none | `SyntaxKind::MatchEnding` | parser task-33 match ending; owns either `otherwise` plus a statement list and `end;`, or `exhaustive` plus optional syntax-level justification and semicolon |
| `SurfaceNodeKind::BreakStatement` | none | `SyntaxKind::BreakStatement` | parser task-33 `break;` jump statement; loop-context validity is semantic and not encoded here |
| `SurfaceNodeKind::ContinueStatement` | none | `SyntaxKind::ContinueStatement` | parser task-33 `continue;` jump statement; loop-context and termination obligations are semantic and not encoded here |
| `SurfaceNodeKind::AlgorithmTerminationClause` | none | `SyntaxKind::AlgorithmTerminationClause` | parser task-34 `terminating` modifier before `algorithm`; termination proof obligations are semantic and not encoded here |
| `SurfaceNodeKind::AlgorithmRequiresClause` | none | `SyntaxKind::AlgorithmRequiresClause` | parser task-34 header `requires FormulaExpression` clause |
| `SurfaceNodeKind::AlgorithmEnsuresClause` | none | `SyntaxKind::AlgorithmEnsuresClause` | parser task-34 header `ensures FormulaExpression` clause |
| `SurfaceNodeKind::AlgorithmDecreasingClause` | none | `SyntaxKind::AlgorithmDecreasingClause` | parser task-34 header `decreasing TermList` clause |
| `SurfaceNodeKind::LoopInvariantClause` | none | `SyntaxKind::LoopInvariantClause` | parser task-34 leading loop `invariant FormulaExpression [JustificationClause];` clause |
| `SurfaceNodeKind::LoopDecreasingClause` | none | `SyntaxKind::LoopDecreasingClause` | parser task-34 leading while-loop `decreasing TermList [JustificationClause];` clause |
| `SurfaceNodeKind::AssertStatement` | none | `SyntaxKind::AssertStatement` | parser task-34 algorithm assertion statement; owns `assert`, a `FormulaExpression` or missing-formula recovery, optional syntax-level justification, and semicolon/recovery |
| `SurfaceNodeKind::TermList` | none | `SyntaxKind::TermList` | parser task-34 comma-separated list of one or more decreasing-measure `TermExpression` children with comma tokens and missing-term recovery for empty slots |
| `SurfaceNodeKind::CompactStatement` | none | `SyntaxKind::CompactStatement` | parser task-17 minimal explicit-justification compact statement host plus parser task-22 proof justification host; owns one `Proposition`, one `JustificationClause` or `ProofBlock`, optional recovery, and optional semicolon |
| `SurfaceNodeKind::JustificationClause` | none | `SyntaxKind::JustificationClause` | parser task-17 `by` clause; owns the `by` token plus either `ReferenceList` for ordinary citations or `ComputationJustification` for `by computation(...)` |
| `SurfaceNodeKind::ReferenceList` | none | `SyntaxKind::ReferenceList` | parser task-17 source-ordered citation list; owns citation nodes separated by comma tokens |
| `SurfaceNodeKind::Reference` | none | `SyntaxKind::Reference` | parser task-17 local citation; owns one identifier token and optional task-31 `TemplateArguments` |
| `SurfaceNodeKind::QualifiedReference` | none | `SyntaxKind::QualifiedReference` | parser task-17 namespace-qualified citation; owns `NamespacePath`, the final dot token, final identifier token, and optional task-31 `TemplateArguments` |
| `SurfaceNodeKind::GroupedReference` | none | `SyntaxKind::GroupedReference` | parser task-17 grouped citation; owns `NamespacePath`, `.{`, grouped items separated by comma tokens, optional delimiter recovery, and optional `}` |
| `SurfaceNodeKind::GroupedReferenceItem` | none | `SyntaxKind::GroupedReferenceItem` | parser task-17 grouped citation member; owns one identifier token and optional task-31 `TemplateArguments` |
| `SurfaceNodeKind::BulkReference` | none | `SyntaxKind::BulkReference` | parser task-17 bulk citation; owns `NamespacePath` plus the compound `.*` token |
| `SurfaceNodeKind::ComputationJustification` | none | `SyntaxKind::ComputationJustification` | parser task-17 computation proof payload; owns the `computation` token and optional parenthesized computation-option list |
| `SurfaceNodeKind::ComputationOption` | none | `SyntaxKind::ComputationOption` | parser task-17 computation option; owns `steps`, `timeout`, or `nest`, a colon token, and a numeral token or `MissingProofStep` recovery |
| `SurfaceNodeKind::SelectorAccess` | none | `SyntaxKind::SelectorAccess` | parser task-10 postfix selector access or selector-call surface; preserves syntax-only dot role |
| `SurfaceNodeKind::StructureUpdate` | none | `SyntaxKind::StructureUpdate` | parser task-10 functional `term "with" "(" field_update_list ")"` update surface |
| `SurfaceNodeKind::FieldUpdate` | none | `SyntaxKind::FieldUpdate` | parser task-10 `selector ":=" term_expression` field update inside `StructureUpdate` |
| `SurfaceNodeKind::QuaExpression` | none | `SyntaxKind::QuaExpression` | parser task-11 type qualification; child order is base term-shape, `qua` token, and a `TypeExpression` or `MissingTypeExpression` recovery |
| `SurfaceNodeKind::ModulePath` | none | `SyntaxKind::ModulePath` | `module_path`; optional `RelativePrefix`, first `PathSegment`, then repeated `.` token plus `PathSegment`; only this path shape may contain `RelativePrefix` |
| `SurfaceNodeKind::NamespacePath` | none | `SyntaxKind::NamespacePath` | `namespace_path`; first `PathSegment`, then repeated `.` token plus identifier `PathSegment`; relative prefixes are not allowed |
| `SurfaceNodeKind::QualifiedSymbol` | none | `SyntaxKind::QualifiedSymbol` | `qualified_symbol`; zero or more identifier namespace `PathSegment` + `.` token pairs followed by a final user-symbol `PathSegment`, or the task-8 attribute-ref flattening where dotted prefix `PathSegment`s may also be user-symbol tokens before the final user-symbol |
| `SurfaceNodeKind::PathSegment` | none | `SyntaxKind::PathSegment` | wraps exactly one identifier or user-symbol token; role is determined by parent and token kind |
| `SurfaceNodeKind::RelativePrefix` | none | `SyntaxKind::RelativePrefix` | wraps exactly one `.` or `..` token at the start of a `ModulePath` |
| `SurfaceNodeKind::InfixExpression(SurfaceInfixOperator)` | spelling, precedence, associativity | `SyntaxKind::InfixExpression` | task-12 infix Pratt expression shape |
| `SurfaceNodeKind::PrefixExpression(SurfacePrefixOperator)` | spelling, precedence | `SyntaxKind::PrefixExpression` | task-12 prefix Pratt expression shape |
| `SurfaceNodeKind::PostfixExpression(SurfacePostfixOperator)` | spelling, precedence | `SyntaxKind::PostfixExpression` | task-12 postfix Pratt expression shape |
| `SurfaceNodeKind::FormulaExpression` | none | `SyntaxKind::FormulaExpression` | parser task-13/14 formula wrapper; owns exactly one formula child, including atomic formulas, connectives, quantifiers, parenthesized formulas, and formula constants |
| `SurfaceNodeKind::BuiltinPredicateApplication` | none | `SyntaxKind::BuiltinPredicateApplication` | parser task-13 built-in `in`, `=`, or `<>` predicate; owns left term, predicate token, and right term or missing-term recovery |
| `SurfaceNodeKind::IsAssertion` | none | `SyntaxKind::IsAssertion` | parser task-13 generic `is` assertion; owns subject term, `is`, optional `not`, and a type/body child without resolver classification |
| `SurfaceNodeKind::AttributeTestChain` | none | `SyntaxKind::AttributeTestChain` | parser task-13 attribute-only assertion body; owns one or more task-8 `AttributeRef` children |
| `SurfaceNodeKind::PredicateApplication` | none | `SyntaxKind::PredicateApplication` | parser task-13 syntax-only user predicate application; owns one or more predicate segments |
| `SurfaceNodeKind::PredicateSegment` | none | `SyntaxKind::PredicateSegment` | parser task-13 user predicate segment; owns optional term-list children, optional negation tokens, one predicate head, and optional right term-list children |
| `SurfaceNodeKind::PredicateHead` | none | `SyntaxKind::PredicateHead` | parser task-13 predicate symbol wrapper; owns an active `QualifiedSymbol` or template-local identifier plus optional task-31 `TemplateArguments` |
| `SurfaceNodeKind::InlinePredicateApplication` | none | `SyntaxKind::InlinePredicateApplication` | parser task-13 inline predicate call shape with identifier head and parenthesized term arguments |
| `SurfaceNodeKind::PrefixFormula(SurfaceFormulaPrefixOperator)` | operator | `SyntaxKind::PrefixFormula` | parser task-14 fixed formula prefix, currently `not` |
| `SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator)` | `connective: SurfaceFormulaConnective`, `repeated: bool` | `SyntaxKind::BinaryFormula` | parser task-14 fixed binary connective formula for `&`, `or`, `implies`, `iff`, including token-preserving repetition forms |
| `SurfaceNodeKind::ParenthesizedFormula` | none | `SyntaxKind::ParenthesizedFormula` | parser task-14 formula grouping; owns `(`, one nested `FormulaExpression`, and `)` or delimiter recovery |
| `SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind)` | quantifier | `SyntaxKind::QuantifiedFormula` | parser task-14 universal/existential quantifier surface; owns quantifier token, variable segments, optional condition/body separators, and formula body children |
| `SurfaceNodeKind::QuantifierVariableSegment` | none | `SyntaxKind::QuantifierVariableSegment` | parser task-14 quantified variable segment; owns variable identifiers/comma tokens, optional `be`/`being`, and optional `TypeExpression` |
| `SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant)` | constant | `SyntaxKind::FormulaConstant` | parser task-14 `thesis` or `contradiction` formula constant |
| `SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind)` | recovery kind | `SyntaxKind::ErrorRecovery` | builder-created recovery nodes are recovered |

`SurfaceTokenKind` currently maps to the token raw kinds listed above:
`Identifier`, `ReservedWord`, `ReservedSymbol`, `Numeral`, `LexemeRun`,
`UserSymbol`, `StringLiteral`, `ErrorRecovery`, and `Unknown`.
`SurfaceOperatorAssociativity` currently has `Left`, `Right`, and
`NonAssociative`.

Shared path nodes added for `mizar-parser` task 4 are syntax-only shapes. Their
node ranges run from the first token owned by the path or wrapper through the
last token owned by that node. Parent path nodes list children in source order.
Separator `.` tokens between path segments are direct children of the parent
path node rather than wrapped as `PathSegment`. These nodes do not produce
recovery nodes or trivia entries by themselves; consuming grammar tasks own
missing-path diagnostics, skipped-token trivia, and doc-comment attachment.
`SurfaceNodeView` exposes typed `as_module_path`, `as_namespace_path`,
`as_qualified_symbol`, `as_path_segment`, and `as_relative_prefix` helpers so
consumers do not need raw rowan traversal for these shared path shapes.

Module skeleton nodes added for `mizar-parser` task 5 are syntax-only shapes.
`CompilationUnit` represents the source file surface and owns exactly one
`ItemList` child. `ItemList` children are source-ordered concrete item nodes,
`PlaceholderItem` nodes, and item-level recovery nodes such as `SkippedToken`.
`PlaceholderItem` wraps the source tokens consumed for one top-level item
boundary, including annotation prefixes and recovered items that are missing
their terminating semicolon. The parser must not encode import resolution,
visibility semantics, theorem validity, or symbol identity in these nodes.
`SurfaceNodeView` exposes typed `as_compilation_unit`, `as_item_list`, and
`as_placeholder_item` helpers.
Leading doc-comment attachment to the following item is represented through
`SurfaceTrivia`, not by copying comment text into item nodes.

Import nodes added for `mizar-parser` task 6 are syntax-only shapes.
`ImportItem` represents one `import_stmt` while the import prelude is open. In
well-formed input, its children are source ordered: the `import` token, one or
more `ImportAliasDecl` or `ModuleBranchImport` nodes separated by comma tokens,
and a semicolon token. Malformed recovery may leave an `ImportItem` with no
declaration after `import`, a trailing comma without a following declaration,
or a `SkippedToken` recovery child for malformed source consumed before the
semicolon. `ImportAliasDecl` owns the imported `ModulePath`, an optional `as`
token, and an optional alias `PathSegment`; malformed aliases may omit the
alias segment and may contain a nested `SkippedToken` recovery while carrying a
`MalformedImport` diagnostic. `ModuleBranchImport` owns the base `ModulePath`,
the `.{` token, branch `PathSegment` children separated by comma tokens, and
`}` in well-formed input; malformed branch imports may omit branch segments or
the close token and may contain a nested `SkippedToken` recovery while carrying
`MalformedImport`. These nodes may contain relative `ModulePath` prefixes, but
they do not resolve modules, split branch imports into semantic imports, check
export availability, or assign aliases. `SurfaceNodeView` exposes typed
`as_import_item`, `as_import_alias_decl`, and `as_module_branch_import` helpers.

Export and visibility nodes added for `mizar-parser` task 7 are syntax-only
shapes. `ExportItem` represents one `export_stmt` while the export prelude is
open. In well-formed input, its children are source ordered: the `export` token,
one or more `ModulePath` nodes separated by comma tokens, and a semicolon token.
Malformed recovery may leave an `ExportItem` with no path after `export`, a
trailing comma without a following path, or a nested `SkippedToken` recovery
child for malformed source consumed before the semicolon. `VisibilityMarker`
wraps exactly one `private` or `public` token. `VisibleItem` represents a
top-level visibility prefix on the theorem/notation forms allowed by Chapter
12. Its children are source ordered: annotation-prefix token nodes when
present, one `VisibilityMarker`, and the target item node. Represented theorem
and lemma targets use concrete `TheoremItem` / `LemmaItem` nodes; notation
targets and short legacy theorem fragments remain `PlaceholderItem` targets;
task 31 parses theorem payloads with template predicate arguments as represented
formula hosts when the surrounding theorem shape is otherwise supported.
Duplicate visibility markers, dangling markers, or visibility before a
non-theorem/non-notation top-level declaration may instead contain a nested
`SkippedToken` recovery child and an optional semicolon token while carrying
`MalformedVisibility`. These nodes do not decide public/private semantics,
export availability, symbol identities, theorem validity, or notation
validity. `SurfaceNodeView` exposes typed `as_export_item`,
`as_visibility_marker`, and `as_visible_item` helpers.

Type-expression nodes added for `mizar-parser` task 8 are syntax-only shapes.
`ReserveItem` is the current frontend-reachable host for `TypeExpression` nodes;
it represents top-level `reserve_decl` only and does not implement local
statement semantics. `ReserveSegment` preserves identifier-list commas, the
`for` token, and the following type expression. `TypeExpression` preserves the
surface split into an optional non-empty `AttributeChain` and a required
`TypeHead`. Because mode/radix/attribute classification depends on the active
environment, `TypeHead` is deliberately generic: it owns either builtin
`object`/`set` or a `QualifiedSymbol` and optional `TypeArguments`; it does not
record whether the head is a mode, structure, or radix type. The parser chooses
the syntactic attribute/head boundary by keeping the rightmost available
type-head candidate as the `TypeHead`, not by semantic lookup.

`AttributeRef` owns source-ordered syntax for one attribute occurrence:
optional `non`, optional `ParameterPrefix`, one syntactic `QualifiedSymbol`,
and optional parenthesized term arguments. Struct-qualified
attribute spellings are preserved as the same `QualifiedSymbol` dotted surface;
in that attribute-ref context, prefix `PathSegment`s may wrap user-symbol
tokens as well as namespace identifiers. The AST does not decide which prefix
segment is a structure. `ParameterPrefix`
preserves only the local token split that task 8 can see before an attribute
reference: a single identifier or numeral plus `-`, or a parenthesized
identifier/numeral list plus `-`. It does not validate template-parameter scope
and does not perform the full contextual whole-spelling split for names such as
`n-dimensional`; that source drift remains owned by the future task that passes
parameter-scope facts into lexing/parsing.

`TypeArguments` owns either an `of`/`over` token followed by comma-separated
term arguments, or `[` followed by comma-separated type-template arguments and
an optional `]`. Starting in parser task 9, `of`/`over` and attribute argument
lists use concrete `TermExpression` children instead of task-8
`TermPlaceholder` children. A bracket argument that parses as a type expression
is represented as a nested `TypeExpression`. Starting in parser task 11, a
bracket argument that uses Appendix-A `qua_arg` syntax is represented as a
`TermExpression` child whose term-shape is an identifier `TermReference` or a
left-nested `QuaExpression` chain. That fallback remains narrower than the
ordinary term parser: it starts from an identifier-shaped `qua_arg`, and each
`qua` target is parsed as a radix-type-shaped `TypeExpression`. Missing bracket
closers use
`MalformedTypeExpression` plus `UnmatchedOpeningDelimiter` recovery under the
`TypeArguments` node. `TermPlaceholder` is retained only as legacy task-8
vocabulary and is not emitted by the task-11 bracket `qua_arg` parser path; it
must not encode term classification, operator facts, or name resolution.
`SurfaceNodeView` exposes typed `as_reserve_item`,
`as_reserve_segment`, `as_type_expression`, `as_attribute_chain`,
`as_attribute_ref`, `as_parameter_prefix`, `as_type_head`,
`as_type_arguments`, and `as_term_placeholder` helpers.

Primary term nodes added for `mizar-parser` task 9 are syntax-only shapes.
`TermExpression` is the current wrapper around one term-shape child. Parser
tasks 9 and 10 may place a primary term or postfix chain there; parser task 11
may place a `QuaExpression`; later operator tasks may place operator
expressions without changing the wrapper role. `TermReference` owns either one
identifier token or one shared `QualifiedSymbol`, preserving term-position
references without deciding name resolution or functor classification.
`NumeralTerm` and `ItTerm` wrap the
corresponding single token. `ParenthesizedTerm` owns delimiter tokens plus a
nested `TermExpression` or `MissingTerm` recovery. `ChoiceTerm` owns the `the`
token and a nested `TypeExpression` or `MissingTypeExpression` recovery when
the type operand is absent.

`ApplicationTerm` is deliberately generic for task 9: it preserves ordinary
parenthesized applications and the reserved `[` / `]` bracket functor form, but
does not encode arity, overload selection, or active user-symbol bracket-pair
metadata. Ordinary application child order is the callee `TermReference` or
`QualifiedSymbol`, the `(` token, zero or more `TermExpression` arguments
separated by comma tokens, then optional `)`. Reserved bracket application
child order is `[`, zero or more `TermExpression` arguments separated by comma
tokens, then optional `]`; it has no callee child because the delimiter pair is
the syntax-only head. `StructureConstructor` is emitted only when named field
arguments are visible syntactically; child order is the constructor
`QualifiedSymbol`, optional `TypeArguments`, the `(` token, `FieldArgument`
children separated by comma tokens, then optional `)`. Ambiguous zero-field
forms remain generic `ApplicationTerm` nodes until a later semantic boundary
supplies structure facts. `FieldArgument` owns a field identifier, the `:`
token, and a `TermExpression` or `MissingTerm`. `SetEnumeration` owns `{`,
source-ordered term arguments separated by comma tokens, and an optional `}`.
`SetComprehension` owns `{`, a mapper `TermExpression`, `where`, one or more
`ComprehensionVariableSegment` children separated by comma tokens, optional
`:` plus `FormulaExpression`, and an optional `}`. Missing `}` may be
represented by `UnmatchedOpeningDelimiter` recovery. A
`ComprehensionVariableSegment` owns one generator identifier or a `MissingTerm`
recovery in the identifier position, the `is` token when present, and a
`TypeExpression` or `MissingTypeExpression` recovery when the `is` token is
present. These comprehension nodes do not resolve binder identity, sethood,
capture, or the elaborated Fraenkel symbol. `SurfaceNodeView` exposes typed `as_term_expression`,
`as_term_reference`, `as_numeral_term`, `as_it_term`,
`as_parenthesized_term`, `as_choice_term`, `as_application_term`,
`as_structure_constructor`, `as_field_argument`, `as_set_enumeration`,
`as_set_comprehension`, and `as_comprehension_variable_segment` helpers.

Parser task 10 keeps the dot-role surface syntax-only. `SelectorAccess` owns
the base term-shape child, a `.` token, an identifier field token, and optional
call delimiters plus source-ordered `TermExpression` arguments separated by
comma tokens. Chained selectors nest left-associatively. `StructureUpdate`
owns the base term-shape child, the `with` token, `(`, `FieldUpdate` children
separated by comma tokens, and optional `)`. `FieldUpdate` owns an identifier
selector path (`identifier`, repeated `.` token plus identifier), the `:=`
token, and a `TermExpression` or `MissingTerm` recovery. These nodes do not
decide selector-versus-namespace roles using scope, and they do not represent
standalone statement or algorithm assignments. `SurfaceNodeView` exposes typed
`as_selector_access`, `as_structure_update`, and `as_field_update` helpers.

Parser task 11 adds `QuaExpression` for `term qua type_expression`. The parser
forms selector/update postfix chains before `QuaExpression`, so `p.x qua T`
qualifies the selector result. Because `qua` has the lowest current term
precedence, a selector after a qualified term requires parentheses:
`(p qua T).x`. Chained `qua` expressions nest left-associatively:
`x qua T qua U` is represented as a `QuaExpression` whose base is the
`QuaExpression` for `x qua T`. A `qua` target is a `TypeExpression`; if that
target type contains term arguments, any `qua` inside those arguments belongs
to the argument term before an outer chain can continue. Thus
`x qua Element of S qua Magma` is represented as `x qua Element of (S qua
Magma)`, while `(x qua Element of S) qua Magma` qualifies the outer result.
Missing target types insert `MissingTypeExpression` under `QuaExpression` and
emit `MalformedTypeExpression`; malformed target tails may use type-tail
`SkippedToken` recovery before the surrounding term boundary. `SurfaceNodeView`
exposes `as_qua_expression` for this node. Static validity, type narrowing or
widening, overload selection, and proof obligations remain resolver/checker
responsibilities.

Parser task 12 adds active-lexicon operator expression nodes. `PrefixExpression`
owns the operator token followed by the operand term-shape child and stores the
operator spelling and precedence supplied by parser inputs. `PostfixExpression`
owns the base term-shape child followed by the operator token and stores the
same spelling/precedence payload. `InfixExpression` keeps the existing
`left`, operator token, `right` child order and additionally stores infix
associativity. Selector/update postfix chains and ordinary application are
formed inside Pratt operands and therefore bind tighter than these user
operators; `qua` is formed after Pratt and therefore remains the lowest
term-level operator. Non-associative errors are syntax diagnostics only.
Dangling infix operators may remain diagnostic-only, while dangling prefix
operators keep the represented `PrefixExpression` recoverable by inserting a
`MissingTerm` operand. `SurfaceNodeView` exposes `as_prefix_expression`,
`as_postfix_expression`, and `as_infix_expression` payload accessors. Operator
metadata is parser input, not semantic resolution: these nodes must not carry
symbol ids, selected overloads, inferred types, or proof facts.

Parser tasks 13-14 define the current formula nodes. `FormulaExpression` wraps
one formula child, whether that child is atomic, connective-bearing,
quantified, parenthesized, `thesis`, or `contradiction`, without changing the
wrapper role. Task 13 first exposed formula payloads through theorem/lemma
placeholder hosts; task 22 promotes represented theorem declarations to
concrete `TheoremItem` and `LemmaItem` hosts that own the optional status token,
role token, label, colon, `FormulaExpression`, optional justification or
`ProofBlock`, and enclosing semicolon. Task 31 makes template predicate
arguments concrete syntax for represented theorem/lemma payloads, so those
payloads no longer require a placeholder solely because the predicate head owns
template arguments.

`BuiltinPredicateApplication` owns a left `TermExpression`, the built-in
predicate token (`in`, `=`, or `<>`), and a right `TermExpression` or
`MissingTerm` recovery. `IsAssertion` owns a subject `TermExpression`, the `is`
token, an optional formula-level `not` token, and either a `TypeExpression` or
`AttributeTestChain` body. The node is deliberately generic: it does not decide
whether the body is semantically a type assertion or an attribute assertion.
`AttributeTestChain` owns one or more task-8 `AttributeRef` nodes and exists
for attribute-only assertion bodies such as `non empty` that have no trailing
type head.

`PredicateApplication` owns source-ordered `PredicateSegment` children for
syntax-only user predicate applications and chains. Each `PredicateSegment`
may own left term operands, optional `does not` / `do not` negation tokens, one
`PredicateHead`, and right term operands. `PredicateHead` wraps the predicate
symbol token, qualified symbol, or template-local identifier, and may own
optional task-31 `TemplateArguments`. Built-in predicates are represented only by a single
`BuiltinPredicateApplication` node and must not be mixed into
`PredicateApplication` chains, preserving Appendix A's `a < b = c` syntax-error
boundary. `InlinePredicateApplication` owns an identifier head, parentheses,
and source-ordered term arguments. These formula nodes preserve predicate
spelling and argument shape only; predicate overload resolution, chain
adjacency validity, theorem validity, proof facts, and truth evaluation remain
outside `mizar-syntax`.

Parser task 14 completes the current formula vocabulary. `PrefixFormula` owns
the `not` token followed by one formula child or `MissingFormula` recovery.
`BinaryFormula` owns the left formula child, the connective token, optional
`...` plus repeated connective token for `& ... &` / `or ... or`, and the
right formula child or `MissingFormula` recovery. Its payload records the
fixed connective and whether the repetition form was written; it does not
carry semantic expansion details. `ParenthesizedFormula` owns `(`, a nested
`FormulaExpression`, and `)` or delimiter recovery. `FormulaConstant` wraps the
single `thesis` or `contradiction` token and carries only that constant kind.

`QuantifiedFormula` owns the `for` or `ex` token, source-ordered
`QuantifierVariableSegment` children separated by comma tokens, optional `st`
condition formula for universal quantification, required `st` body formula for
existential quantification, and either a `holds` body formula or nested
quantified-formula body for universal quantification. `QuantifierVariableSegment`
owns the written variable identifiers and commas, an optional `be` / `being`
token, and an optional `TypeExpression`. It does not resolve implicit variable
types from `reserve`, does not classify bound variables semantically, and does
not create proof obligations.

`SurfaceNodeView` exposes `as_prefix_formula`, `as_binary_formula`,
`as_parenthesized_formula`, `as_quantified_formula`,
`as_quantifier_variable_segment`, and `as_formula_constant` helpers. Consumers
inspect fixed formula payloads through `SurfaceNodeKind`: `PrefixFormula`
carries `SurfaceFormulaPrefixOperator`, `BinaryFormula` carries
`SurfaceFormulaBinaryOperator { connective: SurfaceFormulaConnective,
repeated: bool }`, `QuantifiedFormula` carries `SurfaceQuantifierKind`, and
`FormulaConstant` carries `SurfaceFormulaConstant`. Ranges for all task-14
formula nodes run from the first owned source token through the last owned
source token; inserted `MissingFormula` or `MissingTypeExpression` recovery is
zero-width at the insertion point and must be contained by the parent range
unless it is used only as out-of-range recovery context under the general
recovery exception. Parenthesized formulas with a missing `)` range through
the nested formula or insertion point; quantified formulas range through the
condition/body formula or recovery insertion that completed the represented
quantifier.

Parser task 16 starts S-013 statement vocabulary with simple statement nodes.
`StatementItem` remains the module-level wrapper used when a concrete statement
appears at top level in the parse-only corpus. It owns exactly one concrete
parser-owned statement node from the currently implemented statement
vocabulary, including later S-013 / S-014 increments such as compact,
consider/reconsider, conclusion, `then`, and iterative-equality statements.
Task 22 also lets `ProofBlock`, `NowStatement`, `HerebyStatement`, and case
branch bodies own the same concrete statement nodes directly through reasoning
bodies. Statement-level annotations are deferred to task 35 / S-016, so
`StatementItem` does not own annotation-prefix tokens. `reserve` remains the
top-level task-8 `ReserveItem` only because Chapter 4 forbids block-local
`reserve`-shaped statements.

`LetStatement` owns `let`, one or more `QualifiedVariableSegment` children
separated by comma tokens, optional `such` plus `ConditionList`, and `;` when
present. `GivenStatement` has the same qualified-variable and optional
condition shape after `given`. `QualifiedVariableSegment` owns the written
identifier tokens and internal commas, optional `be` / `being`, and optional
`TypeExpression` or `MissingTypeExpression` recovery. It does not resolve
implicit types from `reserve`.

`AssumptionStatement` owns `assume` plus either a single `Proposition` or a
`ConditionList`. `ConditionList` owns `that`, one or more `Proposition`
children separated by statement-level `and` tokens, and optional recovery.
`Proposition` owns an optional label identifier plus colon and one
`FormulaExpression` or `MissingFormula` recovery. `TakeStatement` owns `take`
and source-ordered `Witness` children separated by comma tokens. A `Witness`
owns either one `TermExpression` or a named witness spelling
`identifier "=" TermExpression`; missing witness terms use `MissingTerm`.
`SetStatement` owns `set` and source-ordered `Equating` children separated by
comma tokens. `Equating` owns an identifier or `MissingTerm` recovery, `=`
when present, and a right-hand `TermExpression` or `MissingTerm`.

Task 16 deliberately excludes task-17 justification nodes. A `let` statement
with a top-level `by` tail before its semicolon remains a legacy placeholder
instead of partially parsing into `LetStatement`. These statement nodes do not
validate label uniqueness, references, type well-formedness, witness leakage,
or proof obligations. `SurfaceNodeView` exposes typed `as_statement_item`,
`as_let_statement`, `as_qualified_variable_segment`, `as_assumption_statement`,
`as_proposition`, `as_condition_list`, `as_given_statement`,
`as_take_statement`, `as_witness`, `as_set_statement`, and `as_equating`
helpers.

Parser task 17 starts S-014 justification vocabulary and adds a minimal
explicit-justification compact statement host. `CompactStatement` owns one
`Proposition`, one `JustificationClause`, optional recovery, and the semicolon
token when present. It exists so the shared justification surface can be
exercised before the later statement tasks complete conclusion and equality
dispatch; compact statements without an explicit `by` tail remain later
statement work. `LetStatement` may now own a trailing `JustificationClause`,
but only in the ordinary `by references` shape defined by Chapter 15.

`JustificationClause` owns the leading `by` token plus either a `ReferenceList`
child for ordinary citations or a `ComputationJustification` child for
`by computation(...)`. Task 31 extends the task-17 citation surface so local
references, qualified references, and grouped reference members may own
`TemplateArguments`. `from` is not a justification node because the canonical Chapter 15/16
grammar does not define it as a justification form.

`ReferenceList` owns source-ordered citation children separated by comma
tokens. A local citation is `Reference` with one identifier token. A
`QualifiedReference` owns a `NamespacePath`, the final dot token, and the final
identifier token. A `GroupedReference` owns a `NamespacePath`, the compound
`.{` token, one or more `GroupedReferenceItem` children separated by comma
tokens, and the closing `}` token when present. `GroupedReferenceItem` owns one
identifier token in this increment. A `BulkReference` owns a `NamespacePath`
and the compound `.*` token. `ComputationJustification` owns the `computation`
token and optional parenthesized `ComputationOption` children separated by
comma tokens. Each `ComputationOption` owns one of `steps`, `timeout`, or
`nest`, the colon token, and a numeral token.

Justification nodes preserve citation spelling only. They do not resolve
labels, expand grouped or bulk citations, validate theorem visibility, select
ATP engines, validate computation-option values, or replay computation proofs.
Ranges run from the first owned source token through the last owned source
token. Missing references, grouped items, or computation option operands use
`MissingProofStep` recovery with a zero-width insertion range under the
owning justification node. Malformed tails may own `SkippedToken` recovery and
skipped-token trivia. `SurfaceNodeView` exposes `as_compact_statement`,
`as_justification_clause`, `as_reference_list`, `as_reference`,
`as_qualified_reference`, `as_grouped_reference`,
`as_grouped_reference_item`, `as_bulk_reference`,
`as_computation_justification`, and `as_computation_option` helpers.

Parser task 18 continues S-013 statement vocabulary with the remaining
justified introduction/type-changing forms. `ConsiderStatement` owns
`consider`, one or more `QualifiedVariableSegment` children separated by comma
tokens, `such`, a `ConditionList`, a simple `JustificationClause`, optional
recovery, and the semicolon token when present. `ReconsiderStatement` owns
`reconsider`, one or more `ReconsiderItem` children separated by comma tokens,
`as`, a target `TypeExpression`, a simple `JustificationClause`, optional
recovery, and the semicolon token when present. `ReconsiderItem` owns either a
bare identifier token or the equated spelling `identifier "=" TermExpression`.
Task 18 uses only simple citation justifications on these hosts; computation
justifications remain limited to `CompactStatement` until a later specification
admits them elsewhere.

Task 18 statement nodes preserve syntax only. They do not resolve witness
existence, check whether reconsidered names are already bound, validate target
types, or generate proof obligations. Missing mandatory `by references` tails
use `MissingProofStep` recovery directly under the statement node. Missing
`consider` conditions use `MissingFormula`; missing `reconsider` item
identifiers or right-hand-side terms use `MissingTerm`; missing target types
use `MissingTypeExpression`. `SurfaceNodeView` exposes
`as_consider_statement`, `as_reconsider_statement`, and
`as_reconsider_item` helpers.
Snapshot rendering prints the literal node names.

Parser task 19 adds the conclusion and iterative-equality portion of S-013.
`ConclusionStatement` owns `thus` or `hence`, one `Proposition`, an optional
explicit `JustificationClause`, optional recovery, and the semicolon token
when present. `ThenStatement` is a syntax-only wrapper that owns the `then`
token plus exactly one linkable statement child, or `MissingStatement`
recovery when the modifier appears before a standalone/non-linkable statement.
It does not desugar `hence`, attach predecessor facts, or otherwise encode
proof semantics.

`IterativeEqualityStatement` owns an optional label identifier and colon, the
initial left `TermExpression`, `=`, initial right `TermExpression`, optional
simple citation `JustificationClause`, one or more `IterativeEqualityStep`
children, optional recovery, and the semicolon token when present.
`IterativeEqualityStep` owns `.=` plus one `TermExpression` or `MissingTerm`
and an optional simple citation `JustificationClause`. The compact/equality
dispatch boundary is syntax-only: a justified equality without a top-level
`.=` continuation remains `CompactStatement`; a chain with at least one
top-level `.=` becomes `IterativeEqualityStatement`. Computation
justifications remain disallowed inside iterative equality because the
Chapter 15 production uses `simple_justification` there, but explicit
conclusions may reuse the general task-17 justification surface. These nodes
do not check equality transitivity, predecessor availability, conclusion
validity, or proof obligations. `SurfaceNodeView` exposes
`as_conclusion_statement`, `as_then_statement`,
`as_iterative_equality_statement`, and `as_iterative_equality_step` helpers.
Snapshot rendering prints the literal node names.

Parser task 20 adds the reasoning-block portion of S-013. `NowStatement`
owns optional label syntax, the `now` opener, zero or more nested statement
nodes, optional recovery, and the block-closing `end` and semicolon when
present. `HerebyStatement` owns the same block body shape without label syntax.
`CaseReasoningStatement` owns `per`, `cases`, optional simple citation
`JustificationClause`, the header semicolon, and zero or more source-ordered
homogeneous `CaseItem` children or homogeneous `SupposeItem` children. `CaseItem`
and `SupposeItem` own their
keyword, either a `Proposition` or `ConditionList`, the header semicolon, zero
or more nested statement nodes, optional recovery, and their block-closing
`end` and semicolon when present. The parser surface accepts both `per cases;`
and `per cases by A;` because Chapter 15 prose/examples and existing
parse-only fixtures exercise the no-explicit-justification form; it does not
classify exhaustiveness, branch coverage, label scope, witness leakage, or the
formula exported by a `now` block. `SurfaceNodeView` exposes
`as_now_statement`, `as_hereby_statement`, `as_case_reasoning_statement`,
`as_case_item`, and `as_suppose_item` helpers. Snapshot rendering prints the
literal node names.

Parser task 21 adds the local inline-definition portion of S-013.
`InlineFunctorDefinition` owns `deffunc`, the definition name identifier or a
`MissingTerm` recovery,
parameter parentheses, zero or more `TypedParameter` children separated by
comma tokens, the `->` return-type delimiter, a `TypeExpression` or
`MissingTypeExpression` recovery, `equals`, a `TermExpression` or `MissingTerm`
recovery, optional malformed-tail recovery, and the final semicolon when
present. `InlinePredicateDefinition` owns the same parameter head shape with
`defpred`, `means`, and a `FormulaExpression` or `MissingFormula` recovery.
`TypedParameter` owns the parameter identifier when present, optional `be` or
`being` when written, and the `TypeExpression` or `MissingTypeExpression`
recovery. A missing binder keyword is represented by the absence of that token
inside `TypedParameter`, plus a malformed-type diagnostic; if no recoverable
type follows before the delimiter, `MissingTypeExpression` fills the type slot.
These nodes do
not model scope introduction, definition expansion, parameter guard checks, or
later inline-name application resolution. `SurfaceNodeView` exposes
`as_inline_functor_definition`, `as_inline_predicate_definition`, and
`as_typed_parameter` helpers. Snapshot rendering prints the literal node names.

Parser task 23 starts S-015 definition-family vocabulary. `DefinitionBlockItem`
owns the `definition` opener, source-ordered content, optional recovery, and the
block-closing `end` plus semicolon when present. Concrete content in this
increment is intentionally limited to ordinary `DefinitionParameter` nodes,
`AssumptionStatement`, `AttributeDefinition`, `CorrectnessCondition`,
`TheoremItem`, `LemmaItem`, and visibility-wrapped theorem/lemma content.
Template-ambiguous
parameters such as `let T be type;` and later definition-family forms such as
predicate, functor, mode, redefinition, structure, property, registration,
cluster, and reduction declarations remain source-preserving `PlaceholderItem`
content until their paired parser tasks land.

`DefinitionParameter` owns `let`, qualified-variable segments, optional
condition-list or `such that` formula plus optional justification, optional
recovery, and the semicolon when present. `AttributeDefinition` owns `attr`, the
definition label, `:`, the subject identifier, `is`, an `AttributePattern`,
`means`, a `FormulaDefiniens`, optional recovery, and the semicolon when
present. `AttributePattern` owns an optional `ParameterPrefix` and an
identifier or user-symbol name. `FormulaDefiniens` owns either a single
`FormulaExpression` or conditional `FormulaCase` children with an optional
`otherwise` formula. `CorrectnessCondition` owns the correctness keyword and an
optional general justification, including `by`, `by computation(...)`, or a
full `ProofBlock`.

These nodes preserve definition syntax only. They do not check definitional
correctness, attribute admissibility, existence/uniqueness obligations,
cluster closure, notation resolution, type/proof semantics, or theorem
visibility. Missing attribute labels, subjects, and pattern names use
`MissingTerm`; missing formulas in definition parameters and formula definiens
cases use `MissingFormula`; malformed justifications may use
`MissingProofStep`; missing definition block closers use `MissingEnd`; malformed
tails are skipped under the owning definition content node. `SurfaceNodeView`
exposes `as_definition_block_item`, `as_definition_parameter`,
`as_attribute_definition`, `as_attribute_pattern`, `as_formula_definiens`,
`as_formula_case`, and `as_correctness_condition` helpers. Snapshot rendering
prints the literal node names.

Parser task 24 adds predicate definitions as the next S-015 increment.
`PredicateDefinition` owns `pred`, the definition label, `:`, a
`PredicatePattern`, `means`, a task-23 `FormulaDefiniens`, optional recovery,
and the semicolon when present. Definition-local `public pred` and
`private pred` are represented by the existing `VisibleItem` wrapper around the
`PredicateDefinition`; other visible definition kinds remain with their owning
parser tasks.

`PredicatePattern` preserves the pattern as raw source-ordered token children.
The parser validates that the raw span can match
`[ loci ] def_predicate_symbol [ template_loci ] [ loci ]` under at least one
syntactic split, but the AST does not record which identifier is the predicate
symbol. This keeps phrase-pattern ambiguity, such as prefix-like and
postfix-like two-identifier patterns, resolver-owned. Primitive built-in
predicate tokens `in`, `=`, and `<>` cannot satisfy `def_predicate_symbol` and
are represented as malformed predicate patterns with `MissingTerm` recovery.
Bracketed `template_loci` tokens may be preserved inside `PredicatePattern`,
but task 24 does not add template-specific nodes or classify
`definition ... end;` blocks as template definitions; G-AUD-006 remains open
for S-016. `SurfaceNodeView` exposes `as_predicate_definition` and
`as_predicate_pattern` helpers. Snapshot rendering prints the literal node
names.

Parser task 25 adds functor definitions as the next S-015 increment.
`FunctorDefinition` owns `func`, the definition label, `:`, a
`FunctorPattern`, `->`, a return `TypeExpression` or `MissingTypeExpression`,
either `means FormulaDefiniens` or `equals TermDefiniens`, optional recovery,
and the semicolon when present. Definition-local `public func` and
`private func` are represented by the existing `VisibleItem` wrapper around the
`FunctorDefinition`; correctness conditions following a functor remain
separate `CorrectnessCondition` definition-content nodes.

`FunctorPattern` preserves the pattern as raw source-ordered token children.
The parser validates that the raw span can match the canonical
`[ loci ] functor_symbol [ template_loci ] [ loci ]` shape under at least one
syntactic split, or the documented Chapter 10 two-symbol circumfix shape with
a non-empty loci list between functor-symbol tokens. The AST does not record
which token is the functor symbol, whether a pattern is prefix/postfix/infix,
or how a circumfix pair binds; those roles remain resolver-owned. Bracketed
`template_loci` tokens may be preserved inside `FunctorPattern`, but task 25
does not add template-specific nodes or classify `definition ... end;` blocks
as template definitions; G-AUD-006 remains open for S-016.

`TermDefiniens` and `TermCase` are the `equals`-body counterparts to
`FormulaDefiniens` and `FormulaCase`. `SurfaceNodeView` exposes
`as_functor_definition`, `as_functor_pattern`, `as_term_definiens`, and
`as_term_case` helpers. Snapshot rendering prints the literal node names.

Parser task 26 adds mode definitions as the next S-015 increment.
`ModeDefinition` owns `mode`, the definition label, `:`, a raw `ModePattern`,
`is`, a body `TypeExpression` or `MissingTypeExpression`, the first semicolon
when present, and an optional immediately-following `ModeProperty`.
Definition-local `public mode` and `private mode` are represented by the
existing `VisibleItem` wrapper around the `ModeDefinition`.

`ModePattern` preserves the pattern as raw source-ordered token children. The
parser validates that the raw span can match `mode_def_name [ type_params ]`
with exactly one identifier or active user-symbol name and at most one
non-empty type-parameter list introduced by `of`, `over`, or brackets. The AST
does not record whether the body head is semantically a radix type, whether the
parameter list is dependent, or whether the mode is inhabited. `ModeProperty`
owns the `sethood` keyword plus a required syntax-level general justification;
the sethood proof obligation remains outside the syntax crate.
`SurfaceNodeView` exposes `as_mode_definition`, `as_mode_pattern`, and
`as_mode_property` helpers. Snapshot rendering prints the literal node names.

Parser task 27 adds the redefinition and notation-alias portion of S-015.
`AttributeRedefinition`, `PredicateRedefinition`, and `FunctorRedefinition`
mirror the corresponding task-23 through task-25 definition bodies, but each
owns the leading `redefine` token and a mandatory trailing
`CoherenceCondition`. The spec defines redefinition productions only for
attributes, predicates, and functors; there is no `redefine_mode` production in
the canonical grammar. Mode syntax still participates in task-27 notation
aliases through raw `NotationPattern` children, while any `redefine mode`
source remains recovered/deferred rather than represented as invented language
behavior.
Definition-local `public` / `private` redefinitions use the existing
`VisibleItem` / `VisibilityMarker` wrapper around the concrete redefinition
node, matching Appendix A's `[ visibility ] definitional_item` shape.

`CoherenceCondition` owns the `coherence` keyword, optional `with` plus a label
identifier, a required syntax-level general justification (`JustificationClause`
or `ProofBlock`) when present, optional recovery, and the coherence semicolon
when present. It is separate from the general task-23 `CorrectnessCondition`
because redefinition coherence is part of the redefinition item rather than a
standalone definition-content clause. The AST does not check whether the
coherence proof establishes equivalence, result-type agreement, or compatibility
with the previous definition.

`NotationAlias` represents both `synonym` and `antonym` declarations. Operator
declarations remain a deferred branch of canonical `notation_decl`; this task
adds only the alias surface. `NotationAlias` owns the alias keyword, one
alternate `NotationPattern`, the `for` token, one original `NotationPattern`,
optional recovery, and the terminating semicolon when present.
`NotationPattern` preserves raw source-order tokens for one side of the alias.
The node deliberately does not classify the pattern as predicate, functor,
mode, or attribute and does not decide which token is the symbol being
introduced or referenced. That ambiguity depends on the active symbol
environment and remains resolver-owned.

Top-level and definition-local notation aliases use the same `NotationAlias`
surface; visibility is represented by the existing `VisibleItem` /
`VisibilityMarker` wrapper where the grammar admits `[ visibility ]
notation_decl`. `SurfaceNodeView` exposes `as_attribute_redefinition`,
`as_predicate_redefinition`, `as_functor_redefinition`,
`as_coherence_condition`, `as_notation_alias`, and `as_notation_pattern`
helpers. Snapshot rendering prints the literal node names.

Parser task 28 adds syntax-only property item clauses. `PropertyClause` owns
one canonical property keyword, a required syntax-level general justification
(`JustificationClause` or `ProofBlock`) when present, optional recovery, and
the property semicolon when present. The accepted keywords are the predicate
properties `symmetry`, `asymmetry`, `connectedness`, `reflexivity`, and
`irreflexivity`; the functor properties `commutativity`, `idempotence`,
`involutiveness`, and `projectivity`; and standalone mode `sethood`.
`transitivity` is a reserved word but is not a concrete property-clause
keyword in the current canonical property productions. A `sethood` clause
immediately following a `ModeDefinition` remains represented by task-26
`ModeProperty`; standalone `sethood` property items use `PropertyClause`. The
AST does not validate predicate arity, functor arity, proof obligations, or
which preceding definition a property annotates. `SurfaceNodeView` exposes
`as_property_clause`. Snapshot rendering prints `PropertyClause`.

Parser task 29 adds syntax-only structure definitions and inheritance
definitions. `StructureDefinition` owns `struct`, a raw `StructurePattern`,
`where`, one or more `StructureField` / `StructureProperty` members, `end`,
and the final semicolon when present. `StructurePattern` preserves raw
`struct_def_name [ type_params ]` tokens and does not decide semantic structure
identity. Structure fields own `field`, the field name, `->`, a syntactic
`TypeExpression`, an optional initializer `:= TermExpression`, and the member
semicolon; structure properties own the same shape without an initializer.
`InheritanceDefinition` owns `inherit`, child/parent `InheritanceTarget`
nodes around `extends`, optional explicit `where ... end` inheritance members,
and the final semicolon. `FieldRedefinition` and `PropertyRedefinition`
preserve optional narrowed types and required `from` source tokens; inheritance
coherence reuses `CoherenceCondition` with a mandatory general justification
and without the redefinition-only `with` label branch. The AST does not check
inheritance coverage, diamond consistency, selector compatibility, type
specialization validity, constructor semantics, or proof obligations.
`SurfaceNodeView` exposes `as_structure_definition`, `as_structure_pattern`,
`as_structure_field`, `as_structure_property`, `as_inheritance_definition`,
`as_inheritance_target`, `as_field_redefinition`, and
`as_property_redefinition`. Snapshot rendering prints the literal node names.

Parser task 30 completes the S-015 registration-family vocabulary.
`RegistrationBlockItem` owns the `registration` opener, source-ordered
registration content, optional recovery, and the block-closing `end` plus
semicolon when present. `RegistrationParameter` owns `let`, ordinary
qualified-variable segments, optional `such` condition-list children, optional
syntax-level `by` references, optional recovery, and the semicolon when
present. It does not own proof-bearing definition constraints or
template-definition parameter semantics.

`ExistentialRegistration`, `ConditionalRegistration`, and
`FunctorialRegistration` represent the three `cluster` branches.
`ExistentialRegistration` owns the `cluster` keyword, label, colon, one
attributed `TypeExpression`, the header semicolon, and an `existence`
`CorrectnessCondition`. `ConditionalRegistration` owns registration-adjective
`AttributeRef` children before `->`, consequent registration-adjective
`AttributeRef` children, `for`, a target `TypeExpression`, the header
semicolon, and a `coherence` `CorrectnessCondition`. `FunctorialRegistration`
owns a syntactically unambiguous functorial payload term, `->`, consequent
registration adjectives, `for`, the target type, the header semicolon, and a
`coherence` `CorrectnessCondition`. Registration adjective refs are restricted
to optional `non`, optional `ParameterPrefix`, and an attribute name without
parenthesized arguments; argument-bearing adjective spellings are recovered as
malformed syntax.

`ReductionRegistration` owns `reduce`, the label, colon, left
`TermExpression`, `to`, right `TermExpression`, the header semicolon, and a
`reducibility` `CorrectnessCondition`. Definition-local `public` / `private`
registration items reuse `VisibleItem` around the concrete registration item.
These nodes preserve registration syntax only. They do not compute cluster
closure, infer reduced forms, replay reducibility proofs, validate target
types, decide nullary functorial ambiguity, or check any proof obligation.
Missing labels and unsupported functorial payloads use `MissingTerm`; missing
antecedent/consequent adjectives and target types use `MissingTypeExpression`;
missing correctness justifications may use `MissingProofStep`; missing
registration block closers use `MissingEnd`; malformed tails are skipped under
the owning registration content node. `SurfaceNodeView` exposes
`as_registration_block_item`, `as_registration_parameter`,
`as_existential_registration`, `as_conditional_registration`,
`as_functorial_registration`, and `as_reduction_registration`. Snapshot
rendering prints the literal node names.

### Vocabulary Increment Contract

Node vocabulary grows only in the same change as the `mizar-parser` grammar task
that constructs the new shape. Before or with each increment, this spec must add
the implementation-facing contract for every new public syntax kind:

- the `SurfaceNodeKind` variant name and its raw `SyntaxKind` mapping;
- payload fields, if any, and whether they are parser facts or compatibility
  data;
- child roles and child order, including optional or repeated roles;
- range rules for the node and for its children, including any documented
  recovery exceptions;
- typed accessor or view helpers that consumers should use instead of raw rowan
  traversal;
- snapshot rendering text for the new kind and any escaping or sorting rules;
- recovery/trivia interaction, if the node owns skipped tokens, missing
  constructs, doc-comment attachment, or whitespace-sensitive hints.

The language grammar under `doc/spec/en/` defines what constructs exist. This
module spec defines how those constructs are represented in `SurfaceAst`.

### Builder Boundary

`SurfaceAstBuilder` is the parser-facing construction boundary. Parser code
adds tokens, ordinary nodes, and recovery nodes through builder methods, then
finishes with the root and optional expression root. Parser grammar code must
not push into a private arena, allocate rowan nodes directly, or rely on raw
rowan traversal. If grammar growth needs another tree operation, add it here
as a typed builder or accessor first.

Builder ids are local to one builder instance. A child, root, or expression-root
id from another builder is invalid. `add_node` creates ordinary structural nodes
only; token nodes must be created with `add_token` or `add_recovered_token`, and
recovery nodes with `add_recovery`. `finish` verifies that the optional root and
expression root exist and that non-root structural parents do not share child
subtrees.

During construction, parser infrastructure may inspect already-emitted builder
nodes through typed builder accessors such as `node_kind` and `node_range`.
Those accessors expose only the surface kind and source range needed for parser
composition; they do not expose the private builder arena as a storage contract.

The compatibility root may list both source-order token nodes and structural
nodes that contain those tokens, because task-12 consumers still inspect both
views. The rowan green tree remains source-shaped: when a structural child owns
the source tokens, the builder must emit those tokens once under the structural
rowan node rather than duplicating token leaves from the compatibility root
listing. Recovery nodes may keep context children outside their own insertion
range in compatibility views; those out-of-range context children are not
emitted under the recovery rowan node.

Current rowan construction deduplicates root-listed token nodes only when they
are also descendants of non-recovery structural root children. That structural
subtree may itself contain recovery nodes with in-range token children, as with
malformed import-tail recovery: the token leaves are emitted once under the
structural rowan subtree and omitted from the compatibility root's token pass.
Recovery nodes that are listed directly at the compatibility root are not
deduplication owners for root-listed tokens, so parser producers must not give
such root-level recovery nodes in-range token children unless a later builder
check or rowan emission rule documents that case. Use out-of-range context
children for missing-construct recovery, or nest skipped-token recovery under a
non-recovery structural owner and record the skipped source span in trivia.

### Accessor Conventions

`SurfaceAst::node_view`, `root_view`, `expression_view`, and `token_views`
return typed views that expose kind, range, recovered flag, children, token
payload, operator payload, and recovery kind without requiring rowan traversal.
The compatibility `SurfaceAst::node` accessor remains available for existing
tests and migration code.

### Snapshot Rendering

`SurfaceAst::snapshot_text` returns the deterministic, human-readable surface
snapshot format used by syntax tests and later parser corpus baselines. The
format is versioned with the `surface-ast-snapshot-v1` header and renders the
root view, optional expression root, and token compatibility view in stable
stored order. Each node line includes the surface kind, source-local byte range,
`recovered` flag, and kind-specific payload needed to distinguish the current
syntax vocabulary: token kind/text, operator spelling/precedence/fixity facts,
or recovery kind.

Snapshot text deliberately avoids rowan pointer identity, builder ids,
`SurfaceNodeId` values, raw `SourceId` debug output, absolute paths, timings,
hash-map iteration order, and other nondeterministic data. Ranges are rendered
as byte offsets within the `SurfaceAst` source; source identity belongs to the
outer snapshot/profile record owned by `mizar-test`.

`SurfaceAst::snapshot_text_with_trivia` appends the deterministic trivia side
table described in [trivia.md](./trivia.md). The default syntax snapshot omits
that section so existing syntax-only baselines remain stable.

The current syntax snapshot format is:

```text
surface-ast-snapshot-v1
root:
  <node-or-none>
expression_root:
  <node-or-none>
token_nodes:
  <node-or-none>
```

Node lines are indented by two spaces per depth and use these current forms:

```text
Root range=<start>..<end> recovered=<bool>
Token kind=<SurfaceTokenKind> text="<escaped-text>" range=<start>..<end> recovered=<bool>
CompilationUnit range=<start>..<end> recovered=<bool>
ItemList range=<start>..<end> recovered=<bool>
PlaceholderItem range=<start>..<end> recovered=<bool>
ImportItem range=<start>..<end> recovered=<bool>
ModuleBranchImport range=<start>..<end> recovered=<bool>
ImportAliasDecl range=<start>..<end> recovered=<bool>
ExportItem range=<start>..<end> recovered=<bool>
VisibleItem range=<start>..<end> recovered=<bool>
VisibilityMarker range=<start>..<end> recovered=<bool>
ModulePath range=<start>..<end> recovered=<bool>
NamespacePath range=<start>..<end> recovered=<bool>
QualifiedSymbol range=<start>..<end> recovered=<bool>
PathSegment range=<start>..<end> recovered=<bool>
RelativePrefix range=<start>..<end> recovered=<bool>
ReserveItem range=<start>..<end> recovered=<bool>
ReserveSegment range=<start>..<end> recovered=<bool>
TypeExpression range=<start>..<end> recovered=<bool>
AttributeChain range=<start>..<end> recovered=<bool>
AttributeRef range=<start>..<end> recovered=<bool>
TypeHead range=<start>..<end> recovered=<bool>
TypeArguments range=<start>..<end> recovered=<bool>
TermPlaceholder range=<start>..<end> recovered=<bool>
ParameterPrefix range=<start>..<end> recovered=<bool>
TermExpression range=<start>..<end> recovered=<bool>
TermReference range=<start>..<end> recovered=<bool>
NumeralTerm range=<start>..<end> recovered=<bool>
ItTerm range=<start>..<end> recovered=<bool>
ParenthesizedTerm range=<start>..<end> recovered=<bool>
ChoiceTerm range=<start>..<end> recovered=<bool>
ApplicationTerm range=<start>..<end> recovered=<bool>
StructureConstructor range=<start>..<end> recovered=<bool>
FieldArgument range=<start>..<end> recovered=<bool>
SetEnumeration range=<start>..<end> recovered=<bool>
SetComprehension range=<start>..<end> recovered=<bool>
ComprehensionVariableSegment range=<start>..<end> recovered=<bool>
SelectorAccess range=<start>..<end> recovered=<bool>
StructureUpdate range=<start>..<end> recovered=<bool>
FieldUpdate range=<start>..<end> recovered=<bool>
QuaExpression range=<start>..<end> recovered=<bool>
InfixExpression spelling="<escaped-text>" precedence=<u8> associativity=<SurfaceOperatorAssociativity> range=<start>..<end> recovered=<bool>
PrefixExpression spelling="<escaped-text>" precedence=<u8> range=<start>..<end> recovered=<bool>
PostfixExpression spelling="<escaped-text>" precedence=<u8> range=<start>..<end> recovered=<bool>
FormulaExpression range=<start>..<end> recovered=<bool>
BuiltinPredicateApplication range=<start>..<end> recovered=<bool>
IsAssertion range=<start>..<end> recovered=<bool>
AttributeTestChain range=<start>..<end> recovered=<bool>
PredicateApplication range=<start>..<end> recovered=<bool>
PredicateSegment range=<start>..<end> recovered=<bool>
PredicateHead range=<start>..<end> recovered=<bool>
InlinePredicateApplication range=<start>..<end> recovered=<bool>
PrefixFormula operator=<SurfaceFormulaPrefixOperator> range=<start>..<end> recovered=<bool>
BinaryFormula connective=<SurfaceFormulaConnective> repeated=<bool> range=<start>..<end> recovered=<bool>
ParenthesizedFormula range=<start>..<end> recovered=<bool>
QuantifiedFormula quantifier=<SurfaceQuantifierKind> range=<start>..<end> recovered=<bool>
QuantifierVariableSegment range=<start>..<end> recovered=<bool>
FormulaConstant constant=<SurfaceFormulaConstant> range=<start>..<end> recovered=<bool>
TemplateLoci range=<start>..<end> recovered=<bool>
TemplateLocus range=<start>..<end> recovered=<bool>
TemplateArguments range=<start>..<end> recovered=<bool>
TemplateArgument range=<start>..<end> recovered=<bool>
TemplateParameter range=<start>..<end> recovered=<bool>
AlgorithmDefinition range=<start>..<end> recovered=<bool>
AlgorithmParameters range=<start>..<end> recovered=<bool>
AlgorithmBody range=<start>..<end> recovered=<bool>
AlgorithmStatementList range=<start>..<end> recovered=<bool>
VariableDeclaration range=<start>..<end> recovered=<bool>
VariableBinding range=<start>..<end> recovered=<bool>
AssignmentStatement range=<start>..<end> recovered=<bool>
Lvalue range=<start>..<end> recovered=<bool>
SnapshotStatement range=<start>..<end> recovered=<bool>
ReturnStatement range=<start>..<end> recovered=<bool>
ClaimBlockItem range=<start>..<end> recovered=<bool>
IfStatement range=<start>..<end> recovered=<bool>
WhileStatement range=<start>..<end> recovered=<bool>
ForRangeStatement range=<start>..<end> recovered=<bool>
ForCollectionStatement range=<start>..<end> recovered=<bool>
MatchStatement range=<start>..<end> recovered=<bool>
MatchCase range=<start>..<end> recovered=<bool>
MatchEnding range=<start>..<end> recovered=<bool>
AlgorithmTerminationClause range=<start>..<end> recovered=<bool>
AlgorithmRequiresClause range=<start>..<end> recovered=<bool>
AlgorithmEnsuresClause range=<start>..<end> recovered=<bool>
AlgorithmDecreasingClause range=<start>..<end> recovered=<bool>
LoopInvariantClause range=<start>..<end> recovered=<bool>
LoopDecreasingClause range=<start>..<end> recovered=<bool>
AssertStatement range=<start>..<end> recovered=<bool>
TermList range=<start>..<end> recovered=<bool>
BreakStatement range=<start>..<end> recovered=<bool>
ContinueStatement range=<start>..<end> recovered=<bool>
StatementItem range=<start>..<end> recovered=<bool>
LetStatement range=<start>..<end> recovered=<bool>
QualifiedVariableSegment range=<start>..<end> recovered=<bool>
AssumptionStatement range=<start>..<end> recovered=<bool>
Proposition range=<start>..<end> recovered=<bool>
ConditionList range=<start>..<end> recovered=<bool>
GivenStatement range=<start>..<end> recovered=<bool>
TakeStatement range=<start>..<end> recovered=<bool>
Witness range=<start>..<end> recovered=<bool>
SetStatement range=<start>..<end> recovered=<bool>
Equating range=<start>..<end> recovered=<bool>
ConsiderStatement range=<start>..<end> recovered=<bool>
ReconsiderStatement range=<start>..<end> recovered=<bool>
ReconsiderItem range=<start>..<end> recovered=<bool>
ConclusionStatement range=<start>..<end> recovered=<bool>
ThenStatement range=<start>..<end> recovered=<bool>
IterativeEqualityStatement range=<start>..<end> recovered=<bool>
IterativeEqualityStep range=<start>..<end> recovered=<bool>
NowStatement range=<start>..<end> recovered=<bool>
HerebyStatement range=<start>..<end> recovered=<bool>
CaseReasoningStatement range=<start>..<end> recovered=<bool>
CaseItem range=<start>..<end> recovered=<bool>
SupposeItem range=<start>..<end> recovered=<bool>
InlineFunctorDefinition range=<start>..<end> recovered=<bool>
InlinePredicateDefinition range=<start>..<end> recovered=<bool>
TypedParameter range=<start>..<end> recovered=<bool>
TheoremItem range=<start>..<end> recovered=<bool>
LemmaItem range=<start>..<end> recovered=<bool>
ProofBlock range=<start>..<end> recovered=<bool>
DefinitionBlockItem range=<start>..<end> recovered=<bool>
DefinitionParameter range=<start>..<end> recovered=<bool>
AttributeDefinition range=<start>..<end> recovered=<bool>
AttributePattern range=<start>..<end> recovered=<bool>
FormulaDefiniens range=<start>..<end> recovered=<bool>
FormulaCase range=<start>..<end> recovered=<bool>
CorrectnessCondition range=<start>..<end> recovered=<bool>
PredicateDefinition range=<start>..<end> recovered=<bool>
PredicatePattern range=<start>..<end> recovered=<bool>
FunctorDefinition range=<start>..<end> recovered=<bool>
FunctorPattern range=<start>..<end> recovered=<bool>
TermDefiniens range=<start>..<end> recovered=<bool>
TermCase range=<start>..<end> recovered=<bool>
ModeDefinition range=<start>..<end> recovered=<bool>
ModePattern range=<start>..<end> recovered=<bool>
ModeProperty range=<start>..<end> recovered=<bool>
AttributeRedefinition range=<start>..<end> recovered=<bool>
PredicateRedefinition range=<start>..<end> recovered=<bool>
FunctorRedefinition range=<start>..<end> recovered=<bool>
CoherenceCondition range=<start>..<end> recovered=<bool>
NotationAlias range=<start>..<end> recovered=<bool>
NotationPattern range=<start>..<end> recovered=<bool>
PropertyClause range=<start>..<end> recovered=<bool>
StructureDefinition range=<start>..<end> recovered=<bool>
StructurePattern range=<start>..<end> recovered=<bool>
StructureField range=<start>..<end> recovered=<bool>
StructureProperty range=<start>..<end> recovered=<bool>
InheritanceDefinition range=<start>..<end> recovered=<bool>
InheritanceTarget range=<start>..<end> recovered=<bool>
FieldRedefinition range=<start>..<end> recovered=<bool>
PropertyRedefinition range=<start>..<end> recovered=<bool>
RegistrationBlockItem range=<start>..<end> recovered=<bool>
RegistrationParameter range=<start>..<end> recovered=<bool>
ExistentialRegistration range=<start>..<end> recovered=<bool>
ConditionalRegistration range=<start>..<end> recovered=<bool>
FunctorialRegistration range=<start>..<end> recovered=<bool>
ReductionRegistration range=<start>..<end> recovered=<bool>
CompactStatement range=<start>..<end> recovered=<bool>
JustificationClause range=<start>..<end> recovered=<bool>
ReferenceList range=<start>..<end> recovered=<bool>
Reference range=<start>..<end> recovered=<bool>
QualifiedReference range=<start>..<end> recovered=<bool>
GroupedReference range=<start>..<end> recovered=<bool>
GroupedReferenceItem range=<start>..<end> recovered=<bool>
BulkReference range=<start>..<end> recovered=<bool>
ComputationJustification range=<start>..<end> recovered=<bool>
ComputationOption range=<start>..<end> recovered=<bool>
ErrorRecovery kind=<SyntaxRecoveryKind> range=<start>..<end> recovered=<bool>
```

`<escaped-text>` uses Rust default character escaping so control characters,
quotes, backslashes, and non-printing characters render deterministically. Snapshot
format changes require a new header version plus updates to this spec, the
Japanese companion, and affected baseline snapshots. Update `mizar-test`
snapshot documentation only when the outer snapshot envelope or update policy
changes.

### Range Attachment

Every surface node carries a `SourceRange` from `mizar-session`. For ordinary
nodes, parent ranges contain all child ranges. Recovery nodes may violate that
containment when a zero-width insertion node keeps an opener or skipped token
as context; for example, a missing-`end` recovery node is attached at the EOF
insertion range while its child points back to the block opener.

### Identity Rules

Rowan green-node identity, rowan text ranges, and dense `SurfaceNodeId` values
are internal cache and compatibility details. They are deterministic within a
constructed `SurfaceAst`, but they are not stable artifact ids and must not be
serialized as cross-run identities. Stable consumers should key on deterministic
snapshots, content cache keys, source ids/ranges, and later semantic ids owned
by resolver/checker layers.

### Public Enum Compatibility

The current public syntax enums are not yet the long-lived resolver/LSP surface.
Before parser tasks 5-7 make them plausible downstream inputs, apply the
pre-consumer gate in [todo.md](./todo.md): enums that promise future vocabulary
growth (`SyntaxKind`, `SurfaceNodeKind`, and `SurfaceTokenKind`) are marked
`#[non_exhaustive]` for downstream crates, and the lint-policy gate keeps those
attributes present. `MizarLanguage` remains deliberately exhaustive because it
is an empty rowan marker enum rather than a downstream syntax category.
`SurfaceOperatorAssociativity` is currently a closed three-way operator property
(`Left`, `Right`, `NonAssociative`) and remains deliberately exhaustive unless a
later operator-model task designs a new associativity category. The task-14
formula payload enums (`SurfaceFormulaPrefixOperator`,
`SurfaceFormulaConnective`, `SurfaceQuantifierKind`, and
`SurfaceFormulaConstant`) are also deliberately exhaustive because they encode
the current fixed grammar table; adding a new formula operator, quantifier, or
constant must force local parser/syntax matches and documentation to update.
Matches inside this crate should remain exhaustive so new variants cause local
compile-time updates; downstream crates must include wildcard fallback arms
where `#[non_exhaustive]` requires them.

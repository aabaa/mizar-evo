# mizar-syntax: Surface AST

> 正本は英語です。英語版: [../en/ast.md](../en/ast.md)。

状態: rowan-backed storage 境界と task 12 互換 view は実装済み。完全な AST 範囲は計画中。

## 目的

このモジュールは、`mizar-parser` が生成する、ソースの形を保った `SurfaceAst` を定義する。
`SurfaceAst` は immutable な rowan green tree を backend とする。現在の
`SurfaceNode` / `SurfaceNodeId` surface は、parser と frontend が task 12 の
最小 tree 形状から移行する間の互換 view として残す。

## 責務

- `SurfaceAst`、rowan syntax kind、互換用の構文ノード ID、parser 向け構築 API を定義する。
- ソース順、ソース範囲、回復ノードを保持する。
- モジュール、項目、項、論理式、文、証明、アルゴリズム、アノテーションを表現する。
- 解決済みシンボル ID、推論済み型、overload resolution result、cluster fact、証明義務を持たない。

## 公開 API

### Storage 境界

`SurfaceAst` は rowan green tree を所有する。rowan は構文形状と決定的共有の
storage backend であり、compiler の意味的 identity surface ではない。消費者は
storage 境界そのものをテストする場合を除き、`SurfaceAst` と
`SurfaceNodeView` の typed accessor を使う。生の rowan root は
`SurfaceAst::rowan_root` から、green node は `SurfaceAst::green_node` から取得
できるが、用途は infrastructure test と明示的に文書化された統合に限る。

task 12 の互換 data（`SurfaceNode`、`SurfaceNodeId`、`token_nodes`、`root`、
`expression_root`）は、`SurfaceAst` 内部の private field に backed される。ただし
その surface の一部は移行中の公開 API として残る。互換型、read-only accessor、
`SurfaceNode` の constructor / field は、`mizar-parser`、`mizar-frontend`、既存
test が現在の最小形状を検査し続けられるように export されている。これは公開
互換 API であり、storage backend でも安定 artifact schema でもない。新しい
consumer は `SurfaceNodeView` と typed accessor を優先するべきである。互換 id と
node は cross-run identity として serialize してはならず、consumer が green tree
と独立に mutation することもできない。

### Syntax kind mapping

`SyntaxKind` は rowan の raw kind 語彙である。現在の node kind mapping は以下:

| surface role | raw kind |
|---|---|
| root node | `SyntaxKind::Root` |
| 互換 token node | `SyntaxKind::Token` |
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
| algorithm termination clause node | `SyntaxKind::AlgorithmTerminationClause` |
| algorithm requires clause node | `SyntaxKind::AlgorithmRequiresClause` |
| algorithm ensures clause node | `SyntaxKind::AlgorithmEnsuresClause` |
| algorithm decreasing clause node | `SyntaxKind::AlgorithmDecreasingClause` |
| loop invariant clause node | `SyntaxKind::LoopInvariantClause` |
| loop decreasing clause node | `SyntaxKind::LoopDecreasingClause` |
| algorithm assert statement node | `SyntaxKind::AssertStatement` |
| term list node | `SyntaxKind::TermList` |
| annotation node | `SyntaxKind::Annotation` |
| library annotation node | `SyntaxKind::LibraryAnnotation` |
| annotation label list node | `SyntaxKind::AnnotationLabelList` |
| annotation label node | `SyntaxKind::AnnotationLabel` |
| annotation argument list node | `SyntaxKind::AnnotationArgumentList` |
| annotation argument node | `SyntaxKind::AnnotationArgument` |
| proof-hint option list node | `SyntaxKind::ProofHintOptionList` |
| proof-hint option node | `SyntaxKind::ProofHintOption` |
| standalone diagnostic annotation node | `SyntaxKind::StandaloneDiagnosticAnnotation` |
| annotated statement wrapper node | `SyntaxKind::AnnotatedStatement` |
| annotated algorithm-statement wrapper node | `SyntaxKind::AnnotatedAlgorithmStatement` |
| annotated definition-content wrapper node | `SyntaxKind::AnnotatedDefinitionContent` |
| annotated registration-content wrapper node | `SyntaxKind::AnnotatedRegistrationContent` |
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

token role は別の raw kind として、identifier、reserved word、reserved symbol、
numeral、lexeme run、user symbol、annotation marker、string literal、
error-recovery token、unknown token を持つ。rowan tree は source-shaped であり、各 token はソース順に
一度だけ rowan token leaf として現れる。互換 side table は task 12 API のために
token payload を保持してよいが、それによって rowan tree 内の token leaf や text
を重複させてはならない。

現在の raw discriminant は、この段階の rowan 境界の一部である。

| raw value | `SyntaxKind` | role |
|---:|---|---|
| 0 | `Unknown` | 認識できない raw rowan kind の fallback |
| 1 | `Root` | root node |
| 2 | `Token` | 互換 token wrapper node |
| 3 | `InfixExpression` | infix expression node |
| 4 | `ErrorRecovery` | recovery node |
| 5 | `ModulePath` | module import/export path node |
| 6 | `NamespacePath` | citation/reference namespace path node |
| 7 | `QualifiedSymbol` | dotted active user symbol node。attribute-ref の structure prefix も含む |
| 8 | `PathSegment` | 単一 identifier または user-symbol segment wrapper |
| 9 | `RelativePrefix` | `.` / `..` import-relative prefix wrapper |
| 10 | `CompilationUnit` | module file skeleton node |
| 11 | `ItemList` | top-level item list node |
| 12 | `PlaceholderItem` | task-5 keyword-dispatched placeholder item node |
| 13 | `ImportItem` | task-6 concrete `import` item node |
| 14 | `ImportAliasDecl` | task-6 simple import または alias declaration node |
| 15 | `ModuleBranchImport` | task-6 branch import declaration node |
| 16 | `ExportItem` | task-7 concrete `export` item node |
| 17 | `VisibilityMarker` | task-7 `private` / `public` token wrapper |
| 18 | `VisibleItem` | task-7 visible top-level item wrapper |
| 19 | `ReserveItem` | task-8 concrete top-level `reserve` host item |
| 20 | `ReserveSegment` | task-8 `identifier_list "for" type_expression` segment |
| 21 | `TypeExpression` | task-8 `attribute_chain type_head` node |
| 22 | `AttributeChain` | task-8 non-empty attribute reference 列 |
| 23 | `AttributeRef` | task-8 任意の `non` を含む syntactic attribute reference |
| 24 | `ParameterPrefix` | task-8 attribute parameter-prefix wrapper |
| 25 | `TypeHead` | task-8 generic radix-or-mode type head |
| 26 | `TypeArguments` | task-8 `of` / `over` / bracket argument wrapper |
| 27 | `TermPlaceholder` | task 8 の legacy bracket `qua_arg` stub。task 11 の parser path では生成しない |
| 28 | `TermExpression` | task-9 current term-expression wrapper |
| 29 | `TermReference` | task-9 identifier または qualified-symbol term reference |
| 30 | `NumeralTerm` | task-9 numeral term |
| 31 | `ItTerm` | task-9 `it` term |
| 32 | `ParenthesizedTerm` | task-9 parenthesized term |
| 33 | `ChoiceTerm` | task-9 `"the" type_expression` term |
| 34 | `ApplicationTerm` | task-9 parenthesized / reserved-bracket application term |
| 35 | `StructureConstructor` | task-9 named-field structure-constructor surface |
| 36 | `FieldArgument` | task-9 structure-constructor field argument |
| 37 | `SetEnumeration` | task-9 set-enumeration term |
| 38 | `SelectorAccess` | task-10 selector postfix / selector-call surface |
| 39 | `StructureUpdate` | task-10 functional structure-update postfix |
| 40 | `FieldUpdate` | task-10 structure-update field assignment |
| 41 | `QuaExpression` | task-11 の `term "qua" type_expression` qualification surface |
| 42 | `PrefixExpression` | task-12 prefix operator expression surface |
| 43 | `PostfixExpression` | task-12 postfix operator expression surface |
| 44 | `FormulaExpression` | task 13/14 の formula child 1 個用 wrapper |
| 45 | `BuiltinPredicateApplication` | task-13 の `term_expression builtin_pred term_expression` atomic formula |
| 46 | `IsAssertion` | task-13 の generic `term_expression "is" ...` assertion |
| 47 | `AttributeTestChain` | task-13 の attribute-only `is_assertion_body` chain |
| 48 | `PredicateApplication` | task-13 の syntax-only user predicate application / chain |
| 49 | `PredicateSegment` | task-13 の user predicate segment |
| 50 | `PredicateHead` | task-13 の predicate symbol wrapper |
| 51 | `InlinePredicateApplication` | task-13 の inline predicate call shape |
| 52 | `PrefixFormula` | task-14 の fixed prefix formula shape |
| 53 | `BinaryFormula` | task-14 の fixed binary connective formula shape |
| 54 | `ParenthesizedFormula` | task-14 の parenthesized formula operand |
| 55 | `QuantifiedFormula` | task-14 の universal / existential formula |
| 56 | `QuantifierVariableSegment` | task-14 の quantified variable segment |
| 57 | `FormulaConstant` | task-14 の `thesis` / `contradiction` formula constant |
| 58 | `SetComprehension` | task-15 の set-comprehension / Fraenkel term |
| 59 | `ComprehensionVariableSegment` | task-15 の typed generator segment |
| 60 | `StatementItem` | task-16 の concrete statement 用一時 item host |
| 61 | `LetStatement` | task-16 の `let` generalization statement |
| 62 | `QualifiedVariableSegment` | task-16 の statement-level qualified variable segment |
| 63 | `AssumptionStatement` | task-16 の `assume` / `assume that` statement |
| 64 | `Proposition` | task-16 の任意 label と formula proposition |
| 65 | `ConditionList` | task-16 の `that` / `and` statement-level condition list |
| 66 | `GivenStatement` | task-16 の existential assumption statement |
| 67 | `TakeStatement` | task-16 の witness introduction statement |
| 68 | `Witness` | task-16 の named / unnamed witness item |
| 69 | `SetStatement` | task-16 の local constant-definition statement |
| 70 | `Equating` | task-16 の `set` equating item |
| 71 | `CompactStatement` | task-17 の最小の明示的 justification 付き proposition host |
| 72 | `JustificationClause` | task-17 の `by` citation または computation proof clause |
| 73 | `ReferenceList` | task-17 の comma-separated citation list |
| 74 | `Reference` | task-17 の local reference citation |
| 75 | `QualifiedReference` | task-17 の namespace-qualified reference citation |
| 76 | `GroupedReference` | task-17 の `namespace_path ".{" ... "}"` grouped citation |
| 77 | `GroupedReferenceItem` | task-17 の grouped citation item |
| 78 | `BulkReference` | task-17 の `namespace_path ".*"` bulk citation |
| 79 | `ComputationJustification` | task-17 の `by computation(...)` justification payload |
| 80 | `ComputationOption` | task-17 の `steps` / `timeout` / `nest` computation option |
| 81 | `ConsiderStatement` | task-18 の `consider ... such that ... by ...` choice statement |
| 82 | `ReconsiderStatement` | task-18 の `reconsider ... as ... by ...` type-changing statement |
| 83 | `ReconsiderItem` | task-18 の bare / equated reconsider item |
| 84 | `ConclusionStatement` | task-19 の `thus` / `hence` conclusion statement |
| 85 | `ThenStatement` | task-19 の linkable statement に対する `then` modifier wrapper |
| 86 | `IterativeEqualityStatement` | task-19 の 1 個以上の `.=` continuation を持つ equality chain |
| 87 | `IterativeEqualityStep` | task-19 の `.=` equality-chain continuation step |
| 88 | `NowStatement` | task-20 の labelled reasoning block |
| 89 | `HerebyStatement` | task-20 の diffuse conclusion block |
| 90 | `CaseReasoningStatement` | task-20 の `per cases` reasoning block |
| 91 | `CaseItem` | task-20 の `case ... end;` branch |
| 92 | `SupposeItem` | task-20 の `suppose ... end;` branch |
| 93 | `InlineFunctorDefinition` | task-21 の `deffunc ... equals ...;` local definition |
| 94 | `InlinePredicateDefinition` | task-21 の `defpred ... means ...;` local definition |
| 95 | `TypedParameter` | task-21 の inline-definition typed parameter |
| 96 | `TheoremItem` | task-22 の theorem declaration item |
| 97 | `LemmaItem` | task-22 の lemma declaration item |
| 98 | `ProofBlock` | task-22 の `proof ... end` justification block |
| 99 | `TokenAnnotationMarker` | `@identifier` annotation marker token leaf |
| 100 | `TokenIdentifier` | identifier token leaf |
| 101 | `TokenReservedWord` | reserved-word token leaf |
| 102 | `TokenReservedSymbol` | reserved-symbol token leaf |
| 103 | `TokenNumeral` | numeral token leaf |
| 104 | `TokenLexemeRun` | lexeme-run token leaf |
| 105 | `TokenUserSymbol` | user-symbol token leaf |
| 106 | `TokenStringLiteral` | string-literal token leaf |
| 107 | `TokenErrorRecovery` | lexer recovery token leaf |
| 108 | `TokenUnknown` | unknown token leaf |
| 109 | `DefinitionBlockItem` | task-23 の `definition ... end;` item |
| 110 | `DefinitionParameter` | task-23 の通常 definition `let` parameter |
| 111 | `AttributeDefinition` | task-23 の `attr ... means ...;` definition |
| 112 | `AttributePattern` | task-23 の attribute pattern head |
| 113 | `FormulaDefiniens` | task-23 の formula definiens body |
| 114 | `FormulaCase` | task-23 の conditional formula definiens case |
| 115 | `CorrectnessCondition` | task-23 の correctness-condition clause |
| 116 | `PredicateDefinition` | task-24 の `pred ... means ...;` definition |
| 117 | `PredicatePattern` | task-24 の raw predicate definition pattern |
| 118 | `FunctorDefinition` | task-25 の `func ... means|equals ...;` definition |
| 119 | `FunctorPattern` | task-25 の raw functor definition pattern |
| 120 | `TermDefiniens` | task-25 の term definiens body |
| 121 | `TermCase` | task-25 の conditional term definiens case |
| 122 | `ModeDefinition` | task-26 の `mode ... is ...;` definition |
| 123 | `ModePattern` | task-26 の raw mode definition pattern |
| 124 | `ModeProperty` | task-26 の mode definition に付属する `sethood` property |
| 125 | `AttributeRedefinition` | task-27 の `redefine attr ... means ...; coherence ...;` definition |
| 126 | `PredicateRedefinition` | task-27 の `redefine pred ... means ...; coherence ...;` definition |
| 127 | `FunctorRedefinition` | task-27 の `redefine func ... -> ... means|equals ...; coherence ...;` definition |
| 128 | `CoherenceCondition` | task-27 の必須 redefinition coherence proof tail |
| 129 | `NotationAlias` | task-27 の `synonym` / `antonym` notation alias declaration |
| 130 | `NotationPattern` | task-27 の raw notation alias pattern span |
| 131 | `PropertyClause` | task-28 の predicate / functor / standalone mode property clause |
| 132 | `StructureDefinition` | task-29 の `struct ... where ... end;` definition |
| 133 | `StructurePattern` | task-29 の raw structure definition name と parameter |
| 134 | `StructureField` | task-29 の structure `field` member |
| 135 | `StructureProperty` | task-29 の structure `property` member |
| 136 | `InheritanceDefinition` | task-29 の `inherit ... extends ...` definition |
| 137 | `InheritanceTarget` | task-29 の raw inheritance child / parent target |
| 138 | `FieldRedefinition` | task-29 の inherited field mapping |
| 139 | `PropertyRedefinition` | task-29 の inherited property mapping |
| 140 | `RegistrationBlockItem` | task-30 の `registration ... end;` block item |
| 141 | `RegistrationParameter` | task-30 の registration-local `let` parameter |
| 142 | `ExistentialRegistration` | task-30 の existential `cluster ... existence ...;` registration |
| 143 | `ConditionalRegistration` | task-30 の conditional `cluster ... -> ... coherence ...;` registration |
| 144 | `FunctorialRegistration` | task-30 の functorial `cluster term -> ... coherence ...;` registration |
| 145 | `ReductionRegistration` | task-30 の `reduce ... to ... reducibility ...;` registration |
| 146 | `TemplateParameter` | task-31 の template definition `let` parameter |
| 147 | `TemplateLoci` | task-31 の pattern 側 `[` locus list `]` wrapper |
| 148 | `TemplateLocus` | task-31 の pattern 側 template locus |
| 149 | `TemplateArguments` | task-31 の call / reference 側 template argument list |
| 150 | `TemplateArgument` | task-31 の call / reference 側 template argument |
| 151 | `AlgorithmDefinition` | task-32 の `algorithm ... do ... end;` definition content |
| 152 | `AlgorithmParameters` | task-32 の algorithm formal parameter list |
| 153 | `AlgorithmBody` | task-32 の algorithm `do ... end` body |
| 154 | `AlgorithmStatementList` | task-32 の source-ordered algorithm statement list |
| 155 | `VariableDeclaration` | task-32 の `var` / `const` / ghost declaration statement |
| 156 | `VariableBinding` | task-32 の単一 declaration binding |
| 157 | `AssignmentStatement` | task-32 の assignment または ghost assignment statement |
| 158 | `Lvalue` | task-32 の syntactic assignment target |
| 159 | `SnapshotStatement` | task-32 の `snapshot` statement |
| 160 | `ReturnStatement` | task-32 の `return` statement |
| 161 | `ClaimBlockItem` | task-32 の top-level `claim ... do ... end;` item |
| 162 | `IfStatement` | task-33 の algorithm `if ... do ... [else ...] end;` statement |
| 163 | `WhileStatement` | task-33 の algorithm `while ... do ... end;` statement |
| 164 | `ForRangeStatement` | task-33 の algorithm `for i = ... to|downto ... [step ...] do ... end;` statement |
| 165 | `ForCollectionStatement` | task-33 の algorithm `for x in S [processed V] do ... end;` statement |
| 166 | `MatchStatement` | task-33 の algorithm `match ... do ... end;` statement |
| 167 | `MatchCase` | task-33 の algorithm `case pattern do ... end;` branch |
| 168 | `MatchEnding` | task-33 の `otherwise ... end;` または `exhaustive [justification];` match ending |
| 169 | `BreakStatement` | task-33 の algorithm `break;` statement |
| 170 | `ContinueStatement` | task-33 の algorithm `continue;` statement |
| 171 | `AlgorithmTerminationClause` | task-34 の `terminating` algorithm modifier |
| 172 | `AlgorithmRequiresClause` | task-34 の header `requires formula` clause |
| 173 | `AlgorithmEnsuresClause` | task-34 の header `ensures formula` clause |
| 174 | `AlgorithmDecreasingClause` | task-34 の header `decreasing term_list` clause |
| 175 | `LoopInvariantClause` | task-34 の leading loop `invariant formula [justification];` clause |
| 176 | `LoopDecreasingClause` | task-34 の leading while-loop `decreasing term_list [justification];` clause |
| 177 | `AssertStatement` | task-34 の algorithm `assert formula [justification];` statement |
| 178 | `TermList` | task-34 の comma-separated decreasing-measure term list |
| 179 | `Annotation` | task-35 の attachable annotation prefix |
| 180 | `LibraryAnnotation` | task-35 の `@[ ... ]` annotation prefix |
| 181 | `AnnotationLabelList` | task-35 の comma-separated library annotation label |
| 182 | `AnnotationLabel` | task-35 の library annotation label |
| 183 | `AnnotationArgumentList` | task-35 の parenthesized annotation argument list |
| 184 | `AnnotationArgument` | task-35 の単一 annotation argument |
| 185 | `ProofHintOptionList` | task-35 の `@proof_hint(...)` option list |
| 186 | `ProofHintOption` | task-35 の単一 proof-hint option |
| 187 | `StandaloneDiagnosticAnnotation` | task-35 の `@show_type` / `@eval` standalone diagnostic annotation |
| 188 | `AnnotatedStatement` | task-35 の ordinary statement 用 annotation wrapper |
| 189 | `AnnotatedAlgorithmStatement` | task-35 の algorithm statement 用 annotation wrapper |
| 190 | `AnnotatedDefinitionContent` | task-35 の definition content 用 annotation wrapper |
| 191 | `AnnotatedRegistrationContent` | task-35 の registration content 用 annotation wrapper |

`SyntaxKind::from_raw` は未知の raw value をすべて `Unknown` に写像する。
`SyntaxKind::is_node_kind` は上に列挙したすべての structural node raw kind、つまり現在は
`Root` から task-22 `ProofBlock` まで、task-23 `DefinitionBlockItem` から
task-35 `AnnotatedRegistrationContent` まで、compatibility `Token` wrapper、および
`ErrorRecovery` に対して true である。`is_token_kind` は `TokenAnnotationMarker` から
`TokenUnknown` までの token leaf raw kind に対してのみ true である。
将来の raw value は、既存 snapshot と rowan test が raw 語彙変更時に明確に失敗するよう、
末尾へ追加するか、文書化された予約 range に割り当てるべきである。

### 現在の surface 語彙

現在実装済みの surface node 語彙は意図的に小さい。

| 公開 surface kind | payload | raw rowan node kind | 注記 |
|---|---|---|---|
| `SurfaceNodeKind::Root` | なし | `SyntaxKind::Root` | top-level 互換 root |
| `SurfaceNodeKind::Token(SurfaceToken)` | token kind と interned text | token raw kind の token leaf を 1 つ持つ `SyntaxKind::Token` | rowan token leaf の互換 wrapper |
| `SurfaceNodeKind::CompilationUnit` | なし | `SyntaxKind::CompilationUnit` | parser task 5 の module file skeleton。`ItemList` child を 1 つ持ち、semantic module identity は持たない |
| `SurfaceNodeKind::ItemList` | なし | `SyntaxKind::ItemList` | top-level item placeholder、一時的な `StatementItem` host、item-level recovery marker の source-order list |
| `SurfaceNodeKind::PlaceholderItem` | なし | `SyntaxKind::PlaceholderItem` | 後続 task が concrete item node に置き換えるまで使う、keyword-dispatched top-level item placeholder |
| `SurfaceNodeKind::ImportItem` | なし | `SyntaxKind::ImportItem` | parser task 6 の concrete `import_stmt`。`import` token、comma token で区切られた import declaration node、任意の malformed-tail recovery、任意の semicolon token を所有する |
| `SurfaceNodeKind::ImportAliasDecl` | なし | `SyntaxKind::ImportAliasDecl` | parser task 6 の `module_path ["as" module_identifier]`。`ModulePath`、任意の `as` token、任意の alias `PathSegment`、任意の malformed-tail recovery を所有する |
| `SurfaceNodeKind::ModuleBranchImport` | なし | `SyntaxKind::ModuleBranchImport` | parser task 6 の `module_path ".{" module_identifier { "," module_identifier } "}"`。base `ModulePath`、`.{` token、comma token で区切られた branch `PathSegment`、任意の malformed-tail recovery、任意の `}` を所有する |
| `SurfaceNodeKind::ExportItem` | なし | `SyntaxKind::ExportItem` | parser task 7 の concrete `export_stmt`。`export` token、comma token で区切られた exported `ModulePath` node、任意の malformed-tail recovery、任意の semicolon token を所有する |
| `SurfaceNodeKind::VisibilityMarker` | なし | `SyntaxKind::VisibilityMarker` | parser task 7 の `private` または `public` token 1 個だけを包む wrapper |
| `SurfaceNodeKind::VisibleItem` | なし | `SyntaxKind::VisibleItem` | parser task 7 の top-level visibility wrapper。parsed `Annotation` prefix node があればそれら、1 個の `VisibilityMarker`、現在の target item node を所有する |
| `SurfaceNodeKind::ReserveItem` | なし | `SyntaxKind::ReserveItem` | parser task 8 の concrete top-level `reserve_decl` host item。`reserve` token、1 個の `ReserveSegment`、任意の malformed-tail recovery、任意の semicolon token を所有する |
| `SurfaceNodeKind::ReserveSegment` | なし | `SyntaxKind::ReserveSegment` | parser task 8 の `identifier_list "for" type_expression`。comma token で区切られた identifier token、`for` token、`TypeExpression` または missing-type recovery を所有する |
| `SurfaceNodeKind::TypeExpression` | なし | `SyntaxKind::TypeExpression` | parser task 8 の `attribute_chain type_head`。任意の non-empty `AttributeChain` と generic `TypeHead` を所有する |
| `SurfaceNodeKind::AttributeChain` | なし | `SyntaxKind::AttributeChain` | parser task 8 の non-empty `AttributeRef` node 列 |
| `SurfaceNodeKind::AttributeRef` | なし | `SyntaxKind::AttributeRef` | parser task 8 の任意の `non` token、任意の `ParameterPrefix`、syntactic `QualifiedSymbol`、任意の parenthesized term argument |
| `SurfaceNodeKind::ParameterPrefix` | なし | `SyntaxKind::ParameterPrefix` | parser task 8 の attribute parameter prefix。`parameter "-"` または `"(" parameter_list ")" "-"` |
| `SurfaceNodeKind::TypeHead` | なし | `SyntaxKind::TypeHead` | parser task 8 の generic radix-or-mode head。builtin `object`/`set` token または `QualifiedSymbol` と、任意の `TypeArguments` を所有する |
| `SurfaceNodeKind::TypeArguments` | なし | `SyntaxKind::TypeArguments` | parser task 8 の `of`、`over`、または bracket 構文の type argument wrapper。task 9 は `of`/`over` placeholder を `TermExpression` argument に置き換え、task 11 は bracket `qua_arg` placeholder を `TermExpression` / `QuaExpression` surface に置き換える |
| `SurfaceNodeKind::TermPlaceholder` | なし | `SyntaxKind::TermPlaceholder` | raw-kind 互換性のために残る parser task 8 の legacy syntax-only term-entry stub。task 11 の parser は bracket `qua_arg` 形に対してこれを生成しない |
| `SurfaceNodeKind::TermExpression` | なし | `SyntaxKind::TermExpression` | parser task 9 の current term-expression wrapper。primary term、postfix chain、`QuaExpression`、後続 operator expression のいずれか 1 つの current term-shape child を所有する |
| `SurfaceNodeKind::TermReference` | なし | `SyntaxKind::TermReference` | parser task 9 の term position の identifier token または共有 `QualifiedSymbol`。parenthesized application の前に task 31 の任意の `TemplateArguments` を持てる。semantic classification は持たない |
| `SurfaceNodeKind::NumeralTerm` | なし | `SyntaxKind::NumeralTerm` | parser task 9 の numeral term wrapper |
| `SurfaceNodeKind::ItTerm` | なし | `SyntaxKind::ItTerm` | parser task 9 の `it` keyword term wrapper |
| `SurfaceNodeKind::ParenthesizedTerm` | なし | `SyntaxKind::ParenthesizedTerm` | parser task 9 の parenthesized term。`(`、`TermExpression` または `MissingTerm`、任意の `)` を所有する |
| `SurfaceNodeKind::ChoiceTerm` | なし | `SyntaxKind::ChoiceTerm` | parser task 9 の `"the" TypeExpression` choice term。type operand 欠落時は `MissingTypeExpression` recovery を使う |
| `SurfaceNodeKind::ApplicationTerm` | なし | `SyntaxKind::ApplicationTerm` | parser task 9 の ordinary parenthesized application または reserved-bracket functor application。delimiter と source-order term argument を所有する |
| `SurfaceNodeKind::StructureConstructor` | なし | `SyntaxKind::StructureConstructor` | parser task 9 の、named field argument が見える場合の syntax-only structure-constructor surface |
| `SurfaceNodeKind::FieldArgument` | なし | `SyntaxKind::FieldArgument` | parser task 9 の `identifier ":" term_expression` field argument |
| `SurfaceNodeKind::SetEnumeration` | なし | `SyntaxKind::SetEnumeration` | parser task 9 の set-enumeration term |
| `SurfaceNodeKind::SetComprehension` | なし | `SyntaxKind::SetComprehension` | parser task 15 の set-comprehension / Fraenkel term。`{`、mapper `TermExpression`、`where`、generator segment、任意の condition formula、`}` または delimiter recovery を所有する |
| `SurfaceNodeKind::ComprehensionVariableSegment` | なし | `SyntaxKind::ComprehensionVariableSegment` | parser task 15 の typed generator segment。identifier または `MissingTerm` recovery、任意の `is`、および `is` が存在する場合の `TypeExpression` または `MissingTypeExpression` recovery を所有する |
| `SurfaceNodeKind::StatementItem` | なし | `SyntaxKind::StatementItem` | parser task 16 の一時 module-level statement host。現在実装済みの S-013 / S-014 statement 語彙に属する concrete parser-owned statement node を、直接または task 35 の `AnnotatedStatement` 経由でちょうど 1 個所有する |
| `SurfaceNodeKind::LetStatement` | なし | `SyntaxKind::LetStatement` | parser task 16/17 の `let` generalization。`let`、separator comma 付き qualified-variable segment、任意の `such` と `ConditionList`、任意の task-17 simple `JustificationClause`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::QualifiedVariableSegment` | なし | `SyntaxKind::QualifiedVariableSegment` | parser task 16 の statement-level variable segment。comma token で区切られた identifier token、任意の `be` / `being`、任意の `TypeExpression` または `MissingTypeExpression` recovery を所有する |
| `SurfaceNodeKind::AssumptionStatement` | なし | `SyntaxKind::AssumptionStatement` | parser task 16 の `assume` または `assume that`。`assume` と、1 個の `Proposition` または 1 個の `ConditionList`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::Proposition` | なし | `SyntaxKind::Proposition` | parser task 16 の proposition surface。任意の label identifier と colon、および 1 個の `FormulaExpression` または `MissingFormula` recovery を所有する |
| `SurfaceNodeKind::ConditionList` | なし | `SyntaxKind::ConditionList` | parser task 16 の statement-level condition。`that`、`and` token で区切られた 1 個以上の `Proposition` child、任意の recovery を所有する |
| `SurfaceNodeKind::GivenStatement` | なし | `SyntaxKind::GivenStatement` | parser task 16 の existential assumption。`given`、separator comma 付き qualified-variable segment、任意の `such` と `ConditionList`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::TakeStatement` | なし | `SyntaxKind::TakeStatement` | parser task 16 の witness introduction。`take`、comma token で区切られた 1 個以上の `Witness` child、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::Witness` | なし | `SyntaxKind::Witness` | parser task 16 の witness item。1 個の `TermExpression`、または identifier、`=`、`TermExpression` / `MissingTerm` recovery を所有する |
| `SurfaceNodeKind::SetStatement` | なし | `SyntaxKind::SetStatement` | parser task 16 の local constant definition。`set`、comma token で区切られた 1 個以上の `Equating` child、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::Equating` | なし | `SyntaxKind::Equating` | parser task 16 の equating item。identifier または `MissingTerm` recovery、存在する場合の `=`、`TermExpression` または `MissingTerm` recovery を所有する |
| `SurfaceNodeKind::ConsiderStatement` | なし | `SyntaxKind::ConsiderStatement` | parser task 18 の choice statement。`consider`、separator comma 付き qualified-variable segment、`such`、`ConditionList` または condition recovery、simple `JustificationClause` または missing-justification recovery、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::ReconsiderStatement` | なし | `SyntaxKind::ReconsiderStatement` | parser task 18 の type-changing statement。`reconsider`、separator comma 付き reconsider item、`as`、`TypeExpression` または `MissingTypeExpression`、simple `JustificationClause` または missing-justification recovery、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::ReconsiderItem` | なし | `SyntaxKind::ReconsiderItem` | parser task 18 の reconsider item。identifier、または identifier、`=`、`TermExpression` / `MissingTerm` recovery を所有する |
| `SurfaceNodeKind::ConclusionStatement` | なし | `SyntaxKind::ConclusionStatement` | parser task 19 の conclusion statement と parser task 22 の full proof justification。`thus` または `hence`、1 個の `Proposition`、任意の明示的 `JustificationClause` または `ProofBlock`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::ThenStatement` | なし | `SyntaxKind::ThenStatement` | parser task 19 の sequential modifier wrapper。`then` token と、ちょうど 1 個の linkable statement child または `MissingStatement` recovery を所有する |
| `SurfaceNodeKind::IterativeEqualityStatement` | なし | `SyntaxKind::IterativeEqualityStatement` | parser task 19 の equality chain。任意の label identifier / colon、最初の `TermExpression`、`=`、2 番目の `TermExpression`、任意の simple `JustificationClause`、1 個以上の `IterativeEqualityStep` child、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::IterativeEqualityStep` | なし | `SyntaxKind::IterativeEqualityStep` | parser task 19 の `.=` continuation。`.=` token、`TermExpression` または `MissingTerm`、任意の simple `JustificationClause` を所有する |
| `SurfaceNodeKind::NowStatement` | なし | `SyntaxKind::NowStatement` | parser task 20 の `now ... end;` reasoning block。任意の label identifier / colon、`now`、nested statement node、`MissingEnd` を含む任意の recovery、存在する場合の `end` と semicolon を所有する |
| `SurfaceNodeKind::HerebyStatement` | なし | `SyntaxKind::HerebyStatement` | parser task 20 の `hereby ... end;` diffuse conclusion block。`hereby`、nested statement node、`MissingEnd` を含む任意の recovery、存在する場合の `end` と semicolon を所有する |
| `SurfaceNodeKind::CaseReasoningStatement` | なし | `SyntaxKind::CaseReasoningStatement` | parser task 20 の `per cases` block。`per`、`cases`、任意の simple `JustificationClause`、header semicolon、および source-order かつ homogeneous な `CaseItem` child または source-order かつ homogeneous な `SupposeItem` child を所有する |
| `SurfaceNodeKind::CaseItem` | なし | `SyntaxKind::CaseItem` | parser task 20 の `case ... end;` branch。`case`、`Proposition` または `ConditionList`、header semicolon、nested statement node、`MissingEnd` を含む任意の recovery、存在する場合の `end` と semicolon を所有する |
| `SurfaceNodeKind::SupposeItem` | なし | `SyntaxKind::SupposeItem` | parser task 20 の `suppose ... end;` branch。`suppose`、`Proposition` または `ConditionList`、header semicolon、nested statement node、`MissingEnd` を含む任意の recovery、存在する場合の `end` と semicolon を所有する |
| `SurfaceNodeKind::InlineFunctorDefinition` | なし | `SyntaxKind::InlineFunctorDefinition` | parser task 21 の standalone `deffunc` definition。`deffunc`、name identifier または `MissingTerm` recovery、parameter parentheses、comma token で区切られた `TypedParameter` child 0 個以上、`->`、return `TypeExpression` または `MissingTypeExpression`、`equals`、body `TermExpression` または `MissingTerm`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::InlinePredicateDefinition` | なし | `SyntaxKind::InlinePredicateDefinition` | parser task 21 の standalone `defpred` definition。`defpred`、name identifier または `MissingTerm` recovery、parameter parentheses、comma token で区切られた `TypedParameter` child 0 個以上、`means`、body `FormulaExpression` または `MissingFormula`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::TypedParameter` | なし | `SyntaxKind::TypedParameter` | parser task 21 の inline-definition parameter。存在する場合の parameter identifier、書かれた場合の optional `be` または `being`、`TypeExpression` または `MissingTypeExpression` recovery を所有する |
| `SurfaceNodeKind::TheoremItem` | なし | `SyntaxKind::TheoremItem` | parser task 22 の theorem declaration。任意の leading `Annotation` node、任意の status token を構文的に保存し、`theorem`、label identifier または `MissingTerm`、`:`、`FormulaExpression` または `MissingFormula`、任意の `JustificationClause` または `ProofBlock`、任意の recovery、存在する場合の final semicolon を所有する |
| `SurfaceNodeKind::LemmaItem` | なし | `SyntaxKind::LemmaItem` | parser task 22 の lemma declaration。`lemma` role token で選択され、`TheoremItem` と同じ source-order children を所有する |
| `SurfaceNodeKind::ProofBlock` | なし | `SyntaxKind::ProofBlock` | parser task 22 の full proof justification block。`proof`、reasoning body の nested statement node、`MissingEnd` を含む任意の recovery、存在する場合の `end` を所有する。後続 semicolon は enclosing theorem または statement が所有する |
| `SurfaceNodeKind::DefinitionBlockItem` | なし | `SyntaxKind::DefinitionBlockItem` | parser task 23 の `definition ... end;` block item。任意の leading `Annotation` node、`definition`、source-order の concrete definition content または placeholder content、任意の `MissingEnd`、存在する場合の `end`、存在する場合の final semicolon を所有する |
| `SurfaceNodeKind::DefinitionParameter` | なし | `SyntaxKind::DefinitionParameter` | parser task 23 の通常 definition parameter。`let`、1 個以上の qualified-variable segment、任意の condition または justification、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::AttributeDefinition` | なし | `SyntaxKind::AttributeDefinition` | parser task 23 の `attr` definition。`attr`、label identifier または `MissingTerm`、`:`、subject identifier または `MissingTerm`、`is`、`AttributePattern`、`means`、`FormulaDefiniens`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::AttributePattern` | なし | `SyntaxKind::AttributePattern` | parser task 23 の attribute pattern head。任意の `ParameterPrefix` と identifier または user-symbol name、name 欠落時の `MissingTerm` を所有する |
| `SurfaceNodeKind::FormulaDefiniens` | なし | `SyntaxKind::FormulaDefiniens` | parser task 23 の formula definiens。1 個の `FormulaExpression`、または comma token で区切られた source-order `FormulaCase` child と任意の `otherwise FormulaExpression` を所有する |
| `SurfaceNodeKind::FormulaCase` | なし | `SyntaxKind::FormulaCase` | parser task 23 の conditional formula definiens case。value `FormulaExpression`、`if`、condition `FormulaExpression` を所有し、value または condition formula 欠落時は `MissingFormula` recovery を使う |
| `SurfaceNodeKind::CorrectnessCondition` | なし | `SyntaxKind::CorrectnessCondition` | parser task 23 の correctness condition。condition keyword、任意の general justification（`by`、`by computation`、または `proof`）、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::PredicateDefinition` | なし | `SyntaxKind::PredicateDefinition` | parser task 24 の `pred` definition。`pred`、label identifier または `MissingTerm`、`:`、raw `PredicatePattern`、`means`、`FormulaDefiniens`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::PredicatePattern` | なし | `SyntaxKind::PredicatePattern` | parser task 24 の predicate definition pattern。`pred_pattern`（`loci`、1 個の `def_predicate_symbol`、任意の `template_loci`、任意の trailing `loci`）として受理できる source-order raw pattern token と、grammar-shaped split が存在しない場合の `MissingTerm` recovery を所有する。どの identifier が predicate symbol かは encode しない |
| `SurfaceNodeKind::FunctorDefinition` | なし | `SyntaxKind::FunctorDefinition` | parser task 25 の `func` definition。`func`、label identifier または `MissingTerm`、`:`、raw `FunctorPattern`、`->`、return `TypeExpression` または `MissingTypeExpression`、`means` plus `FormulaDefiniens` または `equals` plus `TermDefiniens`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::FunctorPattern` | なし | `SyntaxKind::FunctorPattern` | parser task 25 の functor definition pattern。canonical single-symbol `func_pattern` または文書化済み two-symbol circumfix surface shape として受理できる source-order raw pattern token と、grammar-shaped split が存在しない場合の `MissingTerm` recovery を所有する。どの token が functor symbol かは encode しない |
| `SurfaceNodeKind::TermDefiniens` | なし | `SyntaxKind::TermDefiniens` | parser task 25 の term definiens。1 個の `TermExpression`、または comma token で区切られた source-order `TermCase` child と任意の `otherwise TermExpression` を所有する |
| `SurfaceNodeKind::TermCase` | なし | `SyntaxKind::TermCase` | parser task 25 の conditional term definiens case。value `TermExpression`、`if`、condition `FormulaExpression` を所有し、value または condition 欠落時は `MissingTerm` または `MissingFormula` recovery を使う |
| `SurfaceNodeKind::ModeDefinition` | なし | `SyntaxKind::ModeDefinition` | parser task 26 の `mode` definition。`mode`、label identifier または `MissingTerm`、`:`、raw `ModePattern`、`is`、body `TypeExpression` または `MissingTypeExpression`、存在する場合の最初の semicolon、任意の `ModeProperty` を所有する |
| `SurfaceNodeKind::ModePattern` | なし | `SyntaxKind::ModePattern` | parser task 26 の mode definition pattern。`mode_def_name [ type_params ]` として受理できる source-order raw token と、grammar-shaped split が存在しない場合の `MissingTerm` recovery を所有する。semantic parameter role は encode しない |
| `SurfaceNodeKind::ModeProperty` | なし | `SyntaxKind::ModeProperty` | parser task 26 の mode definition 直後の `sethood` property。`sethood`、存在する場合の必須 general justification（`JustificationClause` または `ProofBlock`）、任意の recovery、存在する場合の property semicolon を所有する |
| `SurfaceNodeKind::AttributeRedefinition` | なし | `SyntaxKind::AttributeRedefinition` | parser task 27 の `redefine attr`。`redefine`、`attr`、label identifier または `MissingTerm`、`:`、subject identifier または `MissingTerm`、`is`、`AttributePattern`、`means`、`FormulaDefiniens`、存在する場合の最初の semicolon、必須 `CoherenceCondition` を所有する |
| `SurfaceNodeKind::PredicateRedefinition` | なし | `SyntaxKind::PredicateRedefinition` | parser task 27 の `redefine pred`。`redefine`、`pred`、raw `PredicatePattern`、`means`、`FormulaDefiniens`、存在する場合の最初の semicolon、必須 `CoherenceCondition` を所有する |
| `SurfaceNodeKind::FunctorRedefinition` | なし | `SyntaxKind::FunctorRedefinition` | parser task 27 の `redefine func`。`redefine`、`func`、label identifier または `MissingTerm`、`:`、raw `FunctorPattern`、`->`、return `TypeExpression` または `MissingTypeExpression`、`means FormulaDefiniens` または `equals TermDefiniens`、存在する場合の最初の semicolon、必須 `CoherenceCondition` を所有する |
| `SurfaceNodeKind::CoherenceCondition` | なし | `SyntaxKind::CoherenceCondition` | parser task 27 の redefinition coherence tail。`coherence`、任意の `with` と label identifier または `MissingProofStep`、存在する場合の必須 general justification、任意の recovery、存在する場合の coherence semicolon を所有する |
| `SurfaceNodeKind::NotationAlias` | なし | `SyntaxKind::NotationAlias` | parser task 27 の `synonym` または `antonym` declaration。alias keyword、alternate `NotationPattern`、`for`、original `NotationPattern`、任意の recovery、存在する場合の final semicolon を所有する |
| `SurfaceNodeKind::NotationPattern` | なし | `SyntaxKind::NotationPattern` | parser task 27 の raw notation alias pattern。`for` の片側の source-order raw token と、空または delimitation 不能な場合の `MissingTerm` recovery を所有する。pattern を predicate、functor、mode、attribute のいずれかとして分類しない |
| `SurfaceNodeKind::PropertyClause` | なし | `SyntaxKind::PropertyClause` | parser task 28 の property item。canonical な predicate / functor property keyword または standalone `sethood`、存在する場合の必須 general justification（`JustificationClause` または `ProofBlock`）、任意の recovery、存在する場合の property semicolon を所有する |
| `SurfaceNodeKind::StructureDefinition` | なし | `SyntaxKind::StructureDefinition` | parser task 29 の `struct` definition。`struct`、raw `StructurePattern`、`where`、1 個以上の `StructureField` / `StructureProperty` member、`end`、存在する場合の final semicolon を所有する |
| `SurfaceNodeKind::StructurePattern` | なし | `SyntaxKind::StructurePattern` | parser task 29 の structure definition pattern。`struct_def_name [ type_params ]` の source-order raw token と、pattern が空または malformed な場合の `MissingTerm` recovery を所有する。semantic structure identity は解決しない |
| `SurfaceNodeKind::StructureField` | なし | `SyntaxKind::StructureField` | parser task 29 の structure field member。`field`、field identifier または `MissingTerm`、`->`、`TypeExpression` または `MissingTypeExpression`、任意の `:= TermExpression`、存在する場合の member semicolon を所有する |
| `SurfaceNodeKind::StructureProperty` | なし | `SyntaxKind::StructureProperty` | parser task 29 の structure property member。`property`、property identifier または `MissingTerm`、`->`、`TypeExpression` または `MissingTypeExpression`、存在する場合の member semicolon を所有する |
| `SurfaceNodeKind::InheritanceDefinition` | なし | `SyntaxKind::InheritanceDefinition` | parser task 29 の inheritance definition。`inherit`、`extends` の前後の child / parent `InheritanceTarget`、任意の explicit `where ... end` member block、存在する場合の final semicolon を所有する |
| `SurfaceNodeKind::InheritanceTarget` | なし | `SyntaxKind::InheritanceTarget` | parser task 29 の raw inheritance child / parent target。structure-like reference と任意の raw type argument、または parent `set` token を source order で保持し、semantic structure identity は解決しない |
| `SurfaceNodeKind::FieldRedefinition` | なし | `SyntaxKind::FieldRedefinition` | parser task 29 の explicit inheritance field mapping。`field`、child field identifier または `MissingTerm`、任意の narrowed `-> TypeExpression`、必須 `from`、存在する場合の identifier または `it` source、member semicolon を所有する |
| `SurfaceNodeKind::PropertyRedefinition` | なし | `SyntaxKind::PropertyRedefinition` | parser task 29 の explicit inheritance property mapping。`property`、child property identifier または `MissingTerm`、任意の narrowed `-> TypeExpression`、必須 `from`、存在する場合の identifier source、member semicolon を所有する |
| `SurfaceNodeKind::RegistrationBlockItem` | なし | `SyntaxKind::RegistrationBlockItem` | parser task 30 の `registration ... end;` block item。任意の leading `Annotation` node、`registration`、source-order registration content、任意の recovery、`MissingEnd` または存在する場合の `end`、存在する場合の final semicolon を所有する |
| `SurfaceNodeKind::RegistrationParameter` | なし | `SyntaxKind::RegistrationParameter` | parser task 30 の registration-local `let` parameter。qualified variable segment、`TypeExpression` または recovery、任意の condition list と syntax-level `by` reference、存在する場合の parameter semicolon を所有する |
| `SurfaceNodeKind::ExistentialRegistration` | なし | `SyntaxKind::ExistentialRegistration` | parser task 30 の existential cluster registration。`cluster`、label、colon、attributed `TypeExpression`、header semicolon、`existence` correctness condition を所有する |
| `SurfaceNodeKind::ConditionalRegistration` | なし | `SyntaxKind::ConditionalRegistration` | parser task 30 の conditional cluster registration。antecedent registration adjective、`->`、consequent registration adjective、`for`、target `TypeExpression`、header semicolon、`coherence` correctness condition を所有する |
| `SurfaceNodeKind::FunctorialRegistration` | なし | `SyntaxKind::FunctorialRegistration` | parser task 30 の functorial cluster registration。unambiguous な application / operator / bracket payload term、`->`、consequent registration adjective、`for`、target `TypeExpression`、header semicolon、`coherence` correctness condition を所有する |
| `SurfaceNodeKind::ReductionRegistration` | なし | `SyntaxKind::ReductionRegistration` | parser task 30 の reduction registration。`reduce`、label、colon、left `TermExpression`、`to`、right `TermExpression`、header semicolon、`reducibility` correctness condition を所有する |
| `SurfaceNodeKind::TemplateParameter` | なし | `SyntaxKind::TemplateParameter` | parser task 31 の template block 先頭 `let` parameter。通常 value / type / predicate / functor parameter token、任意の constraint または `by` / proof tail、存在する場合の parameter semicolon を所有する |
| `SurfaceNodeKind::TemplateLoci` | なし | `SyntaxKind::TemplateLoci` | parser task 31 の predicate / functor pattern 側 `[` locus list `]`。delimiter、comma token、`TemplateLocus` child または missing-locus recovery を所有する |
| `SurfaceNodeKind::TemplateLocus` | なし | `SyntaxKind::TemplateLocus` | parser task 31 の pattern 側 template locus identifier または missing-term recovery |
| `SurfaceNodeKind::TemplateArguments` | なし | `SyntaxKind::TemplateArguments` | parser task 31 の call / reference 側 `[` template argument list `]`。delimiter、comma token、`TemplateArgument` child または recovery を所有する |
| `SurfaceNodeKind::TemplateArgument` | なし | `SyntaxKind::TemplateArgument` | parser task 31 の単一 template actual。`TypeExpression`、`TermExpression` / `QuaExpression`、または missing-type recovery を包む |
| `SurfaceNodeKind::AlgorithmDefinition` | なし | `SyntaxKind::AlgorithmDefinition` | parser task 32 の `algorithm name [template_loci] (parameters) [-> type_expression] do ... end;` definition content。task 34 により、任意の leading `AlgorithmTerminationClause` と `AlgorithmBody` 前の ordered header verification clause も所有する |
| `SurfaceNodeKind::AlgorithmParameters` | なし | `SyntaxKind::AlgorithmParameters` | parser task 32 の algorithm formal list。`(`、comma-separated identifier token または missing-term recovery、`)` / delimiter recovery を所有する |
| `SurfaceNodeKind::AlgorithmBody` | なし | `SyntaxKind::AlgorithmBody` | parser task 32 の `do ... end` algorithm body。存在する場合の `do` token、`AlgorithmStatementList`、`end` / missing-end recovery を所有する |
| `SurfaceNodeKind::AlgorithmStatementList` | なし | `SyntaxKind::AlgorithmStatementList` | parser task 32 の body statement list。task 33 により control-flow statement、task 34 により `AssertStatement`、task 35 により `AnnotatedAlgorithmStatement` wrapper も source order で所有する |
| `SurfaceNodeKind::VariableDeclaration` | なし | `SyntaxKind::VariableDeclaration` | parser task 32 の `var` / `const` declaration。任意で `ghost` prefix を持つ。1 個以上の `VariableBinding`、任意の shared `as TypeExpression`、任意の syntax-level justification、semicolon または recovery を所有する |
| `SurfaceNodeKind::VariableBinding` | なし | `SyntaxKind::VariableBinding` | parser task 32 の declaration binding。binding identifier と任意の `:= TermExpression` initializer または missing-term recovery を所有する |
| `SurfaceNodeKind::AssignmentStatement` | なし | `SyntaxKind::AssignmentStatement` | parser task 32 の assignment statement。任意で `ghost` prefix を持つ。`Lvalue`、`:=`、代入 `TermExpression` または recovery、semicolon または recovery を所有する |
| `SurfaceNodeKind::Lvalue` | なし | `SyntaxKind::Lvalue` | parser task 32 の syntactic assignment target。selector と namespace の役割解決なしに identifier と任意の dotted identifier segment を所有する |
| `SurfaceNodeKind::SnapshotStatement` | なし | `SyntaxKind::SnapshotStatement` | parser task 32 の `snapshot` statement。snapshot identifier と semicolon または recovery を所有する |
| `SurfaceNodeKind::ReturnStatement` | なし | `SyntaxKind::ReturnStatement` | parser task 32 の `return` statement。任意の returned `TermExpression`、任意の syntax-level justification、semicolon または recovery を所有する |
| `SurfaceNodeKind::ClaimBlockItem` | なし | `SyntaxKind::ClaimBlockItem` | parser task 32 の top-level `claim algorithm_name do ... end;` item。`claim` の前の任意の leading `Annotation` node と、theorem / lemma child を直接または task 35 の `Annotation` / `LibraryAnnotation` prefix 経由で所有する |
| `SurfaceNodeKind::IfStatement` | なし | `SyntaxKind::IfStatement` | parser task 33 の algorithm conditional。`if`、`FormulaExpression` または missing-formula recovery、`do`、then 側 `AlgorithmStatementList`、および `end;` / `else` + nested `IfStatement` / `else` + else 側 `AlgorithmStatementList` + `end;` を所有する |
| `SurfaceNodeKind::WhileStatement` | なし | `SyntaxKind::WhileStatement` | parser task 33 の algorithm while loop。task 34 により `do` と body list の間に leading `LoopInvariantClause` / `LoopDecreasingClause` child も所有する |
| `SurfaceNodeKind::ForRangeStatement` | なし | `SyntaxKind::ForRangeStatement` | parser task 33 の algorithm range loop。task 34 により `do` と body list の間に leading `LoopInvariantClause` child も所有する。`for` loop の `decreasing` は引き続き reject する |
| `SurfaceNodeKind::ForCollectionStatement` | なし | `SyntaxKind::ForCollectionStatement` | parser task 33 の algorithm collection loop。task 34 により `do` と body list の間に leading `LoopInvariantClause` child も所有する。`for` loop の `decreasing` は引き続き reject する |
| `SurfaceNodeKind::MatchStatement` | なし | `SyntaxKind::MatchStatement` | parser task 33 の algorithm structural match。`match`、scrutinee `TermExpression`、`do`、1 個以上の `MatchCase` または missing-statement recovery、`MatchEnding`、final `end;` / recovery を所有する |
| `SurfaceNodeKind::MatchCase` | なし | `SyntaxKind::MatchCase` | parser task 33 の `case term_pattern do ... end;` branch。pattern `TermExpression`、branch statement list、branch `end;` / recovery を所有する |
| `SurfaceNodeKind::MatchEnding` | なし | `SyntaxKind::MatchEnding` | parser task 33 の match ending。`otherwise` + statement list + `end;`、または `exhaustive` + 任意の syntax-level justification + semicolon を所有する |
| `SurfaceNodeKind::BreakStatement` | なし | `SyntaxKind::BreakStatement` | parser task 33 の `break;` jump statement。loop context validity は意味論で扱い、ここには符号化しない |
| `SurfaceNodeKind::ContinueStatement` | なし | `SyntaxKind::ContinueStatement` | parser task 33 の `continue;` jump statement。loop context と termination obligation は意味論で扱い、ここには符号化しない |
| `SurfaceNodeKind::AlgorithmTerminationClause` | なし | `SyntaxKind::AlgorithmTerminationClause` | parser task 34 の `algorithm` 前 `terminating` modifier。termination proof obligation は意味論で扱い、ここには符号化しない |
| `SurfaceNodeKind::AlgorithmRequiresClause` | なし | `SyntaxKind::AlgorithmRequiresClause` | parser task 34 の header `requires FormulaExpression` clause |
| `SurfaceNodeKind::AlgorithmEnsuresClause` | なし | `SyntaxKind::AlgorithmEnsuresClause` | parser task 34 の header `ensures FormulaExpression` clause |
| `SurfaceNodeKind::AlgorithmDecreasingClause` | なし | `SyntaxKind::AlgorithmDecreasingClause` | parser task 34 の header `decreasing TermList` clause |
| `SurfaceNodeKind::LoopInvariantClause` | なし | `SyntaxKind::LoopInvariantClause` | parser task 34 の leading loop `invariant FormulaExpression [JustificationClause];` clause |
| `SurfaceNodeKind::LoopDecreasingClause` | なし | `SyntaxKind::LoopDecreasingClause` | parser task 34 の leading while-loop `decreasing TermList [JustificationClause];` clause |
| `SurfaceNodeKind::AssertStatement` | なし | `SyntaxKind::AssertStatement` | parser task 34 の algorithm assertion statement。`assert`、`FormulaExpression` または missing-formula recovery、任意の syntax-level justification、semicolon / recovery を所有する |
| `SurfaceNodeKind::TermList` | なし | `SyntaxKind::TermList` | parser task 34 の decreasing measure 用 comma-separated list。1 個以上の `TermExpression` child、comma token、空 slot の missing-term recovery を所有する |
| `SurfaceNodeKind::Annotation` | なし | `SyntaxKind::Annotation` | parser task 35 の attachable annotation prefix。固定 `@identifier` marker と任意の `AnnotationArgumentList` / `ProofHintOptionList`、または nested `LibraryAnnotation` を所有する |
| `SurfaceNodeKind::LibraryAnnotation` | なし | `SyntaxKind::LibraryAnnotation` | parser task 35 の `@[ annotation_label_list ]` prefix。`]` 欠落時は delimiter recovery を持つ |
| `SurfaceNodeKind::AnnotationLabelList` | なし | `SyntaxKind::AnnotationLabelList` | parser task 35 の comma-separated library annotation label list。malformed slot は annotation recovery として保持する |
| `SurfaceNodeKind::AnnotationLabel` | なし | `SyntaxKind::AnnotationLabel` | parser task 35 の単一 library annotation label。identifier marker と任意の `AnnotationArgumentList` を所有する |
| `SurfaceNodeKind::AnnotationArgumentList` | なし | `SyntaxKind::AnnotationArgumentList` | parser task 35 の parenthesized annotation argument list。comma token、missing-argument recovery、delimiter recovery を所有する |
| `SurfaceNodeKind::AnnotationArgument` | なし | `SyntaxKind::AnnotationArgument` | parser task 35 の単一 annotation argument。identifier、numeral、string literal、または recovered malformed source を意味評価なしで保持する |
| `SurfaceNodeKind::ProofHintOptionList` | なし | `SyntaxKind::ProofHintOptionList` | parser task 35 の `@proof_hint(...)` option list。comma-separated `ProofHintOption` child を所有する |
| `SurfaceNodeKind::ProofHintOption` | なし | `SyntaxKind::ProofHintOption` | parser task 35 の proof-hint option。Appendix A.21 の option name、colon、要求される numeral または solver-name value を所有し、malformed option recovery を持てる |
| `SurfaceNodeKind::StandaloneDiagnosticAnnotation` | なし | `SyntaxKind::StandaloneDiagnosticAnnotation` | parser task 35 の standalone `@show_type(...)` / `@eval(...)` diagnostic directive。source syntax として parse され、後続 statement には attach しない |
| `SurfaceNodeKind::AnnotatedStatement` | なし | `SyntaxKind::AnnotatedStatement` | parser task 35 の ordinary statement wrapper。1 個以上の leading `Annotation` node と、それに続く concrete statement node を所有する |
| `SurfaceNodeKind::AnnotatedAlgorithmStatement` | なし | `SyntaxKind::AnnotatedAlgorithmStatement` | parser task 35 の algorithm statement wrapper。1 個以上の leading `Annotation` node と、それに続く concrete algorithm statement node を所有する |
| `SurfaceNodeKind::AnnotatedDefinitionContent` | なし | `SyntaxKind::AnnotatedDefinitionContent` | parser task 35 の definition-content wrapper。1 個以上の leading `Annotation` node と、それに続く concrete definition content node を所有する |
| `SurfaceNodeKind::AnnotatedRegistrationContent` | なし | `SyntaxKind::AnnotatedRegistrationContent` | parser task 35 の registration-content wrapper。1 個以上の leading `Annotation` node と、それに続く concrete registration content node を所有する |
| `SurfaceNodeKind::CompactStatement` | なし | `SyntaxKind::CompactStatement` | parser task 17 の最小の明示的 justification 付き compact statement host と parser task 22 の proof justification host。1 個の `Proposition`、1 個の `JustificationClause` または `ProofBlock`、任意の recovery、任意の semicolon を所有する |
| `SurfaceNodeKind::JustificationClause` | なし | `SyntaxKind::JustificationClause` | parser task 17 の `by` clause。`by` token と、通常 citation 用の `ReferenceList` または `by computation(...)` 用の `ComputationJustification` を所有する |
| `SurfaceNodeKind::ReferenceList` | なし | `SyntaxKind::ReferenceList` | parser task 17 の source-order citation list。comma token で区切られた citation node を所有する |
| `SurfaceNodeKind::Reference` | なし | `SyntaxKind::Reference` | parser task 17 の local citation。1 個の identifier token と任意の task 31 `TemplateArguments` を所有する |
| `SurfaceNodeKind::QualifiedReference` | なし | `SyntaxKind::QualifiedReference` | parser task 17 の namespace-qualified citation。`NamespacePath`、最後の dot token、最後の identifier token、任意の task 31 `TemplateArguments` を所有する |
| `SurfaceNodeKind::GroupedReference` | なし | `SyntaxKind::GroupedReference` | parser task 17 の grouped citation。`NamespacePath`、`.{`、comma token で区切られた grouped item、任意の delimiter recovery、任意の `}` を所有する |
| `SurfaceNodeKind::GroupedReferenceItem` | なし | `SyntaxKind::GroupedReferenceItem` | parser task 17 の grouped citation member。1 個の identifier token と任意の task 31 `TemplateArguments` を所有する |
| `SurfaceNodeKind::BulkReference` | なし | `SyntaxKind::BulkReference` | parser task 17 の bulk citation。`NamespacePath` と compound `.*` token を所有する |
| `SurfaceNodeKind::ComputationJustification` | なし | `SyntaxKind::ComputationJustification` | parser task 17 の computation proof payload。`computation` token と任意の parenthesized computation-option list を所有する |
| `SurfaceNodeKind::ComputationOption` | なし | `SyntaxKind::ComputationOption` | parser task 17 の computation option。`steps`、`timeout`、`nest` のいずれか、colon token、numeral token または `MissingProofStep` recovery を所有する |
| `SurfaceNodeKind::SelectorAccess` | なし | `SyntaxKind::SelectorAccess` | parser task 10 の postfix selector access または selector-call surface。syntax-only dot role を保持する |
| `SurfaceNodeKind::StructureUpdate` | なし | `SyntaxKind::StructureUpdate` | parser task 10 の functional `term "with" "(" field_update_list ")"` update surface |
| `SurfaceNodeKind::FieldUpdate` | なし | `SyntaxKind::FieldUpdate` | parser task 10 の、`StructureUpdate` 内の `selector ":=" term_expression` field update |
| `SurfaceNodeKind::QuaExpression` | なし | `SyntaxKind::QuaExpression` | parser task 11 の type qualification。child order は base term-shape、`qua` token、`TypeExpression` または `MissingTypeExpression` recovery |
| `SurfaceNodeKind::ModulePath` | なし | `SyntaxKind::ModulePath` | `module_path`。任意の `RelativePrefix`、最初の `PathSegment`、続く `.` token + `PathSegment` の反復。この path 形だけが `RelativePrefix` を持てる |
| `SurfaceNodeKind::NamespacePath` | なし | `SyntaxKind::NamespacePath` | `namespace_path`。最初の `PathSegment`、続く `.` token + identifier `PathSegment` の反復。相対 prefix は許さない |
| `SurfaceNodeKind::QualifiedSymbol` | なし | `SyntaxKind::QualifiedSymbol` | `qualified_symbol`。0 個以上の namespace identifier `PathSegment` + `.` token の組に最後の user-symbol `PathSegment` が続く形、または task 8 の attribute-ref flattening として、最後の user-symbol の前に user-symbol token の dotted prefix `PathSegment` も許す形 |
| `SurfaceNodeKind::PathSegment` | なし | `SyntaxKind::PathSegment` | identifier または user-symbol token を 1 つだけ包む。役割は親と token kind で決まる |
| `SurfaceNodeKind::RelativePrefix` | なし | `SyntaxKind::RelativePrefix` | `ModulePath` 先頭の `.` または `..` token を 1 つだけ包む |
| `SurfaceNodeKind::InfixExpression(SurfaceInfixOperator)` | spelling、precedence、associativity | `SyntaxKind::InfixExpression` | task 12 の infix Pratt expression 形状 |
| `SurfaceNodeKind::PrefixExpression(SurfacePrefixOperator)` | spelling、precedence | `SyntaxKind::PrefixExpression` | task 12 の prefix Pratt expression 形状 |
| `SurfaceNodeKind::PostfixExpression(SurfacePostfixOperator)` | spelling、precedence | `SyntaxKind::PostfixExpression` | task 12 の postfix Pratt expression 形状 |
| `SurfaceNodeKind::FormulaExpression` | なし | `SyntaxKind::FormulaExpression` | parser task 13/14 の formula wrapper。atomic formula、connective、quantifier、parenthesized formula、formula constant を含む formula child を 1 つだけ所有する |
| `SurfaceNodeKind::BuiltinPredicateApplication` | なし | `SyntaxKind::BuiltinPredicateApplication` | parser task 13 の built-in `in`、`=`、`<>` predicate。left term、predicate token、right term または missing-term recovery を所有する |
| `SurfaceNodeKind::IsAssertion` | なし | `SyntaxKind::IsAssertion` | parser task 13 の generic `is` assertion。subject term、`is`、任意の `not`、resolver classification を持たない type/body child を所有する |
| `SurfaceNodeKind::AttributeTestChain` | なし | `SyntaxKind::AttributeTestChain` | parser task 13 の attribute-only assertion body。1 個以上の task-8 `AttributeRef` child を所有する |
| `SurfaceNodeKind::PredicateApplication` | なし | `SyntaxKind::PredicateApplication` | parser task 13 の syntax-only user predicate application。1 個以上の predicate segment を所有する |
| `SurfaceNodeKind::PredicateSegment` | なし | `SyntaxKind::PredicateSegment` | parser task 13 の user predicate segment。任意の term-list child、任意の negation token、1 個の predicate head、任意の right term-list child を所有する |
| `SurfaceNodeKind::PredicateHead` | なし | `SyntaxKind::PredicateHead` | parser task 13 の predicate symbol wrapper。active `QualifiedSymbol` または template-local identifier と、任意の task 31 `TemplateArguments` を所有する |
| `SurfaceNodeKind::InlinePredicateApplication` | なし | `SyntaxKind::InlinePredicateApplication` | parser task 13 の identifier head と parenthesized term argument を持つ inline predicate call shape |
| `SurfaceNodeKind::PrefixFormula(SurfaceFormulaPrefixOperator)` | operator | `SyntaxKind::PrefixFormula` | parser task 14 の fixed formula prefix。現在は `not` |
| `SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator)` | `connective: SurfaceFormulaConnective`、`repeated: bool` | `SyntaxKind::BinaryFormula` | parser task 14 の `&`、`or`、`implies`、`iff` fixed binary connective formula。token-preserving repetition form を含む |
| `SurfaceNodeKind::ParenthesizedFormula` | なし | `SyntaxKind::ParenthesizedFormula` | parser task 14 の formula grouping。`(`、nested `FormulaExpression`、`)` または delimiter recovery を所有する |
| `SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind)` | quantifier | `SyntaxKind::QuantifiedFormula` | parser task 14 の universal/existential quantifier surface。quantifier token、variable segment、optional condition/body separator、formula body child を所有する |
| `SurfaceNodeKind::QuantifierVariableSegment` | なし | `SyntaxKind::QuantifierVariableSegment` | parser task 14 の quantified variable segment。variable identifier/comma token、optional `be`/`being`、optional `TypeExpression` を所有する |
| `SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant)` | constant | `SyntaxKind::FormulaConstant` | parser task 14 の `thesis` / `contradiction` formula constant |
| `SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind)` | recovery kind | `SyntaxKind::ErrorRecovery` | builder が作る recovery node は recovered |

`SurfaceTokenKind` は、上に挙げた token raw kind に対応する現在の語彙として
`Identifier`、`ReservedWord`、`ReservedSymbol`、`Numeral`、`LexemeRun`、
`UserSymbol`、`AnnotationMarker`、`StringLiteral`、`ErrorRecovery`、`Unknown`
を持つ。
`SurfaceOperatorAssociativity` は現在 `Left`、`Right`、`NonAssociative` を持つ。

`mizar-parser` task 4 のために追加された共有 path node は syntax-only の形である。
node range は、その path または wrapper が所有する最初の token から最後の token
までとする。親 path node は子を source order で列挙する。segment 間の separator
`.` token は `PathSegment` で包まず、親 path node の直接 child とする。これらの
node 自体は recovery node や trivia entry を生成しない。missing path diagnostic、
skipped-token trivia、doc-comment attachment は消費側の文法タスクが所有する。
`SurfaceNodeView` は `as_module_path`、`as_namespace_path`、`as_qualified_symbol`、
`as_path_segment`、`as_relative_prefix` の typed helper を公開し、consumer がこれらの
共有 path 形のために生の rowan traversal を使わずに済むようにする。

`mizar-parser` task 5 で追加された module skeleton node は syntax-only の形である。
`CompilationUnit` は source file surface を表し、`ItemList` child をちょうど 1 つ
所有する。`ItemList` の child は source order の concrete item node、
`PlaceholderItem` node、`SkippedToken` のような item-level recovery node である。
`PlaceholderItem` は top-level item boundary 1 つとして消費された source token を包み、
annotation prefix や終端セミコロンを欠いた回復済み item も含める。parser はこれらの
node に import resolution、visibility semantics、theorem validity、symbol identity を
encode してはならない。
`SurfaceNodeView` は `as_compilation_unit`、`as_item_list`、
`as_placeholder_item` の typed helper を公開する。後続 item への leading
doc-comment attachment は、comment text を item node にコピーせず、`SurfaceTrivia`
で表現する。

`mizar-parser` task 6 で追加された import node は syntax-only の形である。
`ImportItem` は import prelude が開いている間の `import_stmt` 1 つを表す。well-formed
input では、child は source order で、`import` token、comma token で区切られた
1 個以上の `ImportAliasDecl` または `ModuleBranchImport` node、semicolon token である。
malformed recovery では、`import` の後に declaration を持たない `ImportItem`、
後続 declaration のない trailing comma、または semicolon の前に消費した malformed
source に対する `SkippedToken` recovery child が現れ得る。`ImportAliasDecl` は
import される `ModulePath`、任意の `as` token、任意の alias `PathSegment` を所有する。
不正 alias では `MalformedImport` diagnostic を伴い、alias segment が欠けてよく、
nested `SkippedToken` recovery を含み得る。`ModuleBranchImport` は well-formed input
では base `ModulePath`、`.{` token、comma token で区切られた branch `PathSegment`
child、`}` を所有する。不正 branch import では `MalformedImport` を伴い、branch
segment または close token が欠けてよく、nested `SkippedToken` recovery を含み得る。
これらの node は relative `ModulePath` prefix を含み得るが、module resolution、
branch import の semantic import への分割、export availability 検査、alias 割り当ては
行わない。`SurfaceNodeView` は `as_import_item`、`as_import_alias_decl`、
`as_module_branch_import` の typed helper を公開する。

`mizar-parser` task 7 で追加された export / visibility node は syntax-only の形である。
`ExportItem` は export prelude が開いている間の `export_stmt` 1 つを表す。well-formed
input では、child は source order で、`export` token、comma token で区切られた
1 個以上の `ModulePath` node、semicolon token である。malformed recovery では、
`export` の後に path を持たない `ExportItem`、後続 path のない trailing comma、
または semicolon の前に消費した malformed source に対する nested `SkippedToken`
recovery child が現れ得る。`VisibilityMarker` は `private` または `public` token を
1 つだけ包む。`VisibleItem` は Chapter 12 が許す theorem / notation form 上の
top-level visibility prefix を表す。child は source order で、parsed `Annotation`
prefix node があればそれら、1 個の `VisibilityMarker`、target item node である。表現済み
theorem / lemma target は concrete な `TheoremItem` / `LemmaItem` node を使う。
notation target と短い legacy theorem fragment は `PlaceholderItem` target のままである。
task 31 は template predicate argument を持つ theorem payload を、周辺 theorem shape が
表現可能な場合には represented formula host として parse する。duplicate visibility
marker、dangling marker、非 theorem/notation top-level declaration の前の visibility では、
`MalformedVisibility` を伴って nested `SkippedToken` recovery child と任意の semicolon
token を含み得る。これらの node は public/private semantics、export availability、
symbol identity、theorem validity、notation validity を判断しない。`SurfaceNodeView` は
`as_export_item`、`as_visibility_marker`、`as_visible_item` の typed helper を公開する。

`mizar-parser` task 8 で追加する type-expression node は syntax-only の形である。
`ReserveItem` は、現在 frontend から到達可能な `TypeExpression` の host である。
これは top-level `reserve_decl` だけを表し、local statement-level `reserve` semantics は
実装しない。`ReserveSegment` は identifier-list の comma、`for` token、後続の
type expression を保持する。`TypeExpression` は、任意の non-empty `AttributeChain` と
必須の `TypeHead` という surface split を保持する。mode / radix / attribute の分類は
active environment に依存するため、`TypeHead` は意図的に generic である。builtin
`object` / `set` または `QualifiedSymbol` と任意の `TypeArguments` を所有するが、それが
mode、structure、radix type のいずれかは記録しない。parser は semantic lookup ではなく、
右端に残る syntactic type-head candidate を `TypeHead` として確保することで
attribute/head boundary を決める。

`AttributeRef` は 1 個の attribute occurrence の source-order syntax を所有する。
任意の `non`、任意の `ParameterPrefix`、1 個の syntactic `QualifiedSymbol`、任意の
parenthesized term argument を含む。struct-qualified attribute spelling は
同じ dotted `QualifiedSymbol` surface として保持する。その attribute-ref context では、
prefix `PathSegment` が namespace identifier だけでなく user-symbol token を包んでもよい。
どの prefix segment が structure であるかは AST では判断しない。`ParameterPrefix` は task 8 が attribute reference の前で
局所的に見える token split だけを保持する。つまり、1 個の identifier または numeral と
`-`、または parenthesized identifier / numeral list と `-` である。template-parameter scope の
妥当性は検証せず、`n-dimensional` のような名前に対する完全な contextual whole-spelling split
も行わない。この source drift は parameter-scope facts を lexing / parsing に渡す将来 task の
責務として残す。

`TypeArguments` は `of` / `over` token と comma-separated term arguments、または
`[`、comma-separated type-template arguments、任意の `]` を所有する。parser task 9 以降、
`of`/`over` と attribute argument list は task 8 の `TermPlaceholder` child ではなく、
concrete `TermExpression` child を使う。type expression として parse できる bracket
argument は nested `TypeExpression` として表す。parser task 11 以降、Appendix A の
`qua_arg` syntax を使う bracket argument は `TermExpression` child として表し、その
term-shape は identifier の `TermReference` または left-nested `QuaExpression` chain である。
この fallback は通常の term parser より狭く、identifier-shaped `qua_arg` からだけ始まり、
各 `qua` target は radix-type 形の `TypeExpression` として parse する。
bracket close が欠ける場合は
`TypeArguments` node の下で
`MalformedTypeExpression` と `UnmatchedOpeningDelimiter` recovery を使う。
`TermPlaceholder` は task 8 の legacy vocabulary としてのみ残り、task 11 の
bracket `qua_arg` parser path では生成しない。term classification、operator fact、
name resolution を encode してはならない。
`SurfaceNodeView` は
`as_reserve_item`、`as_reserve_segment`、`as_type_expression`、`as_attribute_chain`、
`as_attribute_ref`、`as_parameter_prefix`、`as_type_head`、`as_type_arguments`、
`as_term_placeholder` の typed helper を公開する。

`mizar-parser` task 9 で追加される primary term node は syntax-only の形である。
`TermExpression` は現在、1 個の term-shape child を包む wrapper である。parser task 9 と
10 では primary term または postfix chain を置くことができ、parser task 11 では
`QuaExpression` を置くことができる。後続 operator task が operator expression を置いても
wrapper role は変わらない。
`TermReference` は identifier token 1 個または共有 `QualifiedSymbol` 1 個を所有し、
term-position reference を name resolution や functor classification なしで保持する。
`NumeralTerm` と `ItTerm` は対応する token 1 個を包む。`ParenthesizedTerm` は delimiter
token と nested `TermExpression` または `MissingTerm` recovery を所有する。`ChoiceTerm` は
`the` token と nested `TypeExpression`、または type operand 欠落時の
`MissingTypeExpression` recovery を所有する。

`ApplicationTerm` は task 9 では意図的に generic である。ordinary parenthesized application と
reserved `[` / `]` bracket functor form を保持するが、arity、overload selection、active
user-symbol bracket-pair metadata は encode しない。ordinary application の child order は
callee の `TermReference` または `QualifiedSymbol`、`(` token、comma token で区切られた 0 個以上の
`TermExpression` argument、任意の `)` である。reserved bracket application の child order は
`[`、comma token で区切られた 0 個以上の `TermExpression` argument、任意の `]` であり、
delimiter pair 自体が syntax-only head なので callee child を持たない。`StructureConstructor`
は named field argument が syntactically に見える場合だけ出力し、child order は constructor
`QualifiedSymbol`、任意の `TypeArguments`、`(` token、comma token で区切られた
`FieldArgument` children、任意の `)` である。`S()` のような ambiguous zero-field form は、
将来の semantic boundary が structure fact を渡すまで generic `ApplicationTerm` として残す。
`FieldArgument` は field identifier、`:` token、`TermExpression` または `MissingTerm` を
所有する。`SetEnumeration` は `{`、comma token で区切られた source-order term arguments、
任意の `}` を所有する。`SetComprehension` は `{`、mapper `TermExpression`、`where`、
comma token で区切られた 1 個以上の `ComprehensionVariableSegment`、任意の `:` と
`FormulaExpression`、任意の `}` を所有する。`}` 欠落は `UnmatchedOpeningDelimiter`
recovery で表せる。`ComprehensionVariableSegment` は generator identifier、または
identifier 位置の `MissingTerm` recovery、存在する場合の `is` token、そして `is` token が
存在する場合の `TypeExpression` または `MissingTypeExpression` recovery を所有する。
これらの comprehension node は binder identity、sethood、capture、elaborated
Fraenkel symbol を解決しない。
`SurfaceNodeView` は `as_term_expression`、`as_term_reference`、`as_numeral_term`、
`as_it_term`、`as_parenthesized_term`、`as_choice_term`、`as_application_term`、
`as_structure_constructor`、`as_field_argument`、`as_set_enumeration`、
`as_set_comprehension`、`as_comprehension_variable_segment` の typed helper を公開する。

Parser task 10 は dot-role surface を syntax-only に保つ。`SelectorAccess` は
base term-shape child、`.` token、identifier field token、任意の call delimiter と
comma token で区切られた source-order `TermExpression` argument を所有する。
selector chain は left-associative に nest する。`StructureUpdate` は base term-shape
child、`with` token、`(`、comma token で区切られた `FieldUpdate` children、任意の `)` を
所有する。`FieldUpdate` は identifier selector path（identifier、反復する `.` token +
identifier）、`:=` token、`TermExpression` または `MissingTerm` recovery を所有する。
これらの node は scope を使って selector-versus-namespace role を判断せず、standalone
statement / algorithm assignment も表現しない。`SurfaceNodeView` は
`as_selector_access`、`as_structure_update`、`as_field_update` の typed helper を公開する。

Parser task 11 は `term qua type_expression` のために `QuaExpression` を追加する。
parser は selector/update postfix chain を `QuaExpression` より先に形成するため、
`p.x qua T` は selector result を修飾する。`qua` は現在の term precedence で最も低いため、
修飾済み term の後に selector を置くには `(p qua T).x` のように括弧が必要である。
`qua` chain は left-associative に nest し、`x qua T qua U` は `x qua T` の
`QuaExpression` を base とする `QuaExpression` として表す。`qua` target は
`TypeExpression` である。target type が term argument を含む場合、その argument 内の
`qua` は outer chain が続く前に argument term に属する。そのため
`x qua Element of S qua Magma` は `x qua Element of (S qua Magma)` として表し、
outer result をさらに修飾するには `(x qua Element of S) qua Magma` と書く。
target type 欠落時は `QuaExpression` の下に `MissingTypeExpression` を挿入し、
`MalformedTypeExpression` を出す。malformed target tail は surrounding term boundary の前で
type-tail `SkippedToken` recovery を使ってよい。`SurfaceNodeView` はこの node のために
`as_qua_expression` を公開する。static validity、type narrowing / widening、
overload selection、proof obligation は resolver / checker の責務である。

Parser task 12 は active-lexicon operator expression node を追加する。
`PrefixExpression` は operator token、その後に operand term-shape child を所有し、
parser input 由来の operator spelling と precedence を保持する。`PostfixExpression` は
base term-shape child、その後に operator token を所有し、同じ spelling/precedence payload を
保持する。`InfixExpression` は既存の `left`、operator token、`right` という child order を
保ち、さらに infix associativity を保持する。Selector/update postfix chain と ordinary
application は Pratt operand の内側で形成されるため、これらの user operator より強く bind する。
`qua` は Pratt の後に形成されるため、term-level operator の中で最も低いままである。
Non-associative error は syntax diagnostic のみである。dangling infix operator は
diagnostic のみでもよい一方、dangling prefix operator は `MissingTerm` operand を挿入して
recoverable な `PrefixExpression` を保持する。`SurfaceNodeView` は
`as_prefix_expression`、`as_postfix_expression`、`as_infix_expression` payload accessor を公開する。
Operator metadata は parser input であり、semantic resolution ではない。これらの node は
symbol id、selected overload、inferred type、proof fact を運んではならない。

Parser task 13〜14 は現在の formula node を定義する。`FormulaExpression` は atomic、
connective、quantified、parenthesized、`thesis`、`contradiction` のいずれであっても
formula child を 1 つ包み、wrapper role は変わらない。task 13 では最初に theorem/lemma
placeholder host から formula payload を公開した。task 22 では、表現済み theorem
declaration を concrete な `TheoremItem` / `LemmaItem` host に昇格し、任意の status token、
role token、label、colon、`FormulaExpression`、任意の justification または `ProofBlock`、
および enclosing semicolon を所有させる。task 31 は template predicate argument を
represented theorem / lemma payload の concrete syntax にするため、predicate head が
template argument を所有するという理由だけで placeholder に残す必要はない。

`BuiltinPredicateApplication` は left `TermExpression`、built-in predicate token（`in`、
`=`、`<>`）、right `TermExpression` または `MissingTerm` recovery を所有する。
`IsAssertion` は subject `TermExpression`、`is` token、任意の formula-level `not` token、
`TypeExpression` または `AttributeTestChain` body を所有する。この node は意図的に generic
であり、body が意味的に type assertion なのか attribute assertion なのかを判断しない。
`AttributeTestChain` は 1 個以上の task-8 `AttributeRef` node を所有し、trailing type head を
持たない `non empty` のような attribute-only assertion body のために存在する。

`PredicateApplication` は syntax-only user predicate application / chain の source-order
`PredicateSegment` child を所有する。各 `PredicateSegment` は left term operand、任意の
`does not` / `do not` negation token、1 個の `PredicateHead`、right term operand を持てる。
`PredicateHead` は predicate symbol token、qualified symbol、または template-local
identifier を包み、任意の task-31 `TemplateArguments` を所有できる。built-in predicate は単独の
`BuiltinPredicateApplication` node としてだけ表現し、`PredicateApplication` chain へ混ぜては
ならない。これにより Appendix A の `a < b = c` syntax-error boundary を保つ。
`InlinePredicateApplication` は identifier head、parentheses、source-order term argument を
所有する。これらの formula node は predicate spelling と argument shape だけを保持する。
predicate overload resolution、chain adjacency validity、theorem validity、proof fact、
truth evaluation は `mizar-syntax` の外に残る。

Parser task 14 は現在の formula vocabulary を完了する。`PrefixFormula` は `not`
token と、1 個の formula child または `MissingFormula` recovery を所有する。
`BinaryFormula` は left formula child、connective token、`& ... &` / `or ... or` 用の
optional `...` と repeated connective token、right formula child または
`MissingFormula` recovery を所有する。payload は fixed connective と repetition form が
書かれたかどうかだけを記録し、semantic expansion detail は運ばない。
`ParenthesizedFormula` は `(`、nested `FormulaExpression`、`)` または delimiter recovery を
所有する。`FormulaConstant` は 1 個の `thesis` または `contradiction` token を包み、
constant kind だけを運ぶ。

`QuantifiedFormula` は `for` または `ex` token、comma token で区切られた source-order
`QuantifierVariableSegment` child、universal quantification 用の optional `st`
condition formula、existential quantification 用の required `st` body formula、および
universal quantification 用の `holds` body formula または nested quantified-formula body を
所有する。`QuantifierVariableSegment` は書かれた variable identifier と comma、
optional `be` / `being` token、optional `TypeExpression` を所有する。`reserve` 由来の
implicit variable type を解決せず、bound variable を意味的に分類せず、proof obligation を
作らない。

`SurfaceNodeView` は `as_prefix_formula`、`as_binary_formula`、
`as_parenthesized_formula`、`as_quantified_formula`、
`as_quantifier_variable_segment`、`as_formula_constant` helper を公開する。consumer は
fixed formula payload を `SurfaceNodeKind` から読む。`PrefixFormula` は
`SurfaceFormulaPrefixOperator` を、`BinaryFormula` は
`SurfaceFormulaBinaryOperator { connective: SurfaceFormulaConnective, repeated: bool }` を、
`QuantifiedFormula` は `SurfaceQuantifierKind` を、`FormulaConstant` は
`SurfaceFormulaConstant` を運ぶ。task 14 formula node の range は、最初に所有する source
token から最後に所有する source token までを覆う。挿入された `MissingFormula` または
`MissingTypeExpression` recovery は insertion point の zero-width range を持ち、一般の
recovery 例外で out-of-range context としてだけ使われる場合を除き、parent range に
含まれなければならない。`)` が欠落した parenthesized formula は nested formula または
insertion point までを range とし、quantified formula は condition/body formula または
その quantifier 表現を完了した recovery insertion までを range とする。

Parser task 16 は simple statement node で S-013 statement 語彙を開始する。
`StatementItem` は、parse-only corpus で concrete statement が top level に現れたときの
module-level wrapper として残る。これは現在実装済みの statement 語彙から concrete
parser-owned statement node をちょうど 1 個所有し、後続の S-013 / S-014 増分である
compact、consider / reconsider、conclusion、`then`、iterative-equality statement も
同じ wrapper に入る。task 22 では `ProofBlock`、`NowStatement`、`HerebyStatement`、
case branch body も reasoning body を通じて同じ concrete statement node を直接所有する。
task 35 では、1 個以上の attachable annotation prefix が statement に先行する場合、
ordinary statement position は `AnnotatedStatement` wrapper を所有できる。
`@show_type` や `@eval` などの standalone diagnostic annotation は、後続 statement に
attach せず、statement-position syntax として parse される。Chapter 4 が
block-local `reserve` shaped statement を禁じているため、`reserve` は top-level
task-8 `ReserveItem` のままである。

`LetStatement` は `let`、comma token で区切られた 1 個以上の
`QualifiedVariableSegment`、任意の `such` と `ConditionList`、存在する場合の `;` を
所有する。`GivenStatement` も `given` 後に同じ qualified-variable と任意 condition
形状を持つ。`QualifiedVariableSegment` は書かれた identifier token と内部 comma、
任意の `be` / `being`、任意の `TypeExpression` または `MissingTypeExpression` recovery
を所有する。`reserve` からの implicit type は解決しない。

`AssumptionStatement` は `assume` と、単一の `Proposition` または `ConditionList` を
所有する。`ConditionList` は `that`、statement-level `and` token で区切られた 1 個以上の
`Proposition`、任意の recovery を所有する。`Proposition` は任意の label identifier と
colon、および 1 個の `FormulaExpression` または `MissingFormula` recovery を所有する。
`TakeStatement` は `take` と、comma token で区切られた source-order の `Witness` child
を所有する。`Witness` は 1 個の `TermExpression`、または
`identifier "=" TermExpression` の named witness spelling を所有し、witness term 欠落時は
`MissingTerm` を使う。`SetStatement` は `set` と、comma token で区切られた source-order
の `Equating` child を所有する。`Equating` は identifier または `MissingTerm` recovery、
存在する場合の `=`、右辺 `TermExpression` または `MissingTerm` を所有する。

Task 16 は task-17 justification node を意図的に除外する。semicolon 前に top-level `by`
tail を持つ `let` statement は、部分的に `LetStatement` へ parse せず legacy placeholder
のまま残す。これらの statement node は label uniqueness、reference、type
well-formedness、witness leakage、proof obligation を検証しない。`SurfaceNodeView` は
`as_statement_item`、`as_let_statement`、`as_qualified_variable_segment`、
`as_assumption_statement`、`as_proposition`、
`as_condition_list`、`as_given_statement`、`as_take_statement`、`as_witness`、
`as_set_statement`、`as_equating` helper を公開する。

Parser task 17 は S-014 justification vocabulary を開始し、最小の明示的 justification
付き compact statement host を追加する。`CompactStatement` は 1 個の `Proposition`、
1 個の `JustificationClause`、任意の recovery、存在する場合の semicolon token を所有する。
これは後続 statement task が conclusion と equality dispatch を完了する前に、共有
justification surface を exercise するために存在する。明示的 `by` tail を持たない
compact statement は後続 statement work に残す。`LetStatement` は trailing
`JustificationClause` を所有できるようになるが、Chapter 15 が定義する通常の
`by references` 形に限る。

`JustificationClause` は先頭の `by` token と、通常 citation 用の `ReferenceList` child
または `by computation(...)` 用の `ComputationJustification` child を所有する。Task 17
の reference surface は、task 31 によって local reference、qualified reference、
grouped reference member が `TemplateArguments` を所有できるよう拡張される。また `from` は
canonical Chapter 15 / 16 grammar が justification form として定義していないため、
justification node ではない。

`ReferenceList` は comma token で区切られた source-order の citation child を所有する。
local citation は 1 個の identifier token を持つ `Reference` である。
`QualifiedReference` は `NamespacePath`、最後の dot token、最後の identifier token を
所有する。`GroupedReference` は `NamespacePath`、compound `.{` token、comma token で
区切られた 1 個以上の `GroupedReferenceItem`、存在する場合の closing `}` token を
所有する。`GroupedReferenceItem` はこの増分では 1 個の identifier token を所有する。
`BulkReference` は `NamespacePath` と compound `.*` token を所有する。
`ComputationJustification` は `computation` token と、comma token で区切られた任意の
parenthesized `ComputationOption` child を所有する。各 `ComputationOption` は
`steps`、`timeout`、`nest` のいずれか、colon token、numeral token を所有する。

Justification node は citation spelling だけを保持する。label resolution、grouped /
bulk citation expansion、theorem visibility validation、ATP engine selection、
computation-option value validation、computation proof replay は行わない。range は最初の
owned source token から最後の owned source token までである。欠落した reference、
grouped item、computation option operand は、owning justification node 配下の
zero-width insertion range を持つ `MissingProofStep` recovery を使う。malformed tail は
`SkippedToken` recovery と skipped-token trivia を所有してよい。`SurfaceNodeView` は
`as_compact_statement`、`as_justification_clause`、`as_reference_list`、`as_reference`、
`as_qualified_reference`、`as_grouped_reference`、`as_grouped_reference_item`、
`as_bulk_reference`、`as_computation_justification`、`as_computation_option` helper を
公開する。snapshot rendering は literal node name を出力する。

Parser task 18 は、残りの justified introduction / type-changing form により
S-013 statement vocabulary を継続する。`ConsiderStatement` は `consider`、comma token
で区切られた 1 個以上の `QualifiedVariableSegment`、`such`、`ConditionList`、simple
`JustificationClause`、任意の recovery、存在する場合の semicolon token を所有する。
`ReconsiderStatement` は `reconsider`、comma token で区切られた 1 個以上の
`ReconsiderItem`、`as`、target `TypeExpression`、simple `JustificationClause`、
任意の recovery、存在する場合の semicolon token を所有する。`ReconsiderItem` は bare
identifier token、または equated spelling `identifier "=" TermExpression` を所有する。
Task 18 はこれらの host で simple citation justification だけを使い、computation
justification は後続仕様が他の host へ許可するまで `CompactStatement` に限る。

Task 18 statement node は syntax だけを保持する。witness existence、reconsidered name が
既に bound されているか、target type の validation、proof obligation generation は行わない。
mandatory な `by references` tail の欠落は statement node 直下の `MissingProofStep`
recovery を使う。`consider` condition の欠落は `MissingFormula`、`reconsider` item の
identifier または右辺 term 欠落は `MissingTerm`、target type 欠落は
`MissingTypeExpression` を使う。`SurfaceNodeView` は `as_consider_statement`、
`as_reconsider_statement`、`as_reconsider_item` helper を公開する。

Parser task 19 は S-013 の conclusion と iterative-equality 部分を追加する。
`ConclusionStatement` は `thus` または `hence`、1 個の `Proposition`、任意の明示的
`JustificationClause`、任意の recovery、存在する場合の semicolon token を所有する。
`ThenStatement` は syntax-only wrapper であり、`then` token とちょうど 1 個の
linkable statement child、または modifier が standalone / non-linkable statement の前に
現れた場合の `MissingStatement` recovery を所有する。この node は `hence` を desugar
せず、predecessor fact を接続せず、proof semantics を encode しない。

`IterativeEqualityStatement` は任意の label identifier と colon、最初の left
`TermExpression`、`=`、最初の right `TermExpression`、任意の simple citation
`JustificationClause`、1 個以上の `IterativeEqualityStep` child、任意の recovery、
存在する場合の semicolon token を所有する。`IterativeEqualityStep` は `.=` と
1 個の `TermExpression` または `MissingTerm`、任意の simple citation
`JustificationClause` を所有する。compact / equality dispatch boundary は syntax-only
である。top-level `.=` continuation を持たない justified equality は
`CompactStatement` のままにし、1 個以上の top-level `.=` を持つ chain は
`IterativeEqualityStatement` になる。Chapter 15 の production が iterative equality で
`simple_justification` を使うため、computation justification は iterative equality 内で
許可しない。一方、明示的な conclusion は一般の task-17 justification surface を再利用してよい。
これらの node は equality transitivity、predecessor availability、conclusion validity、
proof obligation を検査しない。`SurfaceNodeView` は `as_conclusion_statement`、
`as_then_statement`、`as_iterative_equality_statement`、
`as_iterative_equality_step` helper を公開する。snapshot rendering は literal node name を
出力する。

Parser task 20 は S-013 の reasoning-block 部分を追加する。`NowStatement` は任意の
label syntax、`now` opener、0 個以上の nested statement node、任意の recovery、
存在する場合の block-closing `end` と semicolon を所有する。`HerebyStatement` は
label syntax を持たない同じ block body 形状である。`CaseReasoningStatement` は
`per`、`cases`、任意の simple citation `JustificationClause`、header semicolon、
source-order の homogeneous な `CaseItem` child 0 個以上、または homogeneous な
`SupposeItem` child 0 個以上を所有する。`CaseItem` と
`SupposeItem` は keyword、`Proposition` または `ConditionList`、header semicolon、
0 個以上の nested statement node、任意の recovery、存在する場合の block-closing
`end` と semicolon を所有する。Chapter 15 の prose / example と既存 parse-only
fixture が explicit justification なしの形を exercise しているため、parser surface は
`per cases;` と `per cases by A;` の両方を受け入れる。parser は exhaustiveness、
branch coverage、label scope、witness leakage、`now` block から export される formula を
分類しない。`SurfaceNodeView` は `as_now_statement`、`as_hereby_statement`、
`as_case_reasoning_statement`、`as_case_item`、`as_suppose_item` helper を公開する。
snapshot rendering は literal node name を出力する。

Parser task 21 は S-013 の local inline-definition 部分を追加する。
`InlineFunctorDefinition` は `deffunc`、definition name identifier または
`MissingTerm` recovery、parameter
parentheses、comma token で区切られた `TypedParameter` child 0 個以上、`->`
return-type delimiter、`TypeExpression` または `MissingTypeExpression` recovery、
`equals`、`TermExpression` または `MissingTerm` recovery、任意の malformed-tail
recovery、存在する場合の final semicolon を所有する。
`InlinePredicateDefinition` は同じ parameter head 形状に加えて、`defpred`、
`means`、`FormulaExpression` または `MissingFormula` recovery を所有する。
`TypedParameter` は存在する場合の parameter identifier、書かれた場合の optional
`be` または `being`、`TypeExpression` または `MissingTypeExpression` recovery を
所有する。binder keyword 欠落は `TypedParameter` 内でその token がないことで表し、
malformed-type diagnostic を併記する。delimiter 前に recoverable な type がない場合は
`MissingTypeExpression` が type slot を埋める。これらの node は scope introduction、
definition expansion、parameter guard check、後続の
inline-name application resolution を model しない。`SurfaceNodeView` は
`as_inline_functor_definition`、`as_inline_predicate_definition`、
`as_typed_parameter` helper を公開する。snapshot rendering は literal node name を
出力する。

Parser task 23 は S-015 definition-family vocabulary を開始する。
`DefinitionBlockItem` は任意の leading `Annotation` node、`definition` opener、
source-order content、任意の recovery、存在する場合の block-closing `end` と
semicolon を所有する。この増分で
concrete content として扱う範囲は、通常の `DefinitionParameter` node、
`AssumptionStatement`、`AttributeDefinition`、`CorrectnessCondition`、
`TheoremItem`、`LemmaItem`、および visibility-wrapped theorem / lemma content に
意図的に限定する。
`let T be type;` のような template-ambiguous parameter と、predicate、functor、
mode、redefinition、structure、property、registration、cluster、reduction declaration
など後続の definition-family form は、対応する parser task が実装されるまで
source-preserving `PlaceholderItem` content のまま保持する。

`DefinitionParameter` は `let`、qualified-variable segment、任意の condition-list
または `such that` formula と任意の justification、任意の recovery、存在する場合の
semicolon を所有する。`AttributeDefinition` は `attr`、definition label、`:`、
subject identifier、`is`、`AttributePattern`、`means`、`FormulaDefiniens`、
任意の recovery、存在する場合の semicolon を所有する。`AttributePattern` は
任意の `ParameterPrefix` と identifier または user-symbol name を所有する。
`FormulaDefiniens` は単一の `FormulaExpression`、または任意の `otherwise` formula
を伴う conditional `FormulaCase` child を所有する。`CorrectnessCondition` は
correctness keyword と、`by`、`by computation(...)`、または full `ProofBlock` を
含む任意の general justification を所有する。

これらの node は definition syntax だけを保持する。definitional correctness、
attribute admissibility、existence / uniqueness obligation、cluster closure、
notation resolution、type / proof semantics、theorem visibility は検査しない。
attribute label、subject、pattern name の欠落は `MissingTerm` を使う。definition
parameter と formula definiens case 内の formula 欠落は `MissingFormula` を使う。
malformed justification は `MissingProofStep` を使ってよい。definition block closer
欠落は `MissingEnd` を使う。malformed tail は owning definition content node の下で
skip される。`SurfaceNodeView` は `as_definition_block_item`、
`as_definition_parameter`、`as_attribute_definition`、`as_attribute_pattern`、
`as_formula_definiens`、`as_formula_case`、`as_correctness_condition` helper を公開する。
snapshot rendering は literal node name を出力する。

Parser task 24 は、次の S-015 増分として predicate definition を追加する。
`PredicateDefinition` は `pred`、definition label、`:`、`PredicatePattern`、
`means`、task-23 の `FormulaDefiniens`、任意の recovery、存在する場合の semicolon を
所有する。definition-local な `public pred` と `private pred` は、既存の
`VisibleItem` wrapper で `PredicateDefinition` を包んで表す。その他の visible
definition kind は、それぞれの parser task に残す。

`PredicatePattern` は pattern を source-order の raw token child として保持する。
parser は raw span が `[ loci ] def_predicate_symbol [ template_loci ] [ loci ]` に
少なくとも 1 通りの syntactic split で一致できることを検証するが、どの
identifier が predicate symbol かは AST に記録しない。これにより、prefix-like /
postfix-like な 2 identifier pattern のような phrase-pattern ambiguity は resolver
所有のまま残る。primitive built-in predicate token である `in`、`=`、`<>` は
`def_predicate_symbol` を満たせないため、malformed predicate pattern として
`MissingTerm` recovery で表される。bracketed `template_loci` token は
`PredicatePattern` 内に保持してよいが、task 24 は template-specific node を追加せず、
`definition ... end;` block を template definition として分類しない。G-AUD-006 は
S-016 に向けて open のままである。`SurfaceNodeView` は
`as_predicate_definition` と `as_predicate_pattern` helper を公開する。
snapshot rendering は literal node name を出力する。

Parser task 25 は functor definition を次の S-015 増分として追加する。
`FunctorDefinition` は `func`、definition label、`:`、`FunctorPattern`、`->`、
return `TypeExpression` または `MissingTypeExpression`、`means FormulaDefiniens` または
`equals TermDefiniens`、任意の recovery、存在する場合の semicolon を所有する。
definition-local な `public func` と `private func` は、既存の `VisibleItem` wrapper で
`FunctorDefinition` を包んで表す。functor に続く correctness condition は、別個の
`CorrectnessCondition` definition-content node のままである。

`FunctorPattern` は pattern を source-order の raw token child として保持する。
parser は、raw span が canonical な
`[ loci ] functor_symbol [ template_loci ] [ loci ]` 形に少なくとも 1 通りの syntactic
split で一致できるか、または Chapter 10 で文書化されている two-symbol circumfix
shape（non-empty loci list を functor-symbol token が挟む形）に一致できることを検証する。
AST は、どの token が functor symbol か、pattern が prefix / postfix / infix のどれか、
circumfix pair がどう結合するかを記録しない。それらの role は resolver-owned のままである。
bracketed `template_loci` token は `FunctorPattern` 内に保持してよいが、task 25 は
template-specific node を追加せず、`definition ... end;` block を template definition として
分類しない。G-AUD-006 は S-016 に向けて open のままである。

`TermDefiniens` と `TermCase` は、`FormulaDefiniens` / `FormulaCase` に対応する
`equals` body 用 node である。`SurfaceNodeView` は `as_functor_definition`、
`as_functor_pattern`、`as_term_definiens`、`as_term_case` helper を公開する。
snapshot rendering は literal node name を出力する。

Parser task 26 は mode definition を次の S-015 増分として追加する。
`ModeDefinition` は `mode`、definition label、`:`、raw `ModePattern`、`is`、
body `TypeExpression` または `MissingTypeExpression`、存在する場合の最初の
semicolon、直後に続く任意の `ModeProperty` を所有する。definition-local な
`public mode` と `private mode` は、`ModeDefinition` を既存の `VisibleItem`
wrapper で包んで表す。

`ModePattern` は pattern を source-order raw token child として保持する。parser は
raw span が、ちょうど 1 個の identifier または active user-symbol name と、`of`、
`over`、または bracket で導入される最大 1 個の non-empty type-parameter list からなる
`mode_def_name [ type_params ]` に一致することを検証する。AST は body head が semantic
に radix type か、parameter list が dependent か、mode が inhabited かを記録しない。
`ModeProperty` は `sethood` keyword と必須の syntax-level general justification を
所有する。sethood proof obligation は syntax crate の外に残る。`SurfaceNodeView` は
`as_mode_definition`、`as_mode_pattern`、`as_mode_property` helper を公開する。
snapshot rendering は literal node name を出力する。

Parser task 27 は S-015 の redefinition と notation alias 部分を追加する。
`AttributeRedefinition`、`PredicateRedefinition`、`FunctorRedefinition` は、対応する
task 23〜25 の definition body と同じ形を再利用するが、先頭の `redefine` token と
末尾の必須 `CoherenceCondition` を所有する。仕様は attribute、predicate、functor に
ついてのみ redefinition production を定義しており、canonical grammar には
`redefine_mode` production は存在しない。したがって mode syntax は task 27 の
notation alias では raw `NotationPattern` child として参加するが、`redefine mode`
source は創作された言語挙動として表現せず、recovery / deferred 扱いに残す。
definition-local な `public` / `private` redefinition は、Appendix A の
`[ visibility ] definitional_item` shape に合わせ、concrete redefinition node を既存の
`VisibleItem` / `VisibilityMarker` wrapper で包んで表す。

`CoherenceCondition` は `coherence` keyword、任意の `with` と label identifier、
存在する場合の必須 syntax-level general justification（`JustificationClause` または
`ProofBlock`）、任意の recovery、存在する場合の coherence semicolon を所有する。
これは一般の task-23 `CorrectnessCondition` とは別ノードである。redefinition の
coherence は standalone な definition-content clause ではなく、redefinition item の一部
だからである。AST は、その coherence proof が同値性、result-type agreement、
または previous definition との compatibility を証明するかを検査しない。

`NotationAlias` は `synonym` と `antonym` declaration の両方を表す。operator
declaration は canonical `notation_decl` の deferred branch のままであり、この task は
alias surface だけを追加する。`NotationAlias` は alias keyword、alternate
`NotationPattern`、`for` token、original `NotationPattern`、任意の recovery、存在する
場合の terminating semicolon を所有する。`NotationPattern` は alias の片側を
source-order raw token として保持する。このノードは pattern を predicate、functor、
mode、attribute のいずれかとして分類せず、どの token が導入・参照される symbol で
あるかも決定しない。その曖昧性は active symbol environment に依存するため、
resolver-owned のまま残る。

Top-level と definition-local の notation alias は同じ `NotationAlias` surface を使う。
grammar が `[ visibility ] notation_decl` を認める場所では、visibility は既存の
`VisibleItem` / `VisibilityMarker` wrapper で表す。`SurfaceNodeView` は
`as_attribute_redefinition`、`as_predicate_redefinition`,
`as_functor_redefinition`、`as_coherence_condition`、`as_notation_alias`、
`as_notation_pattern` helper を公開する。snapshot rendering は literal node name を
出力する。

Parser task 28 は syntax-only の property item 句を追加する。`PropertyClause` は
canonical な property keyword、存在する場合の必須 syntax-level general justification
（`JustificationClause` または `ProofBlock`）、任意の recovery、存在する場合の
property semicolon を所有する。受理する keyword は predicate property の `symmetry`、
`asymmetry`、`connectedness`、`reflexivity`、`irreflexivity`、functor property の
`commutativity`、`idempotence`、`involutiveness`、`projectivity`、および standalone
mode `sethood` である。`transitivity` は reserved word だが、現在の canonical property
production では concrete property-clause keyword ではない。`ModeDefinition` 直後の
`sethood` 句は task-26 の `ModeProperty` として表され続ける。standalone `sethood`
property item は `PropertyClause` を使う。AST は predicate arity、functor arity、
proof obligation、property がどの先行 definition を注釈するかを検査しない。
`SurfaceNodeView` は `as_property_clause` を公開する。snapshot rendering は
`PropertyClause` を出力する。

Parser task 29 は syntax-only の structure definition と inheritance definition を
追加する。`StructureDefinition` は `struct`、raw `StructurePattern`、`where`、1 個以上の
`StructureField` / `StructureProperty` member、`end`、存在する場合の final semicolon を
所有する。`StructurePattern` は raw `struct_def_name [ type_params ]` token を保持し、
semantic な structure identity は決定しない。structure field は `field`、field name、
`->`、syntactic `TypeExpression`、任意の initializer `:= TermExpression`、member
semicolon を所有する。structure property は initializer を持たない同じ形を所有する。
`InheritanceDefinition` は `inherit`、`extends` をはさむ child / parent
`InheritanceTarget` node、任意の explicit `where ... end` inheritance member、final
semicolon を所有する。`FieldRedefinition` と `PropertyRedefinition` は任意の narrowed
type と必須 `from` source token を保持する。inheritance coherence は mandatory general
justification を持つ `CoherenceCondition` を再利用し、redefinition 専用の `with` label
branch は持たない。AST は inheritance coverage、diamond consistency、selector
compatibility、type specialization validity、constructor semantics、proof obligation を
検査しない。`SurfaceNodeView` は `as_structure_definition`、
`as_structure_pattern`、`as_structure_field`、`as_structure_property`、
`as_inheritance_definition`、`as_inheritance_target`、`as_field_redefinition`、
`as_property_redefinition` を公開する。snapshot rendering は literal node name を出力する。

Parser task 30 は S-015 の registration-family vocabulary を完了する。
`RegistrationBlockItem` は任意の leading `Annotation` node、`registration` opener、
source-order の registration content、任意の recovery、存在する場合の block-closing
`end` と semicolon を所有する。
`RegistrationParameter` は `let`、通常の qualified-variable segment、任意の `such`
condition-list child、任意の syntax-level `by` reference、任意の recovery、存在する場合の
semicolon を所有する。proof-bearing な definition constraint や template-definition
parameter semantics は所有しない。

`ExistentialRegistration`、`ConditionalRegistration`、
`FunctorialRegistration` は 3 種類の `cluster` branch を表す。
`ExistentialRegistration` は `cluster` keyword、label、colon、1 個の attributed
`TypeExpression`、header semicolon、`existence` の `CorrectnessCondition` を所有する。
`ConditionalRegistration` は `->` の前の registration-adjective `AttributeRef` child、
consequent の registration-adjective `AttributeRef` child、`for`、target
`TypeExpression`、header semicolon、`coherence` の `CorrectnessCondition` を所有する。
`FunctorialRegistration` は syntactically unambiguous な functorial payload term、
`->`、consequent registration adjective、`for`、target type、header semicolon、
`coherence` の `CorrectnessCondition` を所有する。registration adjective ref は optional
`non`、optional `ParameterPrefix`、parenthesized argument を持たない attribute name に
制限される。argument-bearing adjective spelling は malformed syntax として recover する。

`ReductionRegistration` は `reduce`、label、colon、left `TermExpression`、`to`、
right `TermExpression`、header semicolon、`reducibility` の
`CorrectnessCondition` を所有する。definition-local な `public` / `private`
registration item は concrete registration item を既存の `VisibleItem` で包む。
これらの node は registration syntax だけを保持する。cluster closure、reduced form の
推論、reducibility proof replay、target type validation、nullary functorial ambiguity の
判定、proof obligation check は行わない。label 欠落と unsupported functorial payload は
`MissingTerm` を使う。antecedent / consequent adjective と target type の欠落は
`MissingTypeExpression` を使う。correctness justification 欠落は `MissingProofStep` を
使ってよい。registration block closer 欠落は `MissingEnd` を使う。malformed tail は
owning registration content node の下で skip される。`SurfaceNodeView` は
`as_registration_block_item`、`as_registration_parameter`、
`as_existential_registration`、`as_conditional_registration`、
`as_functorial_registration`、`as_reduction_registration` を公開する。snapshot rendering は
literal node name を出力する。

### 語彙増分の契約

node 語彙は、その形を構築する `mizar-parser` 文法タスクと同じ変更でのみ増やす。
各増分では、実装と同時または先行して、追加する各公開 syntax kind について
この仕様に次の契約を書く。

- `SurfaceNodeKind` variant 名と raw `SyntaxKind` mapping。
- payload field がある場合、その内容と、それが parser fact なのか互換 data なのか。
- child role と child order。optional / repeated role も含める。
- node と child の range rule。文書化された recovery 例外も含める。
- 生の rowan traversal ではなく consumer が使うべき typed accessor / view helper。
- 新しい kind の snapshot rendering text と、escaping / sorting rule。
- skipped token、欠落 construct、doc-comment attachment、空白依存 hint を所有する場合の
  recovery / trivia との相互作用。

`doc/spec/ja/` 配下の言語文法は、どの構文要素が存在するかを定義する。この
モジュール仕様は、それらを `SurfaceAst` でどう表現するかを定義する。

### Builder 境界

`SurfaceAstBuilder` は parser 向けの構築境界である。parser code は builder
method 経由で token、通常 node、recovery node を追加し、root と任意の
expression root を指定して finish する。parser grammar code は private arena
へ直接 push したり、rowan node を直接確保したり、生の rowan traversal に依存
したりしてはならない。文法拡張で新しい tree 操作が必要になった場合は、まず
ここに typed builder または accessor として追加する。

builder id は 1 つの builder instance に局所的である。別 builder 由来の child、
root、expression-root id は無効である。`add_node` は通常の structural node だけを
作る。token node は `add_token` または `add_recovered_token`、recovery node は
`add_recovery` で作らなければならない。`finish` は、任意の root と expression
root が存在すること、また non-root の structural parent が child subtree を共有
していないことを検証する。

構築中、parser 基盤は `node_kind` や `node_range` のような typed builder accessor
を通じて、すでに送出した builder node を検査してよい。これらの accessor は parser
composition に必要な surface kind と source range だけを公開し、private builder
arena を storage contract として露出しない。

互換 root は、task 12 の consumer が両方の view を検査し続けられるよう、ソース順
の token node と、それらの token を含む structural node の両方を列挙してよい。
rowan green tree は source-shaped のままである。structural child が source token
を所有する場合、builder は互換 root listing から token leaf を重複させず、その
structural rowan node の下に一度だけ出力しなければならない。Recovery node は自身
の insertion range の外にある context child を互換 view に保持してよいが、その
out-of-range context child は recovery rowan node の下には出力しない。

現在の rowan construction は、root に列挙された token node が non-recovery の
structural root child の descendant でもある場合に deduplicate する。その structural
subtree は、malformed import-tail recovery のように、in-range token child を持つ
recovery node を内部に含んでよい。この場合 token leaf は structural rowan subtree の
下に一度だけ出力され、互換 root の token pass からは省略される。互換 root に直接
列挙される recovery node は root-listed token の deduplication owner ではないため、
後続の builder check または rowan emission rule がその case を文書化するまでは、
parser producer はそのような root-level recovery node に in-range token child を
持たせてはならない。missing-construct recovery には out-of-range context child を
使うか、skipped-token recovery を non-recovery structural owner の下に nest し、
skip された source span を trivia に記録する。

### Accessor 規約

`SurfaceAst::node_view`、`root_view`、`expression_view`、`token_views` は typed
view を返し、rowan traversal を要求せずに kind、range、recovered flag、children、
token payload、operator payload、recovery kind を公開する。互換用の
`SurfaceAst::node` accessor は既存テストと移行コードのために残す。

### Snapshot rendering

`SurfaceAst::snapshot_text` は、syntax test と後続の parser corpus baseline が使う、
決定的で人間可読な surface snapshot format を返す。format は
`surface-ast-snapshot-v1` header で version 付けされ、root view、任意の
expression root、token 互換 view を保存順で安定して描画する。各 node 行には
surface kind、source-local な byte range、`recovered` flag、および現在の構文語彙
を区別するための kind 固有 payload（token kind/text、operator
spelling/precedence/fixity fact、または recovery kind）を含める。

snapshot text は、rowan pointer identity、builder id、`SurfaceNodeId` 値、
生の `SourceId` debug 出力、absolute path、実行時間、hash-map iteration order、
その他の非決定的データを意図的に含めない。range は `SurfaceAst` の source 内の
byte offset として描画する。source identity は `mizar-test` が所有する外側の
snapshot/profile record の責務である。

`SurfaceAst::snapshot_text_with_trivia` は、[trivia.md](./trivia.md) で定義する
決定的な trivia side table を追加して描画する。既定の syntax snapshot はその
section を省略し、既存の syntax-only baseline を安定させる。

現在の syntax snapshot format は次のとおり。

```text
surface-ast-snapshot-v1
root:
  <node-or-none>
expression_root:
  <node-or-none>
token_nodes:
  <node-or-none>
```

node 行は depth ごとに 2 space で indent し、現在は次の形を使う。

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
Annotation range=<start>..<end> recovered=<bool>
LibraryAnnotation range=<start>..<end> recovered=<bool>
AnnotationLabelList range=<start>..<end> recovered=<bool>
AnnotationLabel range=<start>..<end> recovered=<bool>
AnnotationArgumentList range=<start>..<end> recovered=<bool>
AnnotationArgument range=<start>..<end> recovered=<bool>
ProofHintOptionList range=<start>..<end> recovered=<bool>
ProofHintOption range=<start>..<end> recovered=<bool>
StandaloneDiagnosticAnnotation range=<start>..<end> recovered=<bool>
AnnotatedStatement range=<start>..<end> recovered=<bool>
AnnotatedAlgorithmStatement range=<start>..<end> recovered=<bool>
AnnotatedDefinitionContent range=<start>..<end> recovered=<bool>
AnnotatedRegistrationContent range=<start>..<end> recovered=<bool>
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

`<escaped-text>` は Rust の default character escaping を使うため、制御文字、
quote、backslash、非表示 character は決定的に描画される。snapshot format を変更する
場合は、新しい header version に加え、この仕様、日本語 companion、影響を受ける
baseline snapshot の更新が必要である。外側の snapshot envelope または update policy
が変わる場合にのみ、`mizar-test` snapshot documentation を更新する。

### Range attachment

各 surface node は `mizar-session` の `SourceRange` を持つ。通常 node では親の
range がすべての子の range を包含する。recovery node は、zero-width insertion
node が opener や skipped token を context として保持する場合、この包含関係を
破ってよい。たとえば missing-`end` recovery node は EOF の挿入 range に付き、
子は block opener を指し戻す。

### Identity rules

rowan green-node identity、rowan text range、dense な `SurfaceNodeId` は内部
cache と互換性の詳細である。構築済み `SurfaceAst` の中では決定的だが、安定
artifact id ではなく、cross-run identity として serialize してはならない。
安定した消費者は deterministic snapshot、content cache key、source id/range、
および後段の resolver/checker layer が所有する semantic id を key にする。

### 公開 enum の互換性

現在の公開 syntax enum は、まだ長命な resolver / LSP surface ではない。parser
task 5〜7 により downstream input として現実的になる前に、[todo.md](./todo.md)
の consumer 前ゲートを適用する。将来の語彙増加を約束する enum
（`SyntaxKind`、`SurfaceNodeKind`、`SurfaceTokenKind`）は、下流 crate 向けに
`#[non_exhaustive]` とし、lint-policy gate がこれらの属性を固定する。
`MizarLanguage` は downstream の syntax category ではなく空の rowan marker enum
であるため、意図的に exhaustive のままとする。`SurfaceOperatorAssociativity` は現在、
閉じた三分の operator property（`Left`、`Right`、`NonAssociative`）であり、後続の
operator-model task が新しい associativity category を設計しない限り、意図的に
exhaustive のままとする。task 14 の formula payload enum
（`SurfaceFormulaPrefixOperator`、`SurfaceFormulaConnective`、
`SurfaceQuantifierKind`、`SurfaceFormulaConstant`）も、現在の固定 grammar table
を表すため意図的に exhaustive とする。新しい formula operator、quantifier、constant
を追加する場合は、parser/syntax の match と文書更新がローカルで必要になる。この crate
内部の match は exhaustive のままにし、新しい variant 追加時にローカル更新がコンパイル時に
促されるようにする。下流 crate は `#[non_exhaustive]` により必要になる箇所で wildcard
fallback arm を含めなければならない。

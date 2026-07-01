//! Stable diagnostic-code registry.

use std::{collections::BTreeMap, error::Error, fmt, str::FromStr};

const INITIAL_SINCE: &str = "spec-22.7-v1";
const INITIAL_DOC_URL: &str =
    "doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference";

macro_rules! builtin_descriptor {
    ($severity:ident, $number:literal, $name:literal, $family:ident, $summary:literal) => {
        DiagnosticDescriptor {
            code: DiagnosticCode::from_parts_unchecked(DiagnosticSeverity::$severity, $number),
            meaning_key: $name,
            semantic_name: $name,
            default_severity: DiagnosticSeverity::$severity,
            phase_family: PhaseFamily::$family,
            summary: $summary,
            doc_url: INITIAL_DOC_URL,
            status: DiagnosticStatus::Active,
            since: INITIAL_SINCE,
            retired_since: None,
            replacement_codes: &[],
            aliases: &[],
        }
    };
}

/// User-facing diagnostic severity encoded in a diagnostic-code prefix.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticSeverity {
    /// Error diagnostic, encoded by the `E` prefix.
    Error,
    /// Warning diagnostic, encoded by the `W` prefix.
    Warning,
    /// Informational diagnostic, encoded by the `I` prefix.
    Info,
}

impl DiagnosticSeverity {
    /// Returns the stable diagnostic-code prefix for this severity.
    pub const fn prefix(self) -> char {
        match self {
            Self::Error => 'E',
            Self::Warning => 'W',
            Self::Info => 'I',
        }
    }

    const fn from_prefix(prefix: u8) -> Option<Self> {
        match prefix {
            b'E' => Some(Self::Error),
            b'W' => Some(Self::Warning),
            b'I' => Some(Self::Info),
            _ => None,
        }
    }
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        })
    }
}

/// Canonical phase-family vocabulary for registry descriptors.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhaseFamily {
    /// Lexical and syntax diagnostics.
    Syntax,
    /// Type diagnostics.
    Type,
    /// Resolution, overload, and template diagnostics.
    Resolution,
    /// Proof and ATP diagnostics.
    Proof,
    /// Logical consistency and verification-condition diagnostics.
    Logic,
    /// Algorithm verification diagnostics.
    Algorithm,
    /// Structural warnings.
    StructuralWarning,
    /// Proof and ATP warnings.
    ProofWarning,
    /// Algorithm and contract warnings.
    AlgorithmWarning,
    /// Compatibility and packaging warnings.
    CompatibilityWarning,
    /// Informational display diagnostics.
    Info,
}

impl PhaseFamily {
    /// Returns the default severity required by the code range.
    pub const fn default_severity(self) -> DiagnosticSeverity {
        match self {
            Self::Syntax
            | Self::Type
            | Self::Resolution
            | Self::Proof
            | Self::Logic
            | Self::Algorithm => DiagnosticSeverity::Error,
            Self::StructuralWarning
            | Self::ProofWarning
            | Self::AlgorithmWarning
            | Self::CompatibilityWarning => DiagnosticSeverity::Warning,
            Self::Info => DiagnosticSeverity::Info,
        }
    }
}

impl fmt::Display for PhaseFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Syntax => "Syntax",
            Self::Type => "Type",
            Self::Resolution => "Resolution",
            Self::Proof => "Proof",
            Self::Logic => "Logic",
            Self::Algorithm => "Algorithm",
            Self::StructuralWarning => "StructuralWarning",
            Self::ProofWarning => "ProofWarning",
            Self::AlgorithmWarning => "AlgorithmWarning",
            Self::CompatibilityWarning => "CompatibilityWarning",
            Self::Info => "Info",
        })
    }
}

/// Stable public identity for a diagnostic.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DiagnosticCode {
    severity: DiagnosticSeverity,
    number: u16,
}

impl DiagnosticCode {
    /// Creates a diagnostic code from a severity prefix and four-digit number.
    pub const fn from_parts(
        severity: DiagnosticSeverity,
        number: u16,
    ) -> Result<Self, DiagnosticCodeError> {
        if number > 9999 {
            return Err(DiagnosticCodeError::NumberOutOfRange { number });
        }
        Ok(Self { severity, number })
    }

    const fn from_parts_unchecked(severity: DiagnosticSeverity, number: u16) -> Self {
        Self { severity, number }
    }

    /// Returns the severity prefix encoded in this code.
    pub const fn severity(self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the numeric part of this code.
    pub const fn number(self) -> u16 {
        self.number
    }

    /// Returns the canonical phase family for this code's range.
    pub const fn phase_family(self) -> Option<PhaseFamily> {
        match (self.severity, self.number) {
            (DiagnosticSeverity::Error, 1..=99) => Some(PhaseFamily::Syntax),
            (DiagnosticSeverity::Error, 100..=199) => Some(PhaseFamily::Type),
            (DiagnosticSeverity::Error, 200..=299) => Some(PhaseFamily::Resolution),
            (DiagnosticSeverity::Error, 300..=399) => Some(PhaseFamily::Proof),
            (DiagnosticSeverity::Error, 400..=499) => Some(PhaseFamily::Logic),
            (DiagnosticSeverity::Error, 500..=599) => Some(PhaseFamily::Algorithm),
            (DiagnosticSeverity::Warning, 1..=99) => Some(PhaseFamily::StructuralWarning),
            (DiagnosticSeverity::Warning, 100..=199) => Some(PhaseFamily::ProofWarning),
            (DiagnosticSeverity::Warning, 200..=299) => Some(PhaseFamily::AlgorithmWarning),
            (DiagnosticSeverity::Warning, 300..=399) => Some(PhaseFamily::CompatibilityWarning),
            (DiagnosticSeverity::Info, 1..=99) => Some(PhaseFamily::Info),
            _ => None,
        }
    }

    /// Returns the default severity required by the code range.
    pub const fn default_severity(self) -> Option<DiagnosticSeverity> {
        match self.phase_family() {
            Some(phase_family) => Some(phase_family.default_severity()),
            None => None,
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{:04}", self.severity.prefix(), self.number)
    }
}

impl FromStr for DiagnosticCode {
    type Err = DiagnosticCodeError;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        let bytes = code.as_bytes();
        if bytes.len() != 5 {
            return Err(DiagnosticCodeError::InvalidLength {
                actual: bytes.len(),
            });
        }

        let Some(severity) = DiagnosticSeverity::from_prefix(bytes[0]) else {
            return Err(DiagnosticCodeError::InvalidPrefix {
                actual: code
                    .chars()
                    .next()
                    .expect("nonempty code checked by length"),
            });
        };

        if !bytes[1..].iter().all(u8::is_ascii_digit) {
            return Err(DiagnosticCodeError::InvalidDigits);
        }

        let number = u16::from(bytes[1] - b'0') * 1000
            + u16::from(bytes[2] - b'0') * 100
            + u16::from(bytes[3] - b'0') * 10
            + u16::from(bytes[4] - b'0');

        Self::from_parts(severity, number)
    }
}

/// Error returned when parsing or constructing a malformed diagnostic code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagnosticCodeError {
    /// The code was not exactly five ASCII bytes.
    InvalidLength {
        /// Observed byte length.
        actual: usize,
    },
    /// The prefix was not `E`, `W`, or `I`.
    InvalidPrefix {
        /// Observed leading character.
        actual: char,
    },
    /// One or more numeric positions were not ASCII digits.
    InvalidDigits,
    /// The numeric part exceeded four decimal digits.
    NumberOutOfRange {
        /// Observed numeric value.
        number: u16,
    },
}

impl fmt::Display for DiagnosticCodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { actual } => {
                write!(formatter, "diagnostic code must be 5 bytes, got {actual}")
            }
            Self::InvalidPrefix { actual } => {
                write!(formatter, "invalid diagnostic code prefix `{actual}`")
            }
            Self::InvalidDigits => {
                formatter.write_str("diagnostic code number must contain four ASCII digits")
            }
            Self::NumberOutOfRange { number } => {
                write!(formatter, "diagnostic code number {number} exceeds 9999")
            }
        }
    }
}

impl Error for DiagnosticCodeError {}

/// Registry lifecycle status for a diagnostic descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticStatus {
    /// The code may be emitted by current diagnostics.
    Active,
    /// The code is kept for historical records and must not be emitted anew.
    Retired,
}

/// Metadata attached to a stable diagnostic code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticDescriptor {
    /// Stable public code.
    pub code: DiagnosticCode,
    /// Stable compatibility key. Initial descriptors use the semantic name.
    pub meaning_key: &'static str,
    /// Current human-readable semantic name.
    pub semantic_name: &'static str,
    /// Descriptor default severity.
    pub default_severity: DiagnosticSeverity,
    /// Canonical phase family.
    pub phase_family: PhaseFamily,
    /// Short English summary.
    pub summary: &'static str,
    /// Canonical documentation target.
    pub doc_url: &'static str,
    /// Descriptor lifecycle status.
    pub status: DiagnosticStatus,
    /// Version or design task that first allocated the code.
    pub since: &'static str,
    /// Version or design task that retired the code.
    pub retired_since: Option<&'static str>,
    /// Replacement codes for retired diagnostics.
    pub replacement_codes: &'static [DiagnosticCode],
    /// Previous semantic names for this same meaning key.
    pub aliases: &'static [&'static str],
}

/// Built-in descriptors allocated by the initial spec-22 registry.
pub const BUILTIN_DESCRIPTORS: &[DiagnosticDescriptor] = &[
    builtin_descriptor!(
        Error,
        1,
        "syntax.unexpected_token",
        Syntax,
        "Unexpected token in current syntactic context"
    ),
    builtin_descriptor!(
        Error,
        2,
        "syntax.malformed_literal",
        Syntax,
        "Numeric or string literal does not conform to lexical rules"
    ),
    builtin_descriptor!(
        Error,
        3,
        "syntax.unexpected_end_of_file",
        Syntax,
        "File ended with open construct pending"
    ),
    builtin_descriptor!(
        Error,
        10,
        "syntax.missing_end",
        Syntax,
        "Block opened without matching `end`"
    ),
    builtin_descriptor!(
        Error,
        11,
        "syntax.unmatched_delimiter",
        Syntax,
        "Parenthesis, bracket, or `do` without matching close"
    ),
    builtin_descriptor!(
        Error,
        12,
        "syntax.reserved_keyword_as_identifier",
        Syntax,
        "Reserved keyword used as identifier"
    ),
    builtin_descriptor!(
        Error,
        101,
        "type.mismatch",
        Type,
        "Expression type incompatible with required type"
    ),
    builtin_descriptor!(
        Error,
        102,
        "type.narrowing_requires_proof",
        Type,
        "Narrowing coercion without justification"
    ),
    builtin_descriptor!(
        Error,
        103,
        "type.sethood.missing",
        Type,
        "Fraenkel comprehension for type without `sethood`"
    ),
    builtin_descriptor!(
        Error,
        110,
        "type.inference_conflict",
        Type,
        "Conflicting type constraints within an expression"
    ),
    builtin_descriptor!(
        Error,
        120,
        "type.mode_mismatch",
        Type,
        "Mode incompatibility without registered widening"
    ),
    builtin_descriptor!(
        Error,
        121,
        "type.attribute_required",
        Type,
        "Required attribute not registered for the type"
    ),
    builtin_descriptor!(
        Error,
        122,
        "type.attribute_contradiction",
        Type,
        "Attribute combination rejected: mutually exclusive attributes or missing existential cluster"
    ),
    builtin_descriptor!(
        Error,
        201,
        "resolve.ambiguous_symbol",
        Resolution,
        "Two or more equally-ranked overload candidates"
    ),
    builtin_descriptor!(
        Error,
        202,
        "resolve.no_viable_overload",
        Resolution,
        "No candidate survives type-checking"
    ),
    builtin_descriptor!(
        Error,
        203,
        "template.argument_omitted_not_inferable",
        Resolution,
        "Template schema parameter cannot be inferred"
    ),
    builtin_descriptor!(
        Error,
        204,
        "resolve.incompatible_refinement_join",
        Resolution,
        "Same-root redefinitions expose incompatible joined facts"
    ),
    builtin_descriptor!(
        Error,
        301,
        "proof.by.search_exhausted",
        Proof,
        "`by` step: ATP exhausted resource budget"
    ),
    builtin_descriptor!(
        Error,
        302,
        "proof.by.missing_fact",
        Proof,
        "Goal likely provable but required lemma not in scope"
    ),
    builtin_descriptor!(
        Error,
        303,
        "proof.obligation.open",
        Proof,
        "Proof block closed with goals remaining"
    ),
    builtin_descriptor!(
        Error,
        310,
        "proof.counterexample.found",
        Proof,
        "Counterexample model found for the goal"
    ),
    builtin_descriptor!(
        Error,
        320,
        "proof.atp.timeout",
        Proof,
        "All ATP backends timed out"
    ),
    builtin_descriptor!(
        Error,
        321,
        "proof.atp.axiom_budget_exceeded",
        Proof,
        "Axiom set exceeds `max_axioms` limit for the obligation"
    ),
    builtin_descriptor!(
        Error,
        350,
        "proof.kernel.unsupported_evidence",
        Proof,
        "Legacy or backend proof material is unsupported under normal proof policy"
    ),
    builtin_descriptor!(
        Error,
        351,
        "proof.kernel.missing_provenance",
        Proof,
        "Kernel evidence is missing required provenance or context binding"
    ),
    builtin_descriptor!(
        Error,
        352,
        "proof.kernel.invalid_substitution",
        Proof,
        "Explicit substitution evidence failed kernel side conditions"
    ),
    builtin_descriptor!(
        Error,
        353,
        "proof.kernel.invalid_sat_refutation",
        Proof,
        "Kernel-derived SAT refutation check failed"
    ),
    builtin_descriptor!(
        Error,
        401,
        "logic.contradictory_axioms",
        Logic,
        "ATP derived `False` from declared axioms"
    ),
    builtin_descriptor!(
        Error,
        410,
        "logic.circular_definition",
        Logic,
        "Non-recursive definition refers to itself outside an algorithm block"
    ),
    builtin_descriptor!(
        Error,
        411,
        "logic.circular_cluster",
        Logic,
        "Cluster registration creates attribute inheritance cycle"
    ),
    builtin_descriptor!(
        Error,
        420,
        "vc.postcondition.return",
        Logic,
        "`ensures` not provable at `return` site"
    ),
    builtin_descriptor!(
        Error,
        421,
        "vc.assert.failed",
        Logic,
        "`assert` in algorithm body not provable"
    ),
    builtin_descriptor!(
        Error,
        422,
        "vc.precondition.call_site",
        Logic,
        "Callee `requires` clause not provable at call site"
    ),
    builtin_descriptor!(
        Error,
        423,
        "vc.loop.establish",
        Logic,
        "Loop invariant not provable before first iteration"
    ),
    builtin_descriptor!(
        Error,
        424,
        "vc.loop.maintain",
        Logic,
        "Loop invariant not provable to be preserved"
    ),
    builtin_descriptor!(
        Error,
        425,
        "vc.loop.decrease",
        Logic,
        "Termination measure not provably decreasing"
    ),
    builtin_descriptor!(
        Error,
        426,
        "vc.recursion.decrease",
        Logic,
        "Termination measure not provably decreasing at recursive call"
    ),
    builtin_descriptor!(
        Error,
        430,
        "logic.cluster.inconsistency",
        Logic,
        "Cluster registration creates contradiction"
    ),
    builtin_descriptor!(
        Warning,
        1,
        "warn.unused_variable",
        StructuralWarning,
        "Variable declared but never read"
    ),
    builtin_descriptor!(
        Warning,
        2,
        "warn.unused_definition",
        StructuralWarning,
        "Definition never referenced in package"
    ),
    builtin_descriptor!(
        Warning,
        3,
        "warn.unused_hypothesis",
        StructuralWarning,
        "Proof hypothesis never referenced in subsequent steps"
    ),
    builtin_descriptor!(
        Warning,
        10,
        "warn.deprecated_syntax",
        StructuralWarning,
        "Deprecated construct; replacement provided"
    ),
    builtin_descriptor!(
        Warning,
        101,
        "warn.redundant_hypothesis",
        ProofWarning,
        "`by` clause contains fact not used by accepted evidence"
    ),
    builtin_descriptor!(
        Warning,
        102,
        "warn.externally_attested_proof",
        ProofWarning,
        "External backend success without kernel-accepted evidence"
    ),
    builtin_descriptor!(
        Warning,
        103,
        "proof.citation.unused",
        ProofWarning,
        "Explicit citation absent from kernel-accepted `used_axioms`"
    ),
    builtin_descriptor!(
        Warning,
        201,
        "warn.unreachable_code",
        AlgorithmWarning,
        "Statement unreachable under static control-flow analysis"
    ),
    builtin_descriptor!(
        Warning,
        202,
        "warn.loop_may_not_terminate",
        AlgorithmWarning,
        "`terminating` algorithm with unverified loop measure"
    ),
    builtin_descriptor!(
        Warning,
        210,
        "warn.weakened_postcondition",
        AlgorithmWarning,
        "`ensures` weaker than what the verifier can prove"
    ),
    builtin_descriptor!(
        Warning,
        301,
        "compat.breaking_change",
        CompatibilityWarning,
        "Public API change requires a MAJOR bump"
    ),
    builtin_descriptor!(
        Warning,
        302,
        "compat.feature_addition",
        CompatibilityWarning,
        "Backward-compatible API addition requires a MINOR bump"
    ),
    builtin_descriptor!(
        Warning,
        303,
        "compat.overload_resolution_shift",
        CompatibilityWarning,
        "Registration, redefinition, or conditional-cluster change may shift overload/refinement resolution (heuristic MAJOR)"
    ),
    builtin_descriptor!(
        Warning,
        304,
        "compat.version_bump_insufficient",
        CompatibilityWarning,
        "Declared version bump smaller than required"
    ),
    builtin_descriptor!(
        Warning,
        305,
        "compat.edition_increase",
        CompatibilityWarning,
        "Package edition raised; MAJOR by default, review recommended"
    ),
];

/// Validated view of a descriptor slice.
#[derive(Clone, Copy, Debug)]
pub struct DiagnosticRegistry<'a> {
    descriptors: &'a [DiagnosticDescriptor],
}

impl<'a> DiagnosticRegistry<'a> {
    /// Validates against the built-in registry and creates a registry view.
    pub fn new(descriptors: &'a [DiagnosticDescriptor]) -> Result<Self, RegistryValidationError> {
        validate_registry_compatibility(BUILTIN_DESCRIPTORS, descriptors)?;
        Ok(Self { descriptors })
    }

    /// Returns the built-in spec-22 registry.
    pub fn builtin() -> Self {
        Self::new(BUILTIN_DESCRIPTORS).expect("built-in diagnostic registry is valid")
    }

    /// Returns the descriptors backing this registry.
    pub const fn descriptors(self) -> &'a [DiagnosticDescriptor] {
        self.descriptors
    }

    /// Looks up a descriptor by stable diagnostic code.
    pub fn lookup(self, code: DiagnosticCode) -> Option<&'a DiagnosticDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.code == code)
    }

    /// Looks up an active descriptor by current semantic name.
    pub fn lookup_semantic_name(
        self,
        phase_family: PhaseFamily,
        semantic_name: &str,
    ) -> Option<&'a DiagnosticDescriptor> {
        self.descriptors.iter().find(|descriptor| {
            descriptor.status == DiagnosticStatus::Active
                && descriptor.phase_family == phase_family
                && descriptor.semantic_name == semantic_name
        })
    }

    /// Looks up an active alias and returns the unique owning code.
    pub fn lookup_alias(self, phase_family: PhaseFamily, alias: &str) -> Option<DiagnosticCode> {
        self.descriptors
            .iter()
            .find(|descriptor| {
                descriptor.status == DiagnosticStatus::Active
                    && descriptor.phase_family == phase_family
                    && descriptor.aliases.contains(&alias)
            })
            .map(|descriptor| descriptor.code)
    }
}

/// Validates internal descriptor consistency.
pub fn validate_descriptors(
    descriptors: &[DiagnosticDescriptor],
) -> Result<(), RegistryValidationError> {
    let mut codes = BTreeMap::new();
    let mut lookup_names = BTreeMap::new();

    for descriptor in descriptors {
        if let Some(previous) = codes.insert(descriptor.code, *descriptor) {
            return Err(RegistryValidationError::DuplicateCode {
                code: descriptor.code,
                first_name: previous.semantic_name,
                second_name: descriptor.semantic_name,
            });
        }

        let Some(expected_family) = descriptor.code.phase_family() else {
            return Err(RegistryValidationError::CodeOutsideDefinedRange {
                code: descriptor.code,
            });
        };
        if expected_family == PhaseFamily::Info {
            return Err(RegistryValidationError::ReservedInfoCodeAllocated {
                code: descriptor.code,
            });
        }
        if descriptor.phase_family != expected_family {
            return Err(RegistryValidationError::PhaseFamilyMismatch {
                code: descriptor.code,
                expected: expected_family,
                actual: descriptor.phase_family,
            });
        }

        let expected_severity = expected_family.default_severity();
        if descriptor.default_severity != expected_severity {
            return Err(RegistryValidationError::SeverityMismatch {
                code: descriptor.code,
                expected: expected_severity,
                actual: descriptor.default_severity,
            });
        }

        match descriptor.status {
            DiagnosticStatus::Active if descriptor.retired_since.is_some() => {
                return Err(RegistryValidationError::ActiveWithRetiredSince {
                    code: descriptor.code,
                });
            }
            DiagnosticStatus::Active if !descriptor.replacement_codes.is_empty() => {
                return Err(RegistryValidationError::ActiveWithReplacementCodes {
                    code: descriptor.code,
                });
            }
            DiagnosticStatus::Retired if descriptor.retired_since.is_none() => {
                return Err(RegistryValidationError::RetiredWithoutRetiredSince {
                    code: descriptor.code,
                });
            }
            DiagnosticStatus::Active | DiagnosticStatus::Retired => {}
        }

        if descriptor.status == DiagnosticStatus::Active {
            insert_lookup_name(
                &mut lookup_names,
                descriptor.phase_family,
                descriptor.semantic_name,
                descriptor.code,
            )?;
            for alias in descriptor.aliases {
                insert_lookup_name(
                    &mut lookup_names,
                    descriptor.phase_family,
                    alias,
                    descriptor.code,
                )?;
            }
        }
    }

    Ok(())
}

/// Validates that `candidate` preserves compatibility with `baseline`.
pub fn validate_registry_compatibility(
    baseline: &[DiagnosticDescriptor],
    candidate: &[DiagnosticDescriptor],
) -> Result<(), RegistryValidationError> {
    validate_descriptors(baseline)?;
    validate_descriptors(candidate)?;

    for previous in baseline {
        let Some(next) = candidate
            .iter()
            .find(|descriptor| descriptor.code == previous.code)
        else {
            return Err(RegistryValidationError::MissingCode {
                code: previous.code,
            });
        };

        if previous.status == DiagnosticStatus::Retired && next.status == DiagnosticStatus::Active {
            return Err(RegistryValidationError::RetiredCodeReactivated {
                code: previous.code,
            });
        }
        if previous.meaning_key != next.meaning_key {
            return Err(RegistryValidationError::MeaningKeyChanged {
                code: previous.code,
                previous: previous.meaning_key,
                next: next.meaning_key,
            });
        }
        if previous.phase_family != next.phase_family {
            return Err(RegistryValidationError::PhaseFamilyChanged {
                code: previous.code,
                previous: previous.phase_family,
                next: next.phase_family,
            });
        }
        if previous.default_severity != next.default_severity {
            return Err(RegistryValidationError::DefaultSeverityChanged {
                code: previous.code,
                previous: previous.default_severity,
                next: next.default_severity,
            });
        }
        for previous_name in
            std::iter::once(previous.semantic_name).chain(previous.aliases.iter().copied())
        {
            if previous_name != next.semantic_name && !next.aliases.contains(&previous_name) {
                return Err(RegistryValidationError::SemanticRenameWithoutAlias {
                    code: previous.code,
                    previous: previous_name,
                    next: next.semantic_name,
                });
            }
        }
    }

    Ok(())
}

fn insert_lookup_name(
    lookup_names: &mut BTreeMap<(PhaseFamily, &'static str), DiagnosticCode>,
    phase_family: PhaseFamily,
    name: &'static str,
    code: DiagnosticCode,
) -> Result<(), RegistryValidationError> {
    let key = (phase_family, name);
    if let Some(previous_code) = lookup_names.insert(key, code) {
        return Err(RegistryValidationError::NameCollision {
            phase_family,
            name,
            first_code: previous_code,
            second_code: code,
        });
    }
    Ok(())
}

/// Registry validation or compatibility error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryValidationError {
    /// Two descriptors used the same code.
    DuplicateCode {
        /// Duplicated code.
        code: DiagnosticCode,
        /// First descriptor name.
        first_name: &'static str,
        /// Second descriptor name.
        second_name: &'static str,
    },
    /// A code belongs to no currently specified range.
    CodeOutsideDefinedRange {
        /// Offending code.
        code: DiagnosticCode,
    },
    /// A reserved info code was allocated before the language spec mapped it.
    ReservedInfoCodeAllocated {
        /// Offending code.
        code: DiagnosticCode,
    },
    /// Descriptor phase family does not match the code range.
    PhaseFamilyMismatch {
        /// Offending code.
        code: DiagnosticCode,
        /// Expected phase family from the code range.
        expected: PhaseFamily,
        /// Actual descriptor phase family.
        actual: PhaseFamily,
    },
    /// Descriptor severity does not match the code prefix/range.
    SeverityMismatch {
        /// Offending code.
        code: DiagnosticCode,
        /// Expected severity from the code range.
        expected: DiagnosticSeverity,
        /// Actual descriptor severity.
        actual: DiagnosticSeverity,
    },
    /// An active descriptor carried retirement metadata.
    ActiveWithRetiredSince {
        /// Offending code.
        code: DiagnosticCode,
    },
    /// An active descriptor carried replacement-code metadata.
    ActiveWithReplacementCodes {
        /// Offending code.
        code: DiagnosticCode,
    },
    /// A retired descriptor omitted retirement metadata.
    RetiredWithoutRetiredSince {
        /// Offending code.
        code: DiagnosticCode,
    },
    /// An active semantic name or alias collided within one phase family.
    NameCollision {
        /// Phase family where the collision happened.
        phase_family: PhaseFamily,
        /// Colliding name.
        name: &'static str,
        /// First owning code.
        first_code: DiagnosticCode,
        /// Second owning code.
        second_code: DiagnosticCode,
    },
    /// A baseline code disappeared instead of being retired.
    MissingCode {
        /// Missing code.
        code: DiagnosticCode,
    },
    /// A retired code was made active again.
    RetiredCodeReactivated {
        /// Reactivated code.
        code: DiagnosticCode,
    },
    /// A descriptor changed its stable meaning key.
    MeaningKeyChanged {
        /// Offending code.
        code: DiagnosticCode,
        /// Previous meaning key.
        previous: &'static str,
        /// New meaning key.
        next: &'static str,
    },
    /// A descriptor changed phase family.
    PhaseFamilyChanged {
        /// Offending code.
        code: DiagnosticCode,
        /// Previous phase family.
        previous: PhaseFamily,
        /// New phase family.
        next: PhaseFamily,
    },
    /// A descriptor changed default severity.
    DefaultSeverityChanged {
        /// Offending code.
        code: DiagnosticCode,
        /// Previous severity.
        previous: DiagnosticSeverity,
        /// New severity.
        next: DiagnosticSeverity,
    },
    /// A semantic-name rename did not preserve a previous name as an alias.
    SemanticRenameWithoutAlias {
        /// Offending code.
        code: DiagnosticCode,
        /// Previous semantic name.
        previous: &'static str,
        /// New semantic name.
        next: &'static str,
    },
}

impl fmt::Display for RegistryValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCode {
                code,
                first_name,
                second_name,
            } => write!(
                formatter,
                "diagnostic code {code} is allocated twice: {first_name} and {second_name}"
            ),
            Self::CodeOutsideDefinedRange { code } => {
                write!(
                    formatter,
                    "diagnostic code {code} has no specified code range"
                )
            }
            Self::ReservedInfoCodeAllocated { code } => {
                write!(
                    formatter,
                    "reserved info diagnostic code {code} was allocated"
                )
            }
            Self::PhaseFamilyMismatch {
                code,
                expected,
                actual,
            } => write!(
                formatter,
                "diagnostic code {code} belongs to phase family {expected}, not {actual}"
            ),
            Self::SeverityMismatch {
                code,
                expected,
                actual,
            } => write!(
                formatter,
                "diagnostic code {code} has default severity {expected}, not {actual}"
            ),
            Self::ActiveWithRetiredSince { code } => {
                write!(formatter, "active diagnostic code {code} has retired_since")
            }
            Self::ActiveWithReplacementCodes { code } => {
                write!(
                    formatter,
                    "active diagnostic code {code} has replacement codes"
                )
            }
            Self::RetiredWithoutRetiredSince { code } => {
                write!(
                    formatter,
                    "retired diagnostic code {code} is missing retired_since"
                )
            }
            Self::NameCollision {
                phase_family,
                name,
                first_code,
                second_code,
            } => write!(
                formatter,
                "diagnostic name `{name}` in phase family {phase_family} is shared by {first_code} and {second_code}"
            ),
            Self::MissingCode { code } => {
                write!(
                    formatter,
                    "diagnostic code {code} disappeared instead of being retired"
                )
            }
            Self::RetiredCodeReactivated { code } => {
                write!(formatter, "retired diagnostic code {code} was reactivated")
            }
            Self::MeaningKeyChanged {
                code,
                previous,
                next,
            } => write!(
                formatter,
                "diagnostic code {code} changed meaning key from `{previous}` to `{next}`"
            ),
            Self::PhaseFamilyChanged {
                code,
                previous,
                next,
            } => write!(
                formatter,
                "diagnostic code {code} changed phase family from {previous} to {next}"
            ),
            Self::DefaultSeverityChanged {
                code,
                previous,
                next,
            } => write!(
                formatter,
                "diagnostic code {code} changed default severity from {previous} to {next}"
            ),
            Self::SemanticRenameWithoutAlias {
                code,
                previous,
                next,
            } => write!(
                formatter,
                "diagnostic code {code} renamed `{previous}` to `{next}` without an alias"
            ),
        }
    }
}

impl Error for RegistryValidationError {}

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Stage {
    Lexical,
    ParseOnly,
    DeclarationSymbol,
    TypeElaboration,
    FormulaStatement,
    ProofVerification,
    AdvancedSemantics,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lexical => "lexical",
            Self::ParseOnly => "parse_only",
            Self::DeclarationSymbol => "declaration_symbol",
            Self::TypeElaboration => "type_elaboration",
            Self::FormulaStatement => "formula_statement",
            Self::ProofVerification => "proof_verification",
            Self::AdvancedSemantics => "advanced_semantics",
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Stage {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "lexical" => Ok(Self::Lexical),
            "parse_only" => Ok(Self::ParseOnly),
            "declaration_symbol" => Ok(Self::DeclarationSymbol),
            "type_elaboration" => Ok(Self::TypeElaboration),
            "formula_statement" => Ok(Self::FormulaStatement),
            "proof_verification" => Ok(Self::ProofVerification),
            "advanced_semantics" => Ok(Self::AdvancedSemantics),
            other => Err(format!("unknown stage `{other}`")),
        }
    }
}

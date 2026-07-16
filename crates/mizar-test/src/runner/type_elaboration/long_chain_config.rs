use super::source_formula::{
    SourceReservedVariableBuiltinType, SourceReservedVariableModeDefinition,
    SourceReservedVariableModeRadix,
};

pub(in crate::runner) const SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS:
    &[SourceReservedVariableModeDefinition] = &[
    SourceReservedVariableModeDefinition {
        label: "BaseModeDef",
        spelling: "BaseMode",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode1Def",
        spelling: "ChainMode1",
        radix: SourceReservedVariableModeRadix::Mode("BaseMode"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode2Def",
        spelling: "ChainMode2",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode1"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode3Def",
        spelling: "ChainMode3",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode2"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode4Def",
        spelling: "ChainMode4",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode3"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode5Def",
        spelling: "ChainMode5",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode4"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode6Def",
        spelling: "ChainMode6",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode5"),
    },
];

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS:
    &[SourceReservedVariableModeDefinition] = &[
    SourceReservedVariableModeDefinition {
        label: "BaseObjectModeDef",
        spelling: "BaseObjectMode",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode1Def",
        spelling: "ChainObjectMode1",
        radix: SourceReservedVariableModeRadix::Mode("BaseObjectMode"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode2Def",
        spelling: "ChainObjectMode2",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode1"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode3Def",
        spelling: "ChainObjectMode3",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode2"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode4Def",
        spelling: "ChainObjectMode4",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode3"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode5Def",
        spelling: "ChainObjectMode5",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode4"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode6Def",
        spelling: "ChainObjectMode6",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode5"),
    },
];

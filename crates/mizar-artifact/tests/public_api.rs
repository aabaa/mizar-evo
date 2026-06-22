use mizar_artifact::{
    store::CanonicalJson,
    verified_artifact::{
        VerifiedArtifact, VerifiedArtifactError, implementation_hash_input_json,
        interface_hash_input_json,
    },
};

type HashInputHelper = fn(&VerifiedArtifact) -> Result<CanonicalJson, VerifiedArtifactError>;

#[test]
fn verified_artifact_hash_input_helpers_are_public_api() {
    fn accepts_public_helpers(_interface: HashInputHelper, _implementation: HashInputHelper) {}

    accepts_public_helpers(interface_hash_input_json, implementation_hash_input_json);
}

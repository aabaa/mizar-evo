use mizar_doc::{math::bundled_math_assets, render::validate_math_assets};

#[test]
fn opaque_math_asset_handoff_requires_the_complete_exact_bound_set() {
    let assets = bundled_math_assets();
    assert_eq!(assets, bundled_math_assets());
    validate_math_assets(assets).unwrap();
    assert!(
        assets
            .iter()
            .all(|(path, _)| path.starts_with("assets/katex/"))
    );
    for (index, (path, bytes)) in assets.iter().enumerate() {
        assert!(!bytes.is_empty(), "{path}");
        let mut missing = assets.to_vec();
        missing.remove(index);
        assert!(validate_math_assets(&missing).is_err(), "missing {path}");
        let mut corrupted = bytes.to_vec();
        corrupted[0] ^= 1;
        let mut changed = assets.to_vec();
        changed[index] = (path, &corrupted);
        assert!(validate_math_assets(&changed).is_err(), "corrupt {path}");
    }
    let mut extra = assets.to_vec();
    extra.push(("assets/katex/unbound.js", b"unbound"));
    assert!(validate_math_assets(&extra).is_err());
    let mut duplicate = assets.to_vec();
    duplicate.push(assets[0]);
    assert!(validate_math_assets(&duplicate).is_err());
    let css = assets
        .iter()
        .find(|(p, _)| p.ends_with("/katex.min.css"))
        .unwrap();
    let mut mixed = assets.to_vec();
    let runtime = mixed
        .iter()
        .position(|(p, _)| p.ends_with("/katex.min.js"))
        .unwrap();
    mixed[runtime].1 = css.1;
    assert!(validate_math_assets(&mixed).is_err());
}

#[test]
fn opaque_math_configuration_binds_all_safety_limits_and_runtime_options() {
    let bytes = bundled_math_assets()
        .iter()
        .find(|(path, _)| path.ends_with("/config.json"))
        .unwrap()
        .1;
    let config: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(config["release"], "0.19.0");
    assert_eq!(config["payload_limit"], 65536);
    assert_eq!(
        config["options"],
        serde_json::json!({
            "output":"htmlAndMathml", "throwOnError":true, "trust":false,
            "strict":"ignore", "maxExpand":1000, "maxSize":10,
            "globalGroup":false, "leqno":false, "fleqn":false,
            "minRuleThickness":0, "colorIsTextColor":false, "errorColor":"#cc0000"
        })
    );
}

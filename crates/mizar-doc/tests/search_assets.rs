use mizar_doc::render::{bundled_search_assets, validate_search_assets};

#[test]
fn search_asset_handoff_requires_the_exact_reproducible_bound_set() {
    let assets = bundled_search_assets();
    assert_eq!(assets, bundled_search_assets());
    assert_eq!(
        assets,
        &[
            (
                "assets/style.css",
                include_bytes!("../assets/style.css").as_slice()
            ),
            (
                "assets/search.js",
                include_bytes!("../assets/search.js").as_slice()
            ),
            (
                "assets/search-manifest.json",
                include_bytes!("../assets/search-manifest.json").as_slice(),
            ),
        ]
    );
    validate_search_assets(assets).unwrap();
    for (index, (path, bytes)) in assets.iter().enumerate() {
        assert!(!bytes.is_empty(), "{path}");
        let mut missing = assets.to_vec();
        missing.remove(index);
        assert!(validate_search_assets(&missing).is_err(), "missing {path}");
        let mut corrupted = bytes.to_vec();
        corrupted[0] ^= 1;
        let mut changed = assets.to_vec();
        changed[index] = (path, &corrupted);
        assert!(validate_search_assets(&changed).is_err(), "changed {path}");
        let mut duplicate = assets.to_vec();
        duplicate.push(assets[index]);
        assert!(
            validate_search_assets(&duplicate).is_err(),
            "duplicate {path}"
        );
        for (other, (_, replacement)) in assets.iter().enumerate() {
            if other != index {
                let mut swapped = assets.to_vec();
                swapped[index].1 = replacement;
                assert!(validate_search_assets(&swapped).is_err(), "swapped {path}");
            }
        }
    }
    let mut extra = assets.to_vec();
    extra.push(("assets/unbound.js", b"unbound"));
    assert!(validate_search_assets(&extra).is_err());
    let mut reordered = assets.to_vec();
    reordered.reverse();
    validate_search_assets(&reordered).unwrap();
}

#[test]
fn manifest_binds_only_the_local_style_and_search_runtime() {
    let bytes = bundled_search_assets()
        .iter()
        .find(|(path, _)| *path == "assets/search-manifest.json")
        .unwrap()
        .1;
    let manifest: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(manifest["format_version"], 1);
    let records = manifest["assets"].as_array().unwrap();
    assert_eq!(records.len(), 2);
    for (record, path) in records.iter().zip(["assets/style.css", "assets/search.js"]) {
        assert_eq!(record["path"], path);
        let digest = record["sha256"].as_str().unwrap();
        assert_eq!(digest.len(), 64);
        assert!(
            digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        );
    }
}

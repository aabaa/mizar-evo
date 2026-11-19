use super::*;

#[test]
fn doc13_summary_selects_source_ordered_ordinary_structural_paragraphs() {
    use mizar_doc::{comments::CommentTarget, markdown::parse_comments, render::summary_text};
    for (markdown, expected) in [
        ("@returns\n\n@returns duplicate\n\nFirst\n\nSecond", "First"),
        ("- @param\n- tight first\n- tight second", "tight first"),
        ("- @since\n\n- loose first\n\n- loose second", "loose first"),
        ("> @see\n>\n> quoted first\n\nOutside", "quoted first"),
        (
            "# heading\n\n    indented\n\n```\nfenced\n```\n\n<div>\nhtml block\n</div>\n\n[definition]: /url\n\n$$display$$\n\nEligible",
            "Eligible",
        ),
        ("@unknown untouched", "@unknown untouched"),
        ("\\@param escaped", "@param escaped"),
        ("&#64;param entity", "@param entity"),
        ("`@param` code", "@param code"),
        ("*@param* emphasis", "@param emphasis"),
        ("[@param](/url) link", "@param link"),
        ("@PARAM uppercase", "@PARAM uppercase"),
        ("@param, punctuation", "@param, punctuation"),
        ("@param\u{a0}unicode", "@param\u{a0}unicode"),
    ] {
        let (root, plan, _) = doc8_fixture(markdown);
        let attachments = doc7_attach(&plan);
        let parsed = parse_comments(&attachments, &[]);
        assert_eq!(
            summary_text(&parsed, CommentTarget::Module)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{markdown:?}"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    let (root, plan, _) = doc7_fixture(
        "summary_blocks",
        "::: @returns\n\n::: First\n\n::: Second\n",
        None,
        |_, _, _| {},
    );
    let attachments = doc7_attach(&plan);
    let parsed = parse_comments(&attachments, &[]);
    assert_eq!(parsed.blocks.len(), 3);
    assert_eq!(
        summary_text(&parsed, CommentTarget::Module)
            .unwrap()
            .as_deref(),
        Some("First")
    );
    let mut attachments = doc7_attach(&plan);
    attachments.blocks[1].lines = attachments.blocks[2].lines.clone();
    let parsed = parse_comments(&attachments, &[]);
    assert!(summary_text(&parsed, CommentTarget::Module).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc13_summary_distinguishes_absent_empty_and_unavailable_views() {
    use mizar_doc::{
        comments::{CommentTarget, attach_comments},
        markdown::parse_comments,
        render::{module_summary_fragments, search_summaries, summary_text},
    };
    for (markdown, expected) in [
        ("", None),
        ("\n", Some("")),
        ("@param\n\n@returns", Some("")),
        ("# heading\n\n```\ncode\n```\n\n$$x$$", Some("")),
        ("[](/url)\n\nLater text", Some("")),
    ] {
        let (root, plan, _) = doc8_fixture(markdown);
        let attachments = doc7_attach(&plan);
        let parsed = parse_comments(&attachments, &[]);
        assert_eq!(
            summary_text(&parsed, CommentTarget::Module)
                .unwrap()
                .as_deref(),
            expected,
            "{markdown:?}"
        );
        if markdown.is_empty() {
            assert!(
                search_summaries(&parsed)
                    .unwrap()
                    .iter()
                    .all(|(_, text)| text.is_none())
            );
            assert!(
                module_summary_fragments(&parsed)
                    .unwrap()
                    .iter()
                    .all(|(_, html)| html == "<p class=\"mizar-summary\"></p>\n")
            );
        }
        let foreign = attachments.input.exports[0].clone();
        assert!(summary_text(&parsed, CommentTarget::Item(&foreign)).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    let (root, plan, _) = doc8_fixture("Available");
    let attachments = doc7_attach(&plan);
    let mut parsed = parse_comments(&attachments, &[]);
    parsed.blocks[0].fallback = true;
    assert!(summary_text(&parsed, CommentTarget::Module).is_err());
    let mut parsed = parse_comments(&attachments, &[]);
    parsed.blocks[0].events.clear();
    assert!(summary_text(&parsed, CommentTarget::Module).is_err());
    assert!(search_summaries(&parsed).is_err());
    assert!(module_summary_fragments(&parsed).is_err());
    let mut parsed = parse_comments(&attachments, &[]);
    parsed.blocks.clear();
    assert!(summary_text(&parsed, CommentTarget::Module).is_err());
    assert!(search_summaries(&parsed).is_err());
    assert!(module_summary_fragments(&parsed).is_err());
    let absent = plan
        .index
        .modules
        .keys()
        .find(|m| m.module_path == "module_1")
        .unwrap();
    let unavailable = attach_comments(&plan, absent, None).unwrap();
    let parsed = parse_comments(&unavailable, &[]);
    assert!(summary_text(&parsed, CommentTarget::Module).is_err());
    assert!(search_summaries(&parsed).is_err());
    assert!(module_summary_fragments(&parsed).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc13_summary_skips_whole_failed_views_in_attachment_order() {
    use mizar_artifact::store::CanonicalJson as J;
    use mizar_doc::{
        comments::{CommentTarget, attach_comments},
        markdown::parse_comments,
        render::summary_text,
    };
    for (source_text, failed, expected) in [
        ("::: Failed\n\n::: Alpha\n\n::: Beta\n", vec![0], "Alpha"),
        ("::: Alpha\n\n::: Failed\n\n::: Beta\n", vec![1], "Alpha"),
        (
            "::: Failed one\n\n::: @returns\n\n::: Failed two\n\n::: Beta\n",
            vec![0, 2],
            "Beta",
        ),
        ("::: Failed one\n\n::: Failed two\n", vec![0, 1], ""),
    ] {
        let (root, plan, source) =
            doc7_fixture("summary_failure_order", source_text, None, |_, _, _| {});
        let mut attachments =
            attach_comments(&plan, doc7_attach(&plan).module, Some(&source)).unwrap();
        let mut faulty_rows = failed
            .iter()
            .map(|&i| attachments.blocks[i].lines[0].clone())
            .collect::<Vec<_>>();
        for row in &mut faulty_rows {
            let J::Array(fields) = row else {
                panic!("documentation row")
            };
            let J::String(raw) = &fields[2] else {
                panic!("raw spelling")
            };
            fields[1] = doc7_range(1, 1 + raw.len());
        }
        for (&i, row) in failed.iter().zip(&faulty_rows) {
            attachments.blocks[i].lines[0] = row;
        }
        let parsed = parse_comments(&attachments, &[]);
        assert_eq!(
            parsed.blocks.iter().filter(|b| b.fallback).count(),
            failed.len()
        );
        assert_eq!(
            parsed
                .warnings
                .iter()
                .filter(|w| w.code().to_string() == "W0411")
                .count(),
            failed.len()
        );
        let before = format!("{parsed:?}");
        assert_eq!(
            summary_text(&parsed, CommentTarget::Module)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{source_text:?}"
        );
        assert!(!parsed.documentation_failed());
        assert_eq!(before, format!("{parsed:?}"));
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn doc13_summary_flattens_exact_plain_text_without_normalizing_or_truncating() {
    use mizar_doc::{comments::CommentTarget, markdown::parse_comments, render::summary_text};
    let long = "é e\u{301} λ\u{a0}文".repeat(2048);
    for (markdown, expected) in [
        ("**Bold** `x &amp; <y>` [link *label*](/dest) ![image **alt**](/img) &amp; <i a=\"&amp;\">raw</i>".to_owned(), "Bold x &amp; <y> link label image alt & <i a=\"&amp;\">raw</i>".to_owned()),
        (r"$*opaque* &amp; \alpha \<script\>$".into(), r"*opaque* &amp; \alpha \<script\>".into()),
        ("a\t  b\nc  \nd\\\ne\u{b}\u{c}f".into(), "a b c d e f".into()),
        (format!("{long}\n\nIgnored"), long),
    ] {
        let (root, plan, _) = doc8_fixture(&markdown);
        let attachments = doc7_attach(&plan);
        let parsed = parse_comments(&attachments, &[]);
        assert_eq!(summary_text(&parsed, CommentTarget::Module).unwrap(), Some(expected), "{markdown:?}");
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn doc13_summary_module_search_preserve_published_export_order_and_identities() {
    use mizar_artifact::{store::CanonicalJson as J, verified_artifact::ExportVisibility};
    use mizar_doc::{
        comments::{CommentTarget, attach_comments},
        doc_build::DocBuildError,
        markdown::parse_comments,
        render::{module_summary_fragments, search_summaries, summary_text},
    };
    let text = "::: Module prose\nimport X;\n::: @returns\n:::\n::: A &amp; `&amp;` <i>item</i>\ntheorem T: true;\n";
    let (root, mut plan, source) = doc7_fixture(
        "summary_exports",
        text,
        Some(("theorem", "TheoremItem", "theorem T: true;")),
        |artifact, _, landmarks| {
            let mut undocumented = artifact.exports[0].clone();
            undocumented.origin_id.push_str("-undocumented");
            let mut reexport = undocumented.clone();
            reexport.origin_id.push_str("-reexport");
            reexport.visibility = ExportVisibility::Reexported;
            artifact.exports.extend([undocumented, reexport]);
            let start = text.find("import").unwrap();
            landmarks.insert(0, doc7_landmark("ImportItem", start, start + 9, None, None));
        },
    );
    for promote in [false, true] {
        plan.profile.fail_on_doc_warnings = promote;
        let attachments = attach_comments(&plan, doc7_attach(&plan).module, Some(&source)).unwrap();
        let parsed = parse_comments(&attachments, &[]);
        assert_eq!(doc8_codes(&parsed.warnings), ["W0406"]);
        assert_eq!(parsed.documentation_failed(), promote);
        let before = format!("{parsed:?}");
        let search = search_summaries(&parsed).unwrap();
        let module = module_summary_fragments(&parsed).unwrap();
        assert_eq!(search.len(), 3);
        assert_eq!(module.len(), search.len());
        for (index, ((search_export, summary), (module_export, html))) in
            search.iter().zip(&module).enumerate()
        {
            let original = &attachments.input.exports[index];
            assert!(std::ptr::eq(*search_export, original));
            assert!(std::ptr::eq(*module_export, original));
            assert_eq!(
                summary_text(&parsed, CommentTarget::Item(original)).unwrap(),
                *summary
            );
            if std::ptr::eq(original, &attachments.input.exports[0]) {
                assert_eq!(summary.as_deref(), Some("A & &amp; <i>item</i>"));
                assert_eq!(
                    html,
                    "<p class=\"mizar-summary\">A &amp; &amp;amp; &lt;i&gt;item&lt;/i&gt;</p>\n"
                );
            } else {
                assert_eq!(summary, &None);
                assert_eq!(html, "<p class=\"mizar-summary\"></p>\n");
            }
        }
        assert_eq!(
            summary_text(&parsed, CommentTarget::Module)
                .unwrap()
                .as_deref(),
            Some("Module prose")
        );
        assert_eq!(before, format!("{parsed:?}"));
    }
    // The same genuine item publication remains present in both consumers when
    // its only view fails, while the undocumented identities remain absent.
    let mut attachments =
        attach_comments(&plan, doc7_attach(&plan).module, Some(&source)).unwrap();
    let mut faulty_row = attachments.blocks[1].lines[0].clone();
    let mizar_artifact::store::CanonicalJson::Array(fields) = &mut faulty_row else {
        panic!("documentation row")
    };
    let mizar_artifact::store::CanonicalJson::String(raw) = &fields[2] else {
        panic!("raw spelling")
    };
    fields[1] = doc7_range(1, 1 + raw.len());
    attachments.blocks[1].lines[0] = &faulty_row;
    let parsed = parse_comments(&attachments, &[]);
    assert_eq!(doc8_codes(&parsed.warnings), ["W0411"]);
    let search = search_summaries(&parsed).unwrap();
    let module = module_summary_fragments(&parsed).unwrap();
    assert_eq!(search.len(), attachments.input.exports.len());
    assert_eq!(module.len(), search.len());
    assert_eq!(search[0].1.as_deref(), Some(""));
    assert_eq!(module[0].1, "<p class=\"mizar-summary\"></p>\n");
    assert!(search[1..].iter().all(|(_, text)| text.is_none()));
    let mut attachments = attach_comments(&plan, doc7_attach(&plan).module, Some(&source)).unwrap();
    let J::Array(fields) = &mut faulty_row else {
        panic!("documentation row")
    };
    fields[2] = J::string("::: forged\n");
    attachments.blocks[1].lines[0] = &faulty_row;
    let parsed = parse_comments(&attachments, &[]);
    assert!(parsed.blocks[1].fallback);
    assert_eq!(doc8_codes(&parsed.warnings), ["W0411"]);
    assert!(matches!(
        summary_text(&parsed, CommentTarget::Item(&attachments.input.exports[0])),
        Err(DocBuildError::Binding(_))
    ));
    assert!(matches!(
        search_summaries(&parsed),
        Err(DocBuildError::Binding(_))
    ));
    assert!(matches!(
        module_summary_fragments(&parsed),
        Err(DocBuildError::Binding(_))
    ));
    std::fs::remove_dir_all(root).unwrap();
}

// Spec24.3: raw delimiter eligibility and protected boundaries are independent
// of whether CommonMark emits decoded/coalesced text events.
#[test]
fn doc8_inline_math_respects_raw_escaping_runs_paragraphs_and_slots() {
    for (markdown, expected, unmatched) in [
        (r"\$escaped\$ and \\$paired$", vec!["paired"], 0),
        (r"$a\$b$", vec![r"a\$b"], 0),
        ("&#36;literal&#36;", vec![], 0),
        ("cost $2 and $3", vec!["2 and "], 0),
        ("$$$literal$$$ $$$$", vec![], 0),
        ("$a$$b$", vec!["a$$b"], 0),
        ("$a\nb$", vec!["a\nb"], 0),
        ("$a\n\nb$", vec![], 2),
        ("@param $opaque$ prose $body$", vec!["body"], 0),
        ("@see $opaque$", vec![], 0),
        ("@since $opaque$", vec![], 0),
        ("@param $unclosed prose $body$", vec!["body"], 0),
        ("@deprecated $opaque$ explanation $body$", vec!["body"], 0),
        ("$*opaque* \\broken{$", vec!["*opaque* \\broken{"], 0),
    ] {
        let (root, plan, _) = doc8_fixture(markdown);
        let attachments = doc7_attach(&plan);
        let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
        let block = &parsed.blocks[0];
        assert!(!block.fallback, "{markdown:?}");
        let contents: Vec<_> = block
            .math
            .iter()
            .map(|math| {
                assert!(!math.display, "{markdown:?}");
                math.content
                    .iter()
                    .map(|r| &block.text[r.clone()])
                    .collect::<String>()
            })
            .collect();
        assert_eq!(contents, expected, "{markdown:?}");
        assert_eq!(
            doc8_codes(&parsed.warnings),
            vec!["W0409"; unmatched],
            "{markdown:?}"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn doc8_inline_math_preserves_commonmark_barriers_and_label_local_pairs() {
    for (markdown, expected, unmatched) in [
        ("$`code`$", vec![], 2),
        ("$<i>html</i>$", vec![], 2),
        ("$[text](ref)$", vec![], 2),
        ("$![text](ref)$", vec![], 2),
        ("$<https://example.test/>$", vec![], 2),
        ("[$label$](dest \"$title$\")", vec!["label"], 0),
        ("![$label$](dest \"$title$\")", vec!["label"], 0),
        ("<https://example.test/$raw$>", vec![], 0),
        ("[ref]: /$raw$ \"$title$\"", vec![], 0),
        ("`$raw$` <i data-v='$raw$'>x</i>", vec![], 0),
    ] {
        let (root, plan, _) = doc8_fixture(markdown);
        let attachments = doc7_attach(&plan);
        let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
        let block = &parsed.blocks[0];
        let contents: Vec<_> = block
            .math
            .iter()
            .map(|math| {
                math.content
                    .iter()
                    .map(|r| &block.text[r.clone()])
                    .collect::<String>()
            })
            .collect();
        assert_eq!(contents, expected, "{markdown:?}");
        assert_eq!(
            doc8_codes(&parsed.warnings),
            vec!["W0409"; unmatched],
            "{markdown:?}"
        );
        assert_eq!(block.text, format!("{markdown}\n"));
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn doc8_display_math_stays_in_its_container_and_preserves_recovery() {
    for (markdown, expected, codes) in [
        ("  $$x$$\t", vec!["x"], vec![]),
        (r"$$x\$\$y$$", vec![r"x\$\$y"], vec![]),
        ("> $$\n> λ\n> $$", vec!["λ\n"], vec![]),
        ("- $$\n  x\n  $$", vec!["x\n"], vec![]),
        ("$$\n# heading\n$$", vec!["# heading\n"], vec![]),
        ("$$\n- child\n\n$$", vec!["- child\n\n"], vec![]),
        (
            "$$\n@param hidden\n$opaque$\n$$",
            vec!["@param hidden\n$opaque$\n"],
            vec![],
        ),
        ("$$ \t$$", vec![], vec!["W0410"]),
        ("$$\u{a0}$$", vec!["\u{a0}"], vec![]),
        ("$$\n \t\n$$", vec![], vec!["W0410"]),
        ("$$x$$y$$", vec![], vec![]),
        ("$$$x$$$", vec![], vec![]),
        ("```\n$$\n```", vec![], vec![]),
        ("    $$", vec![], vec![]),
        ("<pre>\n$$\n</pre>", vec![], vec![]),
        ("[ref]: /$$", vec![], vec![]),
        ("$$\n\n```\nx\n```\n\n$$", vec![], vec!["W0409", "W0409"]),
        (
            "$$\n\n<pre>\nx\n</pre>\n\n$$",
            vec![],
            vec!["W0409", "W0409"],
        ),
        ("$$\n\n[ref]: /url\n\n$$", vec![], vec!["W0409", "W0409"]),
        ("- $$\n- $$", vec![], vec!["W0409", "W0409"]),
        ("> $$\n\n$$", vec![], vec!["W0409", "W0409"]),
        ("$$\n*literal recovery", vec![], vec!["W0409"]),
        ("$$\nx\n    $$", vec![], vec!["W0409"]),
        ("``` unclosed\n$literal", vec![], vec![]),
    ] {
        let (root, plan, _) = doc8_fixture(markdown);
        let attachments = doc7_attach(&plan);
        let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
        let block = &parsed.blocks[0];
        assert!(!block.fallback, "{markdown:?}");
        assert!(block.fields.is_empty(), "{markdown:?}");
        let contents: Vec<_> = block
            .math
            .iter()
            .map(|math| {
                assert!(math.display, "{markdown:?}");
                math.content
                    .iter()
                    .map(|r| &block.text[r.clone()])
                    .collect::<String>()
            })
            .collect();
        assert_eq!(contents, expected, "{markdown:?}");
        assert_eq!(doc8_codes(&parsed.warnings), codes, "{markdown:?}");
        assert_eq!(block.text, format!("{markdown}\n"));
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn doc8_duplicate_reference_definitions_remain_math_barriers() {
    use mizar_diagnostics::failure_record::DiagnosticPrimaryLocation;

    for (case, (markdown, expected, unmatched)) in [
        ("[r]: /one\n\n[r]: /two \"title\n$$\nend\"\n", vec![], 0),
        (
            "[r]: /one\n\n[r]: /two \"title\n$$\nx\n$$\nend\"\n",
            vec![],
            0,
        ),
        (
            "> [R]: /one\n>\n> [r]: /two \"title\n> $$\n> x\n> $$\n> end\"\n",
            vec![],
            0,
        ),
        (
            "- [R]: /one\n\n  [r]: /two \"title\n  $$\n  x\n  $$\n  end\"\n",
            vec![],
            0,
        ),
        (
            "[r]: /one\n\n[r]: /two \"title\n$$\nx\n$$\nend\"\n\n$$\ny\n$$\n\n$z$ [r]\n",
            vec![(true, "y\n"), (false, "z")],
            0,
        ),
        (
            "[r]: /one\n\n$$\n\n[r]: /two \"title\n$$\nx\n$$\nend\"\n\n$$\n",
            vec![],
            2,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (root, plan, source) = doc8_fixture(markdown);
        let unbound = doc7_attach(&plan);
        let attachments =
            mizar_doc::comments::attach_comments(&plan, unbound.module, Some(&source)).unwrap();
        let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
        let block = &parsed.blocks[0];
        assert!(!block.fallback, "case {case}");
        assert!(block.fields.is_empty(), "case {case}");
        assert_eq!(block.text, markdown, "case {case}");
        assert_eq!(
            block
                .math
                .iter()
                .map(|math| (
                    math.display,
                    math.content
                        .iter()
                        .map(|r| &block.text[r.clone()])
                        .collect::<String>()
                ))
                .collect::<Vec<_>>(),
            expected
                .into_iter()
                .map(|(display, text)| (display, text.to_owned()))
                .collect::<Vec<_>>(),
            "case {case}"
        );
        assert_eq!(doc8_codes(&parsed.warnings), vec!["W0409"; unmatched]);
        assert_eq!(block.source_map.len(), markdown.lines().count());
        for (line, (logical, loaded)) in block.source_map.iter().enumerate() {
            assert_eq!(loaded.start, logical.start + 4 * (line + 1));
            assert_eq!(loaded.end, logical.end + 4 * (line + 1));
            assert_eq!(
                &block.text[logical.clone()],
                &source.text[loaded.start..loaded.end]
            );
        }
        // Reference-only containers retain the pinned CommonMark envelope events.
        match case {
            0 | 1 => assert!(block.events.is_empty()),
            2 => assert_eq!(format!("{:?}", block.events), "[(Start(BlockQuote(None)), 0..54), (End(BlockQuote(None)), 0..54)]"),
            3 => assert_eq!(format!("{:?}", block.events), "[(Start(List(None)), 0..53), (Start(Item), 0..53), (End(Item), 0..53), (End(List(false)), 0..53)]"),
            4 => {
                // Compare the retained link event to an unambiguous first-definition baseline.
                let (baseline_root, baseline_plan, _) = doc8_fixture("[r]: /one\n\n[r]\n");
                let baseline_attachments = doc7_attach(&baseline_plan);
                let baseline = mizar_doc::markdown::parse_comments(&baseline_attachments, &[]);
                assert_eq!(block.events[9].0, baseline.blocks[0].events[1].0);
                assert_eq!(block.events[9].1, 55..58);
                std::fs::remove_dir_all(baseline_root).unwrap();
            }
            _ => {
                for (warning, logical) in parsed.warnings.iter().zip([11, 46]) {
                    let loaded = block.source_map.iter().find_map(|(q3, loaded)| q3.contains(&logical).then_some(loaded.start + logical - q3.start)).unwrap();
                    assert!(matches!(warning.primary_location(), DiagnosticPrimaryLocation::Span(span) if span.range() == mizar_session::SourceRange { source_id: source.source_id, start: loaded, end: loaded + 2 }));
                }
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn doc8_q3_maps_keep_unicode_newlines_container_gaps_and_marker_locations() {
    use mizar_diagnostics::failure_record::{
        DiagnosticDetailValue as V, DiagnosticPrimaryLocation,
    };
    use mizar_doc::comments::attach_comments;
    let text = "\u{feff}import X;\r\n:: removed λ\r\n::: > $α\r\n::: > β$\r\n::: >\r\n::: > @returns\r\n::: >\r\n::: > @see\r\n::: >\r\n::: > $orphan\r\ntheorem T: true;\r\n";
    let (root, plan, source) = doc7_fixture(
        "doc8_maps",
        text,
        Some(("theorem", "TheoremItem", "theorem T: true;")),
        |_, _, landmarks| {
            landmarks.insert(0, doc7_landmark("ImportItem", 0, 9, None, None));
        },
    );
    let module = plan
        .index
        .modules
        .keys()
        .find(|m| m.module_path == "module_0")
        .unwrap();
    let attachments = attach_comments(&plan, module, Some(&source)).unwrap();
    let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
    let block = &parsed.blocks[0];
    assert_eq!(
        block.text,
        "> $α\n> β$\n>\n> @returns\n>\n> @see\n>\n> $orphan\n"
    );
    assert!(!block.fallback);
    assert_eq!(block.source_map.len(), 8);
    for (logical, loaded) in &block.source_map {
        assert_eq!(
            &block.text[logical.clone()],
            &source.text[loaded.start..loaded.end]
        );
        assert!(!source.text[loaded.start..loaded.end].contains(":::"));
    }
    let math = &block.math[0];
    assert_eq!(block.math.len(), 1);
    assert_eq!(
        math.content
            .iter()
            .map(|r| &block.text[r.clone()])
            .collect::<String>(),
        "α\nβ"
    );
    assert!(math.content.len() >= 2);
    assert!(math.content.windows(2).any(|r| r[0].end < r[1].start));
    for (code, marker, subreason) in [
        ("W0406", "@returns", "returns"),
        ("W0405", "@see", "reference"),
        ("W0409", "$", "inline"),
    ] {
        let warning = parsed
            .warnings
            .iter()
            .find(|w| w.code().to_string() == code)
            .unwrap();
        let logical = if marker == "$" {
            block.text.rfind('$').unwrap()
        } else {
            block.text.find(marker).unwrap()
        };
        let loaded = if marker == "$" {
            source.text.rfind('$').unwrap()
        } else {
            source.text.find(marker).unwrap()
        };
        assert!(
            matches!(warning.primary_location(), DiagnosticPrimaryLocation::Span(span)
            if span.range() == mizar_session::SourceRange { source_id: source.source_id, start: loaded, end: loaded + marker.len() })
        );
        assert!(warning.secondary_spans().is_empty());
        let details = warning.details().entries();
        assert_eq!(
            details["documentation.subreason"],
            V::String(subreason.into())
        );
        let V::List(occurrence) = &details["documentation.occurrence"] else {
            panic!("occurrence")
        };
        assert_eq!(
            occurrence[5],
            V::List(vec![
                V::String(logical.to_string()),
                V::String((logical + marker.len()).to_string())
            ])
        );
        assert_eq!(
            warning.stable_detail_key(),
            match code {
                "W0406" => "documentation.tag.missing_body",
                "W0405" => "documentation.tag.missing_slot",
                _ => "documentation.markdown.unmatched_math_delimiter",
            }
        );
    }
    assert_eq!(parsed.warnings.len(), 3);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc8_actual_map_failure_retains_conditions_and_other_blocks() {
    use mizar_artifact::store::CanonicalJson as J;
    use mizar_diagnostics::failure_record::{
        DiagnosticDetailValue as V, DiagnosticPrimaryLocation,
    };
    use mizar_doc::comments::attach_comments;
    let text = "::: @deprecated\n:::\n::: @returns\n:::\n::: $orphan\n\n::: @returns\n\n::: Selected & text\n";
    let (root, mut plan, source) = doc7_fixture("doc8_map_failure", text, None, |_, _, _| {});
    plan.profile.fail_on_doc_warnings = true;
    let module = plan
        .index
        .modules
        .keys()
        .find(|m| m.module_path == "module_0")
        .unwrap();
    // Corrupt only a derived row's map, after the genuine Doc7 boundary. Its
    // raw spelling still parses; the independent block map remains usable.
    let mut faulty_row = {
        let attachments = attach_comments(&plan, module, Some(&source)).unwrap();
        attachments.blocks[0].lines[0].clone()
    };
    let J::Array(fields) = &mut faulty_row else {
        panic!("documentation row")
    };
    let J::String(raw) = &fields[2] else {
        panic!("raw documentation spelling")
    };
    fields[1] = doc7_range(1, 1 + raw.len());
    let mut attachments = attach_comments(&plan, module, Some(&source)).unwrap();
    assert!(attachments.warnings.is_empty());
    assert_eq!(attachments.blocks.len(), 3);
    let saved_projection = attachments.input.comment_projection.clone();
    let block_range = attachments.blocks[0].source_range.unwrap();
    attachments.blocks[0].lines[0] = &faulty_row;
    let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
    assert_eq!(doc8_codes(&parsed.warnings), ["W0411", "W0406", "W0407"]);
    assert!(parsed.blocks[0].fallback);
    assert_eq!(
        parsed.blocks[0].text,
        "@deprecated\n\n@returns\n\n$orphan\n"
    );
    assert!(parsed.blocks[0].fields.is_empty());
    assert!(parsed.blocks[0].math.is_empty());
    assert!(!parsed.blocks[1].fallback);
    assert_eq!(parsed.blocks[1].fields.len(), 1);
    let duplicate = &parsed.warnings[2];
    assert_eq!(duplicate.stable_detail_key(), "documentation.tag.duplicate");
    let V::List(occurrence) = &duplicate.details().entries()["documentation.occurrence"] else {
        panic!("duplicate occurrence")
    };
    assert_eq!(occurrence[3], V::String("1".into()));
    let marker_start = source.text.rfind("@returns").unwrap();
    assert!(
        matches!(duplicate.primary_location(), DiagnosticPrimaryLocation::Span(span)
        if span.range() == mizar_session::SourceRange { source_id: source.source_id, start: marker_start, end: marker_start + "@returns".len() })
    );
    assert_eq!(
        parsed.attachments.input.comment_projection,
        saved_projection
    );
    let warning = &parsed.warnings[0];
    assert!(
        matches!(warning.primary_location(), DiagnosticPrimaryLocation::Span(span)
        if span.range() == block_range)
    );
    let details = warning.details().entries();
    assert_eq!(
        details["documentation.subreason"],
        V::String("mapping_failed".into())
    );
    assert_eq!(
        details["documentation.conversion_stage"],
        V::String("source_map".into())
    );
    assert_eq!(details["documentation.raw"], V::List(vec![]));
    let V::List(retained) = &details["documentation.retained_conditions"] else {
        panic!("retained conditions")
    };
    assert_eq!(retained.len(), 4);
    for (condition, reason, subreason, offsets) in [
        (
            &retained[0],
            "documentation.tag.missing_body",
            "deprecated",
            ["0", "11"],
        ),
        (
            &retained[1],
            "documentation.tag.missing_slot",
            "version",
            ["0", "11"],
        ),
        (
            &retained[2],
            "documentation.tag.missing_body",
            "returns",
            ["13", "21"],
        ),
        (
            &retained[3],
            "documentation.markdown.unmatched_math_delimiter",
            "inline",
            ["23", "24"],
        ),
    ] {
        let V::List(tuple) = condition else {
            panic!("condition tuple")
        };
        let V::List(occurrence) = &tuple[0] else {
            panic!("condition occurrence")
        };
        assert_eq!(occurrence[4], V::String(reason.into()));
        assert_eq!(
            occurrence[5],
            V::List(offsets.into_iter().map(|n| V::String(n.into())).collect())
        );
        assert_eq!(tuple[2], V::String(subreason.into()));
    }
    let retained = format!("{parsed:?}");
    assert_eq!(
        mizar_doc::render::summary_text(&parsed, mizar_doc::comments::CommentTarget::Module)
            .unwrap(),
        Some("Selected & text".into())
    );
    assert!(parsed.documentation_failed());
    assert_eq!(retained, format!("{parsed:?}"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc13_g4_real_doc8_transport_and_browser_witnesses() {
    use mizar_doc::{markdown::parse_comments, render::opaque_math_fragments};

    let cases = [
        ("ordered", "$x$ $x$\n\n$$y$$\n\n> $$\n> α\n> β\n> $$".to_owned(), vec!["rendered"; 4]),
        ("invalid", "$\\frac{$ $\\mizarUnknown$ $z$".into(), vec!["error", "error", "rendered"]),
        ("hostile", "$$\n\\mizarUnknown </script><script>globalThis.payloadRan=1</script><img src=x onerror=\"globalThis.payloadRan=2\"> &lt; &#0; \" ' λ\u{202e}\t\n$$".into(), vec!["error"]),
        ("trust", "$\\href{https://remote.invalid/}{x}$ $\\htmlClass{injected}{x}$ $\\includegraphics{https://remote.invalid/x}$ $z$".into(), vec!["safe", "safe", "safe", "rendered"]),
        ("macros", "$\\gdef\\mizarLeak{7}\\mizarLeak$ $\\mizarLeak$ $z$".into(), vec!["rendered", "error", "rendered"]),
        ("expansion", "$\\def\\a{\\a}\\a$ $z$".into(), vec!["error", "rendered"]),
        ("size", "$\\rule{100em}{100em}$".into(), vec!["rendered"]),
        ("limit", format!("${}{}$", "\\relax", " ".repeat(65530)), vec!["rendered"]),
        ("oversize", format!("${}$ $z$", "λ".repeat(32769)), vec!["error", "rendered"]),
        ("empty", "no math; $$ $$".into(), vec![]),
    ];
    for (name, markdown, expected_states) in cases {
        let (root, plan, _) = doc8_fixture(&markdown);
        let attachments = doc7_attach(&plan);
        let parsed = parse_comments(&attachments, &[]);
        let before = format!("{:?}", parsed);
        let fragments = opaque_math_fragments(&parsed).unwrap();
        assert_eq!(fragments, opaque_math_fragments(&parsed).unwrap());
        assert_eq!(before, format!("{:?}", parsed));
        assert!(std::ptr::eq(parsed.attachments, &attachments));
        assert!(std::ptr::eq(parsed.attachments.plan, &plan));
        assert_eq!(fragments.len(), expected_states.len(), "{name}");
        let mut ordinal = 0;
        for (block_index, block) in parsed.blocks.iter().enumerate() {
            for (occurrence, math) in block.math.iter().enumerate() {
                let payload: String = math
                    .content
                    .iter()
                    .map(|r| &block.text[r.clone()])
                    .collect();
                let hex: String = payload
                    .as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect();
                let tag = if math.display { "div" } else { "span" };
                assert!(fragments[ordinal].starts_with(&format!("<{tag} class=\"mizar-math\" data-mizar-block=\"{block_index}\" data-mizar-occurrence=\"{occurrence}\" data-mizar-display=\"{}\" data-mizar-tex=\"{hex}\"", math.display)), "{name}");
                assert!(!fragments[ordinal].contains("<script>"));
                if name == "ordered" && ordinal == 3 {
                    assert_eq!(payload, "α\nβ\n");
                    assert!(math.content.windows(2).any(|r| r[0].end < r[1].start));
                    assert!(block.source_map.len() > 2);
                }
                if let Some(export) = std::env::var_os("MIZAR_DOC_MATH_WITNESS_DIR") {
                    let dir = PathBuf::from(export).join(name);
                    std::fs::create_dir_all(&dir).unwrap();
                    std::fs::write(dir.join(format!("{ordinal}.tex")), &payload).unwrap();
                    std::fs::write(
                        dir.join(format!("{ordinal}.state")),
                        expected_states[ordinal],
                    )
                    .unwrap();
                    std::fs::write(dir.join("fragments.html"), fragments.join("\n")).unwrap();
                }
                ordinal += 1;
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    // Doc8 rejects empty delimiters and normalizes source newlines. These
    // explicitly classified boundary inputs test transport without claiming
    // that those spellings originated in the parser.
    let (root, plan, _) = doc8_fixture("$x$");
    let attachments = doc7_attach(&plan);
    let mut parsed = parse_comments(&attachments, &[]);
    for (name, payload) in [
        ("controls", "\u{feff}\0\r\n\u{1}\u{7f}λ&<>\"'\\mizarUnknown"),
        ("empty_payload", ""),
    ] {
        parsed.blocks[0].text = format!("${payload}$");
        parsed.blocks[0].math[0].range = 0..payload.len() + 2;
        parsed.blocks[0].math[0].content = std::iter::once(1..payload.len() + 1).collect();
        let fragments = opaque_math_fragments(&parsed).unwrap();
        let hex: String = payload
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(fragments[0].contains(&format!("data-mizar-tex=\"{hex}\"")));
        assert!(!fragments[0].contains('\0'));
        if let Some(export) = std::env::var_os("MIZAR_DOC_MATH_WITNESS_DIR") {
            let dir = PathBuf::from(export).join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("0.tex"), payload).unwrap();
            std::fs::write(
                dir.join("0.state"),
                if payload.is_empty() {
                    "rendered"
                } else {
                    "error"
                },
            )
            .unwrap();
            std::fs::write(dir.join("fragments.html"), fragments.join("\n")).unwrap();
        }
    }
    let length = parsed.blocks[0].text.len();
    parsed.blocks[0].math[0].content = std::iter::once(0..length + 1).collect();
    assert!(opaque_math_fragments(&parsed).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc13_g4_fragments_leave_full_targets_publication_and_segment_maps_borrowed() {
    use mizar_doc::comments::{CommentTarget, attach_comments};
    use mizar_doc::{markdown::parse_comments, render::opaque_math_fragments};

    let original = "::: $m$\r\nimport X;\r\n::: > $α\r\n::: > β$\r\ntheorem T: true;\r\n";
    let (root, plan, source) = doc7_fixture(
        "doc13_targets",
        original,
        Some(("theorem", "TheoremItem", "theorem T: true;")),
        |_, _, landmarks| {
            let start = original.replace("\r\n", "\n").find("import").unwrap();
            landmarks.insert(0, doc7_landmark("ImportItem", start, start + 9, None, None));
        },
    );
    let module = plan
        .index
        .modules
        .keys()
        .find(|m| m.module_path == "module_0")
        .unwrap();
    let attachments = attach_comments(&plan, module, Some(&source)).unwrap();
    let parsed = parse_comments(&attachments, &[]);
    assert_eq!(parsed.blocks.len(), 2);
    assert_eq!(attachments.blocks[0].target, Some(CommentTarget::Module));
    let Some(CommentTarget::Item(export)) = attachments.blocks[1].target else {
        panic!("item target")
    };
    assert!(std::ptr::eq(export, &attachments.input.exports[0]));
    let publication = attachments.plan.manifest.artifact_hash;
    let identity = attachments.module.clone();
    let snapshot = format!("{parsed:?}");
    let fragments = opaque_math_fragments(&parsed).unwrap();
    assert_eq!(fragments.len(), 2);
    assert!(fragments[0].contains("data-mizar-block=\"0\" data-mizar-occurrence=\"0\""));
    assert!(fragments[1].contains("data-mizar-block=\"1\" data-mizar-occurrence=\"0\""));
    let math = &parsed.blocks[1].math[0];
    assert!(math.content.windows(2).any(|r| r[0].end < r[1].start));
    for (block, attachment) in parsed.blocks.iter().zip(&attachments.blocks) {
        assert_eq!(attachment.source_range.unwrap().source_id, source.source_id);
        for (logical, loaded) in &block.source_map {
            assert_eq!(
                &block.text[logical.clone()],
                &source.text[loaded.start..loaded.end]
            );
        }
    }
    assert_eq!(snapshot, format!("{parsed:?}"));
    assert_eq!(parsed.attachments.plan.manifest.artifact_hash, publication);
    assert_eq!(*parsed.attachments.module, identity);
    std::fs::remove_dir_all(root).unwrap();
}

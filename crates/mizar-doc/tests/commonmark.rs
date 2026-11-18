//! CommonMark 0.31.2 examples, John MacFarlane, CC BY-SA 4.0.
//! Corpus: https://spec.commonmark.org/0.31.2/spec.json (retained unchanged).
//! License: https://creativecommons.org/licenses/by-sa/4.0/

use pulldown_cmark::{Options, Parser, html};

// Compare parser conformance independently of two HTML serializer spellings:
// quoted character data and the optional newline immediately inside a list item.
// CommonMark's official normalizer also discards the latter block whitespace.
// Preserve attributes, comments, raw-text elements and all other bytes.
fn comparable_html(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;
    let mut after_item = false;
    let mut in_pre = false;
    while !rest.is_empty() {
        if rest.starts_with("<!--") {
            let end = rest.find("-->").map_or(rest.len(), |end| end + 3);
            output.push_str(&rest[..end]);
            rest = &rest[end..];
            after_item = false;
        } else if rest.starts_with('<') {
            let mut quote = None;
            let mut end = rest.len();
            for (offset, ch) in rest.char_indices().skip(1) {
                if let Some(delimiter) = quote {
                    if ch == delimiter {
                        quote = None;
                    }
                } else if matches!(ch, '\'' | '"') {
                    quote = Some(ch);
                } else if ch == '>' {
                    end = offset + 1;
                    break;
                }
            }
            let tag = &rest[..end];
            output.push_str(tag);
            rest = &rest[end..];
            let lower = tag.to_ascii_lowercase();
            if lower == "<pre>" {
                in_pre = true;
            }
            if lower == "</pre>" {
                in_pre = false;
            }
            after_item = tag == "<li>" && !in_pre;
            let name = lower
                .trim_start_matches('<')
                .split([' ', '\t', '\n', '\r', '>'])
                .next()
                .unwrap();
            if matches!(
                name,
                "script" | "style" | "xmp" | "iframe" | "noembed" | "noframes" | "plaintext"
            ) {
                let closing = format!("</{name}");
                let end = rest
                    .to_ascii_lowercase()
                    .find(&closing)
                    .unwrap_or(rest.len());
                output.push_str(&rest[..end]);
                rest = &rest[end..];
                after_item = false;
            }
        } else {
            let end = rest.find('<').unwrap_or(rest.len());
            let text = &rest[..end];
            let text = if after_item {
                text.trim_start_matches('\n')
            } else {
                text
            };
            output.push_str(&text.replace("&quot;", "\""));
            rest = &rest[end..];
            after_item = false;
        }
    }
    output
}

#[test]
fn pinned_commonmark_examples() {
    let examples: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/commonmark-0.31.2.json"
    ))
    .unwrap();
    let examples = examples.as_array().unwrap();
    assert_eq!(examples.len(), 652);
    for example in examples {
        let mut actual = String::new();
        html::push_html(
            &mut actual,
            Parser::new_ext(example["markdown"].as_str().unwrap(), Options::empty()),
        );
        assert_eq!(
            comparable_html(&actual),
            comparable_html(example["html"].as_str().unwrap()),
            "CommonMark example {} ({})",
            example["example"],
            example["section"]
        );
    }
}

#[test]
fn conformance_comparator_preserves_content_and_structure() {
    for (left, right) in [
        ("<p>\"</p>", "<p>&quot;</p>"),
        (
            "<pre><code>\"  \n</code></pre>",
            "<pre><code>&quot;  \n</code></pre>",
        ),
        ("<li>\n<div>\n", "<li><div>\n"),
    ] {
        assert_eq!(comparable_html(left), comparable_html(right));
    }
    for (left, right) in [
        ("<p>a</p>", "<p>b</p>"),
        ("<em>a</em>", "<strong>a</strong>"),
        ("<a title='&quot;'>a</a>", "<a title='\"'>a</a>"),
        ("<script>\"</script>", "<script>&quot;</script>"),
        ("<style>\"</style>", "<style>&quot;</style>"),
        ("<!-- \" -->", "<!-- &quot; -->"),
        (
            "<pre><code>a \n</code></pre>",
            "<pre><code>a\n</code></pre>",
        ),
        ("<pre><li>\nx</li></pre>", "<pre><li>x</li></pre>"),
    ] {
        assert_ne!(comparable_html(left), comparable_html(right));
    }
}

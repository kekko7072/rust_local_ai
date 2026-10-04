use serde_json::Value;

use super::GenUiModuleSpec;

/// Upper bound on JSON object candidates examined per output, keeping parsing
/// linear-ish on adversarial input.
const MAX_CANDIDATES: usize = 32;

/// Parses a model's raw text output into a validated [`GenUiModuleSpec`].
///
/// Extracts a top-level JSON object, tolerating code fences, leading or
/// trailing prose and trailing commas. Output truncated by the token budget is
/// repaired by cutting at the last completed nested structure and closing the
/// brackets still open there, so a partially generated module still renders.
/// If the first object is not a valid module (stray braces in prose, or the
/// module wrapped in another object), later and nested objects are tried.
/// Returns `None` when no valid module can be recovered. Never panics.
pub fn parse_model_output(text: &str) -> Option<GenUiModuleSpec> {
    let text = strip_code_fences(text.trim());
    text.match_indices('{')
        .take(MAX_CANDIDATES)
        .filter_map(|(start, _)| extract_json_object(&text[start..]))
        .find_map(|value| GenUiModuleSpec::from_json(&value))
}

/// Extracts the first JSON object in `s`, repairing truncation.
fn extract_json_object(s: &str) -> Option<Value> {
    let start = s.find('{')?;

    let mut close_stack: Vec<char> = Vec::new();
    // Byte index just past the last completed nested structure, and the stack
    // depth there. Every pop records a new cut, so only pushes happen after
    // the latest one and `close_stack[..depth]` is still the closers that were
    // open at the cut; recording the depth keeps the scan linear.
    let mut safe_cut: Option<(usize, usize)> = None;
    let mut in_string = false;
    let mut escape = false;

    for (i, c) in s[start..].char_indices().map(|(i, c)| (start + i, c)) {
        if in_string {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' => close_stack.push('}'),
            '[' => close_stack.push(']'),
            '}' | ']' => {
                close_stack.pop();
                if close_stack.is_empty() {
                    return decode_object(&s[start..=i]);
                }
                safe_cut = Some((i + 1, close_stack.len()));
            }
            _ => {}
        }
    }

    // Unbalanced at end of input: most likely cut off by the output budget.
    let (end, depth) = safe_cut?;
    let mut repaired = s[start..end].to_owned();
    repaired.extend(close_stack[..depth].iter().rev());
    decode_object(&repaired)
}

/// Removes ```` ``` ```` and ```` ```lang ```` fence markers.
fn strip_code_fences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find("```") {
        out.push_str(&rest[..pos]);
        rest = rest[pos + 3..].trim_start_matches(|c: char| c.is_ascii_alphabetic());
    }
    out.push_str(rest);
    out
}

/// Decodes `candidate` as a JSON object, retrying once without trailing commas
/// (a common small-model mistake).
fn decode_object(candidate: &str) -> Option<Value> {
    serde_json::from_str::<Value>(candidate)
        .or_else(|_| serde_json::from_str(&strip_trailing_commas(candidate)))
        .ok()
        .filter(Value::is_object)
}

fn strip_trailing_commas(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escape = false;
    for (i, c) in text.char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if c == ',' && text[i + 1..].trim_start().starts_with(['}', ']']) {
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn extracts_object_from_fences_and_prose() {
        let text = "Sure! Here it is:\n```json\n{\"title\":\"A\",\"blocks\":[{\"type\":\"note\",\"text\":\"}{\"}]}\n```\nEnjoy.";
        let spec = parse_model_output(text).unwrap();
        assert_eq!(spec.title, "A");
        assert_eq!(spec.blocks[0].get("text").unwrap(), "}{");
    }

    #[test]
    fn tolerates_trailing_commas_outside_strings() {
        let value =
            extract_json_object(r#"{"title":"a, ]","blocks":[{"type":"note","text":"x",},],}"#)
                .unwrap();
        assert_eq!(
            value,
            json!({"title": "a, ]", "blocks": [{"type": "note", "text": "x"}]})
        );
    }

    #[test]
    fn repairs_truncated_output() {
        let text = r#"{"title":"Trip","blocks":[{"type":"amount","label":"Saved","value":10},{"type":"checklist","label":"Ste"#;
        let spec = parse_model_output(text).unwrap();
        assert_eq!(spec.title, "Trip");
        assert_eq!(spec.blocks.len(), 1);
        assert_eq!(spec.blocks[0].block_type(), "amount");
    }

    #[test]
    fn rejects_text_without_a_module() {
        assert!(parse_model_output("no json here").is_none());
        assert!(parse_model_output(r#"{"title":"#).is_none());
        assert!(parse_model_output(r#"{"title":"x","blocks":[{"type":"bogus"}]}"#).is_none());
    }

    #[test]
    fn skips_invalid_candidates_and_unwraps_nested_modules() {
        let stray = r#"Fill in {placeholder} first. {"title":"Real","blocks":[{"type":"note","text":"ok"}]}"#;
        assert_eq!(parse_model_output(stray).unwrap().title, "Real");
        let wrapped = r#"{"module":{"title":"Inner","blocks":[{"type":"note","text":"ok"}]}}"#;
        assert_eq!(parse_model_output(wrapped).unwrap().title, "Inner");
    }

    const MODULE: &str = r#"```json
{"title":"Trip — fund ✈️","icon":"piggy-bank","tone":"sky","blurb":"Save weekly, \"slowly\".",
 "blocks":[{"type":"amount","label":"Saved","value":120,"prefix":"$"},
           {"type":"checklist","label":"Steps","items":[{"label":"Open {account}","done":false},{"label":"Automate","done":false}]},
           {"type":"note","text":"Brackets ] } and commas , inside strings"}]}
```"#;

    #[test]
    fn every_truncation_point_is_handled_without_panicking() {
        assert_eq!(parse_model_output(MODULE).unwrap().blocks.len(), 3);
        let mut recovered = 0;
        for (end, _) in MODULE.char_indices() {
            if let Some(spec) = parse_model_output(&MODULE[..end]) {
                assert!(!spec.blocks.is_empty());
                assert_eq!(spec.title, "Trip — fund ✈️");
                recovered += 1;
            }
        }
        // Once the first block is complete, partial output keeps rendering.
        assert!(recovered > MODULE.len() / 3, "recovered only {recovered}");
    }

    #[test]
    fn arbitrary_input_never_panics() {
        // Deterministic xorshift so the test needs no dependencies.
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let alphabet: Vec<char> = r#"{}[]",:\ `abc0123456789.-$é✈ tn"#.chars().collect();
        for _ in 0..2_000 {
            let mut input = String::new();
            for _ in 0..(state % 120) {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                input.push(alphabet[(state % alphabet.len() as u64) as usize]);
            }
            let _ = parse_model_output(&input);
            let _ = parse_model_output(&format!("{MODULE}{input}"));
            let _ = parse_model_output(&format!("{input}{MODULE}"));
        }
    }

    #[test]
    fn deeply_nested_input_is_rejected_not_overflowed() {
        let deep = format!(r#"{{"title":"x","blocks":{}"#, "[".repeat(100_000));
        assert!(parse_model_output(&deep).is_none());
        let deep_closed = format!("{}{}", "{\"a\":".repeat(50_000), "}".repeat(50_000));
        assert!(parse_model_output(&deep_closed).is_none());
    }
}

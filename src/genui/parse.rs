use serde_json::Value;

use super::GenUiModuleSpec;

/// Parses a model's raw text output into a validated [`GenUiModuleSpec`].
///
/// Extracts the first top-level JSON object, tolerating code fences, leading or
/// trailing prose and trailing commas. Output truncated by the token budget is
/// repaired by cutting at the last completed nested structure and closing the
/// brackets still open there, so a partially generated module still renders.
/// Returns `None` when no valid module can be recovered.
pub fn parse_model_output(text: &str) -> Option<GenUiModuleSpec> {
    GenUiModuleSpec::from_json(&extract_json_object(text)?)
}

pub(crate) fn extract_json_object(text: &str) -> Option<Value> {
    let s = strip_code_fences(text.trim());
    let start = s.find('{')?;

    let mut close_stack: Vec<char> = Vec::new();
    // Byte index just past the last completed nested structure, together with
    // the closers still open at that point.
    let mut safe_cut: Option<(usize, Vec<char>)> = None;
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
                safe_cut = Some((i + 1, close_stack.clone()));
            }
            _ => {}
        }
    }

    // Unbalanced at end of input: most likely cut off by the output budget.
    let (end, open) = safe_cut?;
    let mut repaired = s[start..end].to_owned();
    repaired.extend(open.iter().rev());
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
}

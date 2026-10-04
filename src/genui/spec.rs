use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Block types a module may contain. Blocks of any other type are dropped
/// during validation.
pub const GENUI_BLOCK_TYPES: &[&str] = &[
    "note",
    "field",
    "amount",
    "progress",
    "checklist",
    "week",
    "stat",
    "list",
    "lessons",
    "reminder",
    "calc",
    "docs",
];

const MAX_TITLE_CHARS: usize = 36;

/// Colour family of a module.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenUiTone {
    #[default]
    Fern,
    Apricot,
    Sky,
    Lilac,
}

impl GenUiTone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fern => "fern",
            Self::Apricot => "apricot",
            Self::Sky => "sky",
            Self::Lilac => "lilac",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "fern" => Some(Self::Fern),
            "apricot" => Some(Self::Apricot),
            "sky" => Some(Self::Sky),
            "lilac" => Some(Self::Lilac),
            _ => None,
        }
    }
}

/// One typed block of a module, kept as the JSON object the model produced.
///
/// The `type` field is guaranteed to be one of [`GENUI_BLOCK_TYPES`]; the
/// remaining fields are type-specific and passed through untouched so a
/// renderer sees exactly what the model wrote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GenUiBlock(Map<String, Value>);

impl GenUiBlock {
    /// Validates a raw block, returning `None` for non-objects and unknown
    /// block types.
    pub fn from_json(value: Value) -> Option<Self> {
        let Value::Object(map) = value else {
            return None;
        };
        let block_type = map.get("type").and_then(Value::as_str)?;
        GENUI_BLOCK_TYPES.contains(&block_type).then_some(Self(map))
    }

    /// The block's `type`, e.g. `"progress"`.
    pub fn block_type(&self) -> &str {
        self.0
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn fields(&self) -> &Map<String, Value> {
        &self.0
    }

    pub fn into_fields(self) -> Map<String, Value> {
        self.0
    }

    /// A one-line, human-readable summary of the block.
    pub fn describe(&self) -> String {
        let block_type = self.block_type();
        let label = ["label", "title", "text"]
            .iter()
            .find_map(|key| self.0.get(*key))
            .map(display)
            .unwrap_or_else(|| block_type.to_owned());
        let prefix = self.0.get("prefix").map(display).unwrap_or_default();
        let field = |key: &str, default: &str| {
            self.0
                .get(key)
                .map(display)
                .unwrap_or_else(|| default.to_owned())
        };
        let item_count = || {
            self.0
                .get("items")
                .and_then(Value::as_array)
                .map_or(0, Vec::len)
        };
        match block_type {
            "amount" => format!("{label}: {prefix}{}", field("value", "")),
            "progress" => format!(
                "{label}: {prefix}{} of {prefix}{}",
                field("value", "0"),
                field("target", "0")
            ),
            "reminder" => format!("Reminder · {label}"),
            "calc" => format!("Calculator · {label}"),
            "checklist" => format!("Checklist · {} steps", item_count()),
            "docs" => format!("Documents · {} tracked", item_count()),
            _ => label,
        }
    }
}

/// Renders a JSON scalar the way it reads to a user (strings unquoted).
fn display(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// A validated genUI module: a titled stack of typed blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenUiModuleSpec {
    pub title: String,
    /// A [lucide](https://lucide.dev) icon name.
    pub icon: String,
    pub tone: GenUiTone,
    pub blurb: String,
    pub blocks: Vec<GenUiBlock>,
}

impl GenUiModuleSpec {
    /// Validates the JSON object a model produced.
    ///
    /// Returns `None` unless it has a non-empty title and at least one block of
    /// a known type. Unknown blocks are dropped, an unknown tone falls back to
    /// [`GenUiTone::Fern`], and missing icon/blurb get defaults.
    pub fn from_json(value: &Value) -> Option<Self> {
        let object = value.as_object()?;
        let title = object.get("title").map(display)?.trim().to_owned();
        if title.is_empty() {
            return None;
        }
        let blocks: Vec<_> = object
            .get("blocks")?
            .as_array()?
            .iter()
            .cloned()
            .filter_map(GenUiBlock::from_json)
            .collect();
        if blocks.is_empty() {
            return None;
        }
        let tone = object
            .get("tone")
            .and_then(Value::as_str)
            .and_then(GenUiTone::parse)
            .unwrap_or_default();
        Some(Self {
            title: truncate_title(title),
            icon: object
                .get("icon")
                .map(display)
                .unwrap_or_else(|| "sparkles".into()),
            tone,
            blurb: object
                .get("blurb")
                .map(display)
                .unwrap_or_else(|| "A plan to get this done.".into()),
            blocks,
        })
    }

    /// The module shape consumed by flutter_local_ai host apps
    /// (`FledgeModule.fromJson`), with a fresh time-based `id`.
    pub fn to_module_json(&self) -> Value {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis());
        json!({
            "id": format!("gen-{millis}"),
            "title": self.title,
            "icon": self.icon,
            "tone": self.tone,
            "kind": "composed",
            "generated": true,
            "aiGenerated": true,
            "blurb": self.blurb,
            "blocks": self.blocks,
        })
    }

    /// A renderer-neutral component tree summarising the module: a `root`
    /// `Column` whose children are a header `Text` and one `Text` per block.
    ///
    /// Each entry has `id`, `type` and `properties`, matching the maps
    /// flutter_local_ai returns for Flutter's `genui` `Component`.
    pub fn to_component_maps(&self) -> Vec<Value> {
        self.summary_components()
            .into_iter()
            .map(|SummaryComponent { id, kind }| {
                let type_name = kind.type_name();
                let mut properties = Map::new();
                match kind {
                    SummaryKind::Column(children) => {
                        properties.insert("children".into(), json!(children));
                    }
                    SummaryKind::Text { text, variant } => {
                        properties.insert("text".into(), text.into());
                        if let Some(variant) = variant {
                            properties.insert("variant".into(), variant.into());
                        }
                    }
                }
                json!({ "id": id, "type": type_name, "properties": properties })
            })
            .collect()
    }

    pub(crate) fn summary_components(&self) -> Vec<SummaryComponent> {
        let block_ids: Vec<String> = (0..self.blocks.len())
            .map(|i| format!("gen_block_{i}"))
            .collect();
        let mut children = vec!["gen_header".to_owned()];
        children.extend(block_ids.iter().cloned());

        let mut components = vec![
            SummaryComponent {
                id: "root".into(),
                kind: SummaryKind::Column(children),
            },
            SummaryComponent {
                id: "gen_header".into(),
                kind: SummaryKind::Text {
                    text: self.title.clone(),
                    variant: Some("h4"),
                },
            },
        ];
        components.extend(block_ids.into_iter().zip(&self.blocks).map(|(id, block)| {
            SummaryComponent {
                id,
                kind: SummaryKind::Text {
                    text: block.describe(),
                    variant: None,
                },
            }
        }));
        components
    }
}

pub(crate) struct SummaryComponent {
    pub(crate) id: String,
    pub(crate) kind: SummaryKind,
}

pub(crate) enum SummaryKind {
    Column(Vec<String>),
    Text {
        text: String,
        variant: Option<&'static str>,
    },
}

impl SummaryKind {
    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Self::Column(_) => "Column",
            Self::Text { .. } => "Text",
        }
    }
}

fn truncate_title(title: String) -> String {
    if title.chars().count() > MAX_TITLE_CHARS {
        let mut short: String = title.chars().take(MAX_TITLE_CHARS - 2).collect();
        short.push('…');
        short
    } else {
        title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_drops_unknown_blocks_and_defaults_fields() {
        let spec = GenUiModuleSpec::from_json(&json!({
            "title": "  Trip fund  ",
            "tone": "neon",
            "blocks": [
                {"type": "amount", "label": "Saved", "value": 120, "prefix": "$"},
                {"type": "hologram"},
                "not a block",
            ],
        }))
        .unwrap();
        assert_eq!(spec.title, "Trip fund");
        assert_eq!(spec.tone, GenUiTone::Fern);
        assert_eq!(spec.icon, "sparkles");
        assert_eq!(spec.blurb, "A plan to get this done.");
        assert_eq!(spec.blocks.len(), 1);
        assert_eq!(spec.blocks[0].describe(), "Saved: $120");
    }

    #[test]
    fn rejects_modules_without_title_or_valid_blocks() {
        assert!(GenUiModuleSpec::from_json(&json!({"blocks": [{"type": "note"}]})).is_none());
        assert!(GenUiModuleSpec::from_json(&json!({"title": "x", "blocks": []})).is_none());
        assert!(GenUiModuleSpec::from_json(&json!({"title": "x"})).is_none());
        assert!(GenUiModuleSpec::from_json(&json!([1, 2])).is_none());
    }

    #[test]
    fn long_titles_are_truncated_on_char_boundaries() {
        let spec = GenUiModuleSpec::from_json(&json!({
            "title": "é".repeat(40),
            "blocks": [{"type": "note", "text": "hi"}],
        }))
        .unwrap();
        assert_eq!(spec.title, format!("{}…", "é".repeat(34)));
    }

    #[test]
    fn module_json_and_component_maps_match_flutter_shapes() {
        let spec = GenUiModuleSpec::from_json(&json!({
            "title": "Plan",
            "icon": "list",
            "tone": "sky",
            "blurb": "Do it",
            "blocks": [
                {"type": "progress", "label": "Fund", "value": 3, "target": 6, "prefix": "$"},
                {"type": "checklist", "label": "Steps", "items": [{"label": "a"}, {"label": "b"}]},
            ],
        }))
        .unwrap();
        let module = spec.to_module_json();
        assert_eq!(module["tone"], "sky");
        assert_eq!(module["kind"], "composed");
        assert!(module["id"].as_str().unwrap().starts_with("gen-"));
        assert_eq!(module["blocks"][0]["target"], 6);

        let components = spec.to_component_maps();
        assert_eq!(
            components[0],
            json!({"id": "root", "type": "Column", "properties": {
                "children": ["gen_header", "gen_block_0", "gen_block_1"]
            }})
        );
        assert_eq!(components[1]["properties"]["variant"], "h4");
        assert_eq!(components[2]["properties"]["text"], "Fund: $3 of $6");
        assert_eq!(components[3]["properties"]["text"], "Checklist · 2 steps");
    }
}

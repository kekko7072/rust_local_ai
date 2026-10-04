use a2ui_types::{
    common::SurfaceId,
    v09::server_to_client::{
        CreateSurface, ServerToClientMessage, UpdateComponents, UpdateDataModel,
    },
};
use serde_json::{json, Map, Value};

use super::{
    spec::{SummaryComponent, SummaryKind},
    GenUiModuleSpec,
};

/// Catalog ID of the A2UI v0.9 basic component catalog, which provides the
/// `Column` and `Text` components used by
/// [`GenUiModuleSpec::to_a2ui_v09_messages`].
pub const A2UI_V09_BASIC_CATALOG_ID: &str =
    "https://a2ui.org/specification/v0_9/basic_catalog.json";

const A2UI_V09: &str = "v0.9";

impl GenUiModuleSpec {
    /// The module as an A2UI v0.9 server-to-client message stream, typed with
    /// the [`a2ui-types`](https://crates.io/crates/a2ui-types) crate.
    ///
    /// Produces, in order:
    /// 1. `createSurface` for `surface_id` using `catalog_id` (usually
    ///    [`A2UI_V09_BASIC_CATALOG_ID`]);
    /// 2. `updateComponents` with the same tree as
    ///    [`to_component_maps`](Self::to_component_maps), in A2UI's flat
    ///    `{"id", "component", ...properties}` form;
    /// 3. `updateDataModel` setting the root of the data model to
    ///    [`to_module_json`](Self::to_module_json), so custom catalog
    ///    components can bind to the full typed blocks.
    ///
    /// Serialize each message as one JSON line (JSONL) to send it to any
    /// A2UI renderer.
    pub fn to_a2ui_v09_messages(
        &self,
        surface_id: impl Into<SurfaceId>,
        catalog_id: impl Into<String>,
    ) -> Vec<ServerToClientMessage> {
        let surface_id = surface_id.into();
        vec![
            message(|m| {
                m.create_surface = Some(CreateSurface {
                    surface_id: surface_id.clone(),
                    catalog_id: catalog_id.into(),
                    theme: None,
                    send_data_model: None,
                });
            }),
            message(|m| {
                m.update_components = Some(UpdateComponents {
                    surface_id: surface_id.clone(),
                    components: self.a2ui_v09_components(),
                });
            }),
            message(|m| {
                m.update_data_model = Some(UpdateDataModel {
                    surface_id,
                    path: Some("/".into()),
                    value: Some(self.to_module_json()),
                });
            }),
        ]
    }

    /// The component list sent in `updateComponents` by
    /// [`to_a2ui_v09_messages`](Self::to_a2ui_v09_messages).
    pub fn a2ui_v09_components(&self) -> Vec<Value> {
        self.summary_components()
            .into_iter()
            .map(|SummaryComponent { id, kind }| {
                let mut component = Map::new();
                component.insert("id".into(), id.into());
                component.insert("component".into(), kind.type_name().into());
                match kind {
                    SummaryKind::Column(children) => {
                        component.insert("children".into(), json!(children));
                    }
                    SummaryKind::Text { text, variant } => {
                        component.insert("text".into(), text.into());
                        if let Some(variant) = variant {
                            component.insert("variant".into(), variant.into());
                        }
                    }
                }
                Value::Object(component)
            })
            .collect()
    }
}

fn message(fill: impl FnOnce(&mut ServerToClientMessage)) -> ServerToClientMessage {
    let mut message = ServerToClientMessage {
        version: A2UI_V09.into(),
        create_surface: None,
        update_components: None,
        update_data_model: None,
        delete_surface: None,
    };
    fill(&mut message);
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_create_update_and_data_messages() {
        let spec = GenUiModuleSpec::from_json(&json!({
            "title": "Trip fund",
            "blocks": [{"type": "amount", "label": "Saved", "value": 10, "prefix": "$"}],
        }))
        .unwrap();
        let messages = spec.to_a2ui_v09_messages("trip", A2UI_V09_BASIC_CATALOG_ID);
        let lines: Vec<Value> = messages
            .iter()
            .map(|m| serde_json::to_value(m).unwrap())
            .collect();

        assert_eq!(
            lines[0],
            json!({"version": "v0.9", "createSurface": {
                "surfaceId": "trip", "catalogId": A2UI_V09_BASIC_CATALOG_ID
            }})
        );
        assert_eq!(
            lines[1]["updateComponents"]["components"],
            json!([
                {"id": "root", "component": "Column", "children": ["gen_header", "gen_block_0"]},
                {"id": "gen_header", "component": "Text", "text": "Trip fund", "variant": "h4"},
                {"id": "gen_block_0", "component": "Text", "text": "Saved: $10"},
            ])
        );
        assert_eq!(lines[2]["updateDataModel"]["path"], "/");
        assert_eq!(
            lines[2]["updateDataModel"]["value"]["blocks"][0]["value"],
            10
        );

        // Round-trips through the strict (deny_unknown_fields) A2UI types.
        for line in lines {
            serde_json::from_value::<ServerToClientMessage>(line).unwrap();
        }
    }
}

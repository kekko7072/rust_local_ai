//! Generative UI: turn a natural-language goal into a small, renderable UI
//! module using the local model.
//!
//! This is a Rust port of the genUI engine shipped in
//! [`flutter_local_ai`](https://pub.dev/packages/flutter_local_ai): the same
//! typed-block schema, the same system instructions and the same tolerant
//! parser, so a module produced on one platform renders identically on the
//! other.
//!
//! The local model decides which typed blocks best express the goal and emits
//! a JSON spec. [`GenUiModuleSpec`] holds the validated result. It can be
//! handed to a host renderer as JSON ([`GenUiModuleSpec::to_module_json`]), as
//! a renderer-neutral component tree ([`GenUiModuleSpec::to_component_maps`]),
//! or, with the `a2ui` feature, as typed A2UI protocol messages
//! (`GenUiModuleSpec::to_a2ui_v09_messages`).
//!
//! ```no_run
//! use rust_local_ai::{detect, genui::{GenUiOptions, LocalAiUiGenerator}};
//!
//! # async fn run() -> rust_local_ai::Result<()> {
//! let generator = LocalAiUiGenerator::new(detect().await?);
//! let module = generator
//!     .generate_module("Save $500 for a weekend trip", &GenUiOptions::default())
//!     .await?;
//! println!("{} ({} blocks)", module.title, module.blocks.len());
//! # Ok(())
//! # }
//! ```
//!
//! Any other model can drive the same generation: send
//! [`GENUI_INSTRUCTIONS`] as its system prompt and pass its raw text to
//! [`parse_model_output`].

#[cfg(feature = "a2ui")]
mod a2ui;
mod generator;
mod parse;
mod spec;

#[cfg(feature = "a2ui")]
pub use a2ui::A2UI_V09_BASIC_CATALOG_ID;
pub use generator::{GenUiOptions, LocalAiUiGenerator, GENUI_MAX_OUTPUT_TOKENS};
pub use parse::parse_model_output;
pub use spec::{GenUiBlock, GenUiModuleSpec, GenUiTone, GENUI_BLOCK_TYPES};

/// The genUI system instructions (the module/block schema).
///
/// Public so other backends can drive exactly the same generation and parse
/// the result with [`parse_model_output`].
pub const GENUI_INSTRUCTIONS: &str = r#"You are Fledge's genUI engine. You design a small mobile "module" that helps a
young adult accomplish a goal. You output ONLY a single JSON object — no prose,
no markdown, no code fences.

The JSON shape is:
{
  "title": string,            // short, sentence case
  "icon": string,             // a lucide icon name, e.g. "piggy-bank"
  "tone": "fern"|"apricot"|"sky"|"lilac",
  "blurb": string,            // one short line describing the module
  "blocks": [ ...blocks ]     // 2-4 blocks, ordered, the right tools for the goal
}

Each block is an object with a "type" and type-specific fields:
- {"type":"amount","label":string,"value":number,"prefix":"$"}
- {"type":"progress","label":string,"value":number,"target":number,"prefix":"$","tone":"brand","quickAdd":[number,number]}
- {"type":"checklist","label":string,"items":[{"label":string,"meta":string,"done":false}]}
- {"type":"week","label":string,"days":[false,false,false,false,false,false,false]}
- {"type":"stat","label":string,"value":string}   // or "dynamic":"remaining" for budgets
- {"type":"list","label":string,"prefix":"$","rows":[{"name":string,"amount":number}]}
- {"type":"lessons","label":string,"items":[{"title":string,"mins":number,"read":false}]}
- {"type":"reminder","title":string,"date":"YYYY-MM-DD","time":"HH:MM","location":string,"remind":true}
- {"type":"calc","label":string,"inputs":[{"key":string,"label":string,"value":number,"prefix":"$"}],"formula":"savingsTimeline"|"tip"|"rentAffordable"|"splitBill"|"takeHome","resultLabel":string}
- {"type":"docs","label":string,"items":[{"name":string,"meta":string,"status":"ok"|"soon"|"late"}]}
- {"type":"note","text":string}

Pick blocks that genuinely fit: savings -> amount + progress + calc; an
appointment -> reminder + checklist; a budget -> amount + list + stat(remaining);
a habit -> week + checklist; learning -> lessons; tracking documents or IDs
(passport, licence, lease, insurance policy) -> docs, where "meta" is a short
expiry/renewal note and "status" is ok (valid), soon (renew soon) or late
(expired). Money is in US dollars ($).
Keep copy warm, plain and encouraging. Never use emoji.
"#;

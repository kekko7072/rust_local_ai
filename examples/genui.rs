//! Generate a genUI module on-device and print it as A2UI v0.9 JSONL.
//!
//! cargo run --example genui --features a2ui -- "Save $500 for a weekend trip"

use rust_local_ai::{
    detect,
    genui::{GenUiOptions, LocalAiUiGenerator, A2UI_V09_BASIC_CATALOG_ID},
};

#[tokio::main]
async fn main() -> rust_local_ai::Result<()> {
    let goal = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Save $500 for a weekend trip".into());
    let model = detect().await?;
    let availability = model.availability().await;
    if !availability.available {
        eprintln!("Local AI unavailable: {:?}", availability.reason);
        return Ok(());
    }

    let module = LocalAiUiGenerator::new(model)
        .generate_module(&goal, &GenUiOptions::default())
        .await?;
    eprintln!("{} — {}", module.title, module.blurb);
    for message in module.to_a2ui_v09_messages("genui", A2UI_V09_BASIC_CATALOG_ID) {
        println!(
            "{}",
            serde_json::to_string(&message).expect("A2UI messages serialize")
        );
    }
    Ok(())
}

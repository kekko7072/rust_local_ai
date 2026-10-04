use rust_local_ai::{detect, GenerationConfig};

#[tokio::main]
async fn main() -> rust_local_ai::Result<()> {
    let model = detect().await?;
    let availability = model.availability().await;
    if !availability.available {
        println!("Local AI unavailable: {:?}", availability.reason);
        return Ok(());
    }
    let session = model
        .open_session(Some("You are a helpful assistant."))
        .await?;
    session
        .add_query_chunk("Explain reinforcement learning simply.")
        .await?;
    println!(
        "{}",
        session.generate(GenerationConfig::default()).await?.text
    );
    session.close().await
}

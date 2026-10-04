use futures_util::StreamExt;
use rust_local_ai::{detect, GenerationConfig};

#[tokio::main]
async fn main() -> rust_local_ai::Result<()> {
    let model = detect().await?;
    if !model.capabilities().await.streaming {
        println!("Streaming is unavailable on {}", model.backend().name);
        return Ok(());
    }
    let session = model.open_session(None).await?;
    session
        .add_query_chunk("Write a haiku about local AI.")
        .await?;
    let mut stream = session.generate_stream(GenerationConfig::default()).await?;
    while let Some(chunk) = stream.next().await {
        print!("{}", chunk?);
    }
    Ok(())
}

use rust_local_ai::detect;

#[tokio::main]
async fn main() -> rust_local_ai::Result<()> {
    let model = detect().await?;
    println!("backend: {:#?}", model.backend());
    println!("availability: {:#?}", model.availability().await);
    println!("capabilities: {:#?}", model.capabilities().await);
    Ok(())
}

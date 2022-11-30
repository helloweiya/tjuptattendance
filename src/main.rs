use anyhow::Result;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    log::debug!("Hello, Word!");
    match mma().await {
        Ok(_) => log::debug!("Good!"),
        Err(e) => log::error!("Error: {}", e),
    }
}

async fn mma() -> Result<()> {
    Ok(())
}

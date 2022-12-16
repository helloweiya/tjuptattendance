use anyhow::Result;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Debug)
        .with_module_level("reqwest", log::LevelFilter::Error)
        .with_module_level("cookie_store", log::LevelFilter::Error)
        .with_module_level("selectors", log::LevelFilter::Error)
        .with_module_level("html5ever", log::LevelFilter::Error)
        .init()
        .unwrap();

    match mma().await {
        Ok(_) => {}
        Err(e) => log::error!("Error: {}", e),
    }
}

async fn mma() -> Result<()> {
    // 解析命令行
    libs::bot::attendance().await?;
    Ok(())
}

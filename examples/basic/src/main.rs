#[tokio::main]
async fn main() -> plethora::Result<()> {
    let _guard = plethora::stuff::builder().file("stuff.toml", true).init()?;
    plethora::scratch::init().await?;
    plethora::binary::install().await?;

    Ok(())
}

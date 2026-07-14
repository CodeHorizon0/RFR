mod runtime;
mod worker;
mod server;

use runtime::FunctionsRuntime;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = FunctionsRuntime::new();

    runtime.load_directory(PathBuf::from("./js")).await?;

    server::start(runtime).await?;

    Ok(())
}

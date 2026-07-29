use std::{env, net::SocketAddr};

use anyhow::Result;
use tarkov_item_manager::{build_app, config::Config};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    let app = build_app(config.clone()).await?;
    let address: SocketAddr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let browser_url = format!("http://127.0.0.1:{}/login", listener.local_addr()?.port());
    tracing::info!(%browser_url, "服务已启动");
    if config.auto_open_browser {
        if let Err(error) = open::that(&browser_url) {
            tracing::warn!(%browser_url, %error, "无法自动打开浏览器");
        }
    }
    axum::serve(listener, app).await?;
    Ok(())
}

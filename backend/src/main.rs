use std::{env, net::SocketAddr};

use anyhow::Result;
use tarkov_item_manager::{build_app, config::Config};
use tracing_subscriber::EnvFilter;

const APP_NAME: &str = "Tarkov Item Manager";
const REPOSITORY_URL: &str = "https://github.com/BlazeSnow/TarkovItemManager";

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
    let browser_url = local_browser_url(listener.local_addr()?.port());
    println!(
        "{APP_NAME}\nVersion: {}\nRepository: {REPOSITORY_URL}\n\nOpen: {browser_url}",
        app_version()
    );
    tracing::info!(%browser_url, "服务已启动");
    if config.desktop_app {
        if let Err(error) = open::that(&browser_url) {
            tracing::warn!(%browser_url, %error, "无法自动打开浏览器");
        }
    }
    axum::serve(listener, app).await?;
    Ok(())
}

fn app_version() -> &'static str {
    option_env!("TARKOV_ITEM_MANAGER_VERSION").unwrap_or("dev")
}

fn local_browser_url(port: u16) -> String {
    format!("http://localhost:{port}/login")
}

#[cfg(test)]
mod tests {
    use super::{app_version, local_browser_url};

    #[test]
    fn defaults_to_development_version() {
        assert_eq!(app_version(), "dev");
    }

    #[test]
    fn formats_local_browser_url() {
        assert_eq!(local_browser_url(3000), "http://localhost:3000/login");
    }
}

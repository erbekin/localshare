use std::path::PathBuf;

use anyhow::Context;
use localshare::{assets, config::Config, qr, server::Server, mdns};
use tokio::fs;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;
    let matches = localshare::cli::get_command().get_matches();

    match matches.subcommand().expect("subcommand required") {
        ("run", m) => {
            let path = m.get_one("workdir").expect("workdir is required argument");
            handle_run(path)
                .await
                .context("Failed to start the server")?;
        }
        ("new", m) => {
            let path = m.get_one("workdir").expect("workdir is required argument");
            handle_new(path)
                .await
                .context("Failed to initialise server directory")?;
        }
        _ => {
            unreachable!("no other subcmd");
        }
    };
    Ok(())
}

async fn handle_new(path: &PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!(
            "Directory '{}' already exists. Remove it first or choose a different path.",
            path.display()
        )
    }
    fs::create_dir(path)
        .await
        .context("Could not create directory")?;
    let config = Config::default();
    fs::create_dir(path.join(&config.path.r#static)).await?;
    fs::create_dir(path.join(&config.path.uploads)).await?;

    let assets = assets::Assets::new();
    assets
        .check_consistency()
        .context("Failed to verify embedded static assets")?;
    assets
        .extract_to_dir(path.join(&config.path.r#static))
        .await?;

    config
        .write_path(path)
        .await
        .context("Failed to write LocalShare.toml")?;

    println!(
        "New LocalShare server configuration has been created at {}",
        path.display()
    );
    Ok(())
}

async fn handle_run(path: &PathBuf) -> anyhow::Result<()> {
    let conf = Config::read_path_and_validate(path)
        .await
        .context(format!(
            "Failed to read configuration from '{}'. \
             Ensure the directory was initialised with 'localshare new'.",
            path.display()
        ))?;

    let mut mdns_service = mdns::start_service(&conf)
        .context("Could not start mDNS service")?;
    qr::generate_qr(path, &conf);

    let server = Server::new(path, conf)?;
    server.launch().await?;
    mdns_service.shutdown();
    Ok(())
}

fn init_logger() -> anyhow::Result<()> {
    use env_logger::{Builder, Env};
    Builder::new()
        .parse_env(
            Env::new()
                .default_filter_or("warn,localshare=info")
                .default_write_style_or("auto"),
        )
        .try_init()
        .context("Logger initialization failed")?;
    Ok(())
}
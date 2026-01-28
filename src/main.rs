use std::path::{PathBuf};

use anyhow::{Context};
use localshare::{assets, config::Config, server::Server};
use tokio::fs;




#[rocket::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;
    let matches = localshare::cli::get_command().get_matches();

    match matches.subcommand().expect("subcommand required") {
        ("run", m) => {
            let path = m.get_one("workdir").expect("workdir is required argument");
            handle_run(path).await.context("command failed, I have nothing to do with it.")?;
        }
        ("new", m) => {
            let path = m.get_one("workdir").expect("workdir is required argument");
            handle_new(path).await.context("hint: you may remove directory and try again")?;
        }
        _ => {
            unreachable!("no other subcmd");
        }
    };
    Ok(())
}


async fn handle_new(path : &PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!("The directory already exists")
    }
    fs::create_dir(path).await.context("Could not create directory")?;
    let config = Config::default();
    fs::create_dir(path.join(&config.path.r#static)).await?;
    fs::create_dir(path.join(&config.path.uploads)).await?;

    let assets = assets::Assets::new();
    assets.check_consistency().context("BuildError: assets module got error")?;
    assets.extract_to_dir(path.join(&config.path.r#static)).await?;

    config.write_path(path).await.context("write error")?;

    println!("New LocalShare server configuration has been created at {}", path.display());
    Ok(())
}
async fn handle_run(path : &PathBuf) -> anyhow::Result<()> {
    let conf = Config::read_path_and_validate(path).await.context("ConfigError")?;
    let server = Server::new(path, conf)?;
    server.launch().await?;
    Ok(())
}

fn init_logger() -> anyhow::Result<()> {
    use env_logger::{Builder, Env};
    Builder::new()
        .parse_env(Env::new()
            .default_filter_or("info")
            .default_write_style_or("auto"))
            .try_init().context("Logger initialization failed")?;
    Ok(())
}

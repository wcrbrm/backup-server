mod args;
mod endpoints;
mod logging;
mod realms;
mod s3;

use args::Command;
use realms::RealmsConfig;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    color_eyre::install().unwrap();
    logging::start("INFO");
    dotenv::dotenv().unwrap();

    let opt = match args::parse() {
        Ok(x) => x,
        Err(e) => {
            panic!("Args parsing error: {}", e);
        }
    };

    match opt.cmd {
        Command::OpenApi => {
            use endpoints::openapi::*;
            println!("{}", serde_json::to_string(&openapi()).unwrap());
        }
        Command::Server { listen, config } => {
            endpoints::run(&listen, config).await?;
        }
        Command::Stat { name, config } => {
            // get realm from name
            let cfg = RealmsConfig::from_toml(&config).expect("realms config");
            if name.is_empty() {
                for (name, realm) in cfg.realms {
                    let (size, total, last_update) = realm.stat().await?;
                    println!(
                        "[{}] {:?}, {} bytes,{} files",
                        name, last_update, size, total
                    )
                }
            } else {
                let errmsg = format!("unknown realm {}, found {:?}", name, cfg.realms.keys());
                let realm = cfg.realms.get(&name).expect(&errmsg);
                let (size, total, last_update) = realm.stat().await?;
                println!(
                    "[{}] {:?}, {} bytes, {} files",
                    name, last_update, size, total,
                );
            }
        }

        Command::Push {
            file,
            name,
            exchange_dir,
            config,
        } => {
            // get realm from name
            let cfg = RealmsConfig::from_toml(&config).expect("realms config");
            let errmsg = format!("unknown realm {}, found {:?}", name, cfg.realms.keys());
            let realm = cfg.realms.get(&name).expect(&errmsg);
            let path = std::path::Path::new(&exchange_dir).join(&file);
            let size = realm.push(&path).await?;
            println!("Uploaded {} bytes", size);
        }
        Command::Pull {
            name,
            exchange_dir,
            config,
        } => {
            let cfg = RealmsConfig::from_toml(&config).expect("realms config");
            let errmsg = format!("unknown realm {}, found {:?}", name, cfg.realms.keys());
            let realm = cfg.realms.get(&name).expect(&errmsg);
            let output = realm.pull(Path::new(&exchange_dir)).await?;
            println!("Saved as {}", output.display());
        }
    }
    Ok(())
}

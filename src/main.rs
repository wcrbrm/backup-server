mod args;
mod endpoints;
mod logging;

use args::Command;
// TODO: read config

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    color_eyre::install().unwrap();
    logging::start("INFO");

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
        Command::Push {
            file,
            name,
            exchange_dir,
            config,
        } => {
            todo!()
        }
        Command::Pull {
            name,
            exchange_dir,
            config,
        } => {
            todo!()
        }
    }
    Ok(())
}

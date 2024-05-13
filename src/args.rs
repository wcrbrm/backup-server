use clap::{Parser, Subcommand};

/// command to execute
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// product open api JSON document
    OpenApi,
    /// serve API with metrics via HTTP
    Server {
        /// Net listening address of HTTP server in case of "server" command
        #[clap(long, default_value = "0.0.0.0:8000", env = "LISTEN")]
        listen: String,
        /// realms configuration TOML file path
        #[clap(short, long, env = "CONFIG_FILE")]
        config: String,
    },
    /// Get statistics on the realm
    Stat {
        /// name of the realm. recommended name format are (project)-(typeofdb)-(db)
        #[clap(short, long, default_value = "")]
        name: String,
        /// file name to be sent to archive
        /// realms configuration TOML file path
        #[clap(short, long, env = "CONFIG_FILE")]
        config: String,
    },
    /// send backup file to remote archive
    Push {
        /// name of the realm. recommended name format are (project)-(typeofdb)-(db)
        #[clap(short, long)]
        name: String,
        /// file name to be sent to archive
        #[clap(short, long)]
        file: String,
        /// remove the original file if successfully uploaded
        #[clap(long)]
        clean: bool,
        /// exchange dir
        #[clap(short, long, env = "EXCHANGE_DIR")]
        exchange_dir: String,
        /// realms configuration TOML file path
        #[clap(short, long, env = "CONFIG_FILE")]
        config: String,
    },
    /// ensure the backup file is downloaded from remote archive into exchange folder
    Pull {
        /// name of the realm. recommended name format are (project)-(typeofdb)-(db)
        #[clap(short, long)]
        name: String,
        /// exchange dir
        #[clap(short, long, env = "EXCHANGE_DIR")]
        exchange_dir: String,
        /// realms configuration TOML file path
        #[clap(short, long, env = "CONFIG_FILE")]
        config: String,
    },
}

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "backup-server",
    about = "CLI utility to backups migration and HTTP server with metrics"
)]
pub struct Cli {
    /// Command to be executed
    #[clap(subcommand)]
    pub cmd: Command,
    /// Logging level
    #[clap(long, env = "RUST_LOG", default_value = "")]
    pub log_level: String,
}

pub fn parse() -> anyhow::Result<Cli> {
    let res = Cli::parse();
    Ok(res)
}

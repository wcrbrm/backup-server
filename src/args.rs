use clap::{Parser, Subcommand};
/*
pub struct SshRealm {
    /// remote SSH host
    #[clap(long, default_value = "", env = "REMOTE_SSH_HOST")]
    pub remote_host: String,
    /// remote SSH user
    #[clap(long, default_value = "root", env = "REMOTE_SSH_USER")]
    pub remote_user: String,
    /// remote SSH port
    #[clap(long, default_value = "22", env = "REMOTE_SSH_PORT")]
    pub remote_port: u16,
    /// remote SSH password, if not provided, will use key file
    #[clap(long, default_value = "", env = "REMOTE_SSH_PASSWORD")]
    pub remote_password: String,
    /// path to id_rsa file
    #[clap(long, default_value = "~/.ssh/id_rsa", env = "REMOTE_SSH_KEY_FILE")]
    pub remote_key_file: String,
}

pub struct S3Realm {
    /// AWS S3 access key
    #[clap(long, env = "ARCHIVE_S3_ACCESS_KEY_ID")]
    pub s3_access_key: String,
    /// AWS S3 secret key
    #[clap(long, env = "ARCHIVE_S3_SECRET_ACCESS_KEY")]
    pub s3_secret_access_key: String,
    /// AWS S3 bucket name
    #[clap(long, env = "ARCHIVE_S3_BUCKET")]
    pub s3_bucket: String,
    /// AWS S3 region name
    #[clap(long, env = "ARCHIVE_S3_REGION")]
    pub s3_region: String,
}
*/

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
    /// send backup file to remote archive
    Push {
        /// name of the database. recommended name format are (project)-(typeofdb)-(db)
        #[clap(short, long)]
        name: String,
        /// file name to be sent to archive
        #[clap(short, long)]
        file: String,
        /// exchange dir
        #[clap(short, long, env = "EXCHANGE_DIR")]
        exchange_dir: String,
        /// realms configuration TOML file path
        #[clap(short, long, env = "CONFIG_FILE")]
        config: String,
    },
    /// ensure the backup file is downloaded from remote archive into exchange folder
    Pull {
        /// name of the database. recommended name format are (project)-(typeofdb)-(db)
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

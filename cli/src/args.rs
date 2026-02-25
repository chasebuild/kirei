use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "cli-template",
    about = "Minimal Rust CLI template with clap + cliclack"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub config: ConfigOptions,
}

#[derive(Args, Debug, Clone)]
pub struct ConfigOptions {
    #[arg(long, default_value = "com.example")]
    pub config_qualifier: String,

    #[arg(long, default_value = "cli-templates")]
    pub config_organization: String,

    #[arg(long, default_value = "cli-template")]
    pub config_application: String,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create or update the local configuration
    Init(InitArgs),

    /// Run the greeting again the way the template does
    Greet(GreetArgs),

    /// Inspect the stored configuration
    Config(ConfigArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Provide the user name without prompting
    #[arg(long)]
    pub user_name: Option<String>,

    /// Overwrite an existing config without asking
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct GreetArgs {
    /// Override the user name used for greeting
    #[arg(long)]
    pub user_name: Option<String>,
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Dump the stored config as JSON
    Show,
    /// Print the path to the config file
    Path,
}

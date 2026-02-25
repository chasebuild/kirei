use clap::{Args, Parser, Subcommand, ValueEnum};
use cli_template_core::unified::ProviderId;

#[derive(Parser, Debug)]
#[command(name = "kirei", about = "Interactive CLI for unified provider APIs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create or update the local configuration
    Init(InitArgs),

    /// Run the greeting again the way the template does
    Greet(GreetArgs),

    /// Inspect the stored configuration
    Config(ConfigArgs),

    /// Explore unified issue streams across providers
    Unified(UnifiedArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Provide the user name without prompting
    #[arg(long)]
    pub user_name: Option<String>,

    /// Pick a default provider when none is specified
    #[arg(long, value_enum)]
    pub default_provider: Option<CliProvider>,

    /// Default GitHub repo when none is supplied
    #[arg(long)]
    pub default_repo: Option<String>,

    /// Default Linear workspace when none is supplied
    #[arg(long)]
    pub default_workspace: Option<String>,

    /// Store a GitHub token for future requests
    #[arg(long)]
    pub github_token: Option<String>,

    /// Store a Linear token for future requests
    #[arg(long)]
    pub linear_token: Option<String>,
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

#[derive(Args, Debug)]
pub struct UnifiedArgs {
    /// Target provider (github, linear, trello, jira)
    #[arg(long, value_enum)]
    pub provider: Option<CliProvider>,

    /// Operation to perform (interactive, list, create)
    #[arg(long, value_enum)]
    pub mode: Option<UnifiedMode>,

    /// Workspace identifier (Linear) or other provider-scoped context
    #[arg(long)]
    pub workspace: Option<String>,

    /// Repo identifier (owner/repo) for GitHub
    #[arg(long)]
    pub repo: Option<String>,

    /// Title for issue creation
    #[arg(long)]
    pub title: Option<String>,

    /// Body for issue creation
    #[arg(long)]
    pub body: Option<String>,

    /// Optional search/filter expression for lists
    #[arg(long)]
    pub search: Option<String>,

    /// Dump the raw provider payload after listing or creating
    #[arg(long)]
    pub raw: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CliProvider {
    Github,
    Linear,
    Trello,
    Jira,
}

impl From<CliProvider> for ProviderId {
    fn from(provider: CliProvider) -> Self {
        match provider {
            CliProvider::Github => ProviderId::Github,
            CliProvider::Linear => ProviderId::Linear,
            CliProvider::Trello => ProviderId::Trello,
            CliProvider::Jira => ProviderId::Jira,
        }
    }
}

impl From<ProviderId> for CliProvider {
    fn from(provider: ProviderId) -> Self {
        match provider {
            ProviderId::Github => CliProvider::Github,
            ProviderId::Linear => CliProvider::Linear,
            ProviderId::Trello => CliProvider::Trello,
            ProviderId::Jira => CliProvider::Jira,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum UnifiedMode {
    Interactive,
    List,
    Create,
}

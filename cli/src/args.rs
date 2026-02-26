use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "kirei", about = "Unified CLI for issue trackers")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List all issues across providers
    Ls(ListArgs),
    /// Create a new issue
    New(CreateArgs),
    /// GitHub commands
    #[command(subcommand)]
    Github(GitHubCommands),
    /// Linear commands
    #[command(subcommand)]
    Linear(LinearCommands),
    /// Trello commands
    #[command(subcommand)]
    Trello(TrelloCommands),
    /// Jira commands
    #[command(subcommand)]
    Jira(JiraCommands),
    /// Configuration
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Provider to list from (github, linear, trello, jira)
    #[arg(short, long)]
    pub provider: Option<String>,
    /// Show raw JSON output
    #[arg(short, long)]
    pub raw: bool,
}

#[derive(Parser, Debug)]
pub struct CreateArgs {
    /// Title for the issue
    pub title: String,
    /// Description for the issue
    #[arg(short, long)]
    pub body: Option<String>,
    /// Provider to create on
    #[arg(short, long)]
    pub provider: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum GitHubCommands {
    /// List issues
    Ls(GitHubLsArgs),
    /// Create an issue
    New(GitHubNewArgs),
    /// Authentication setup
    Auth(GitHubAuthArgs),
    /// Repository management
    #[command(subcommand)]
    Repo(GitHubRepoCommands),
    /// List repositories
    Repos(GitHubReposArgs),
}

#[derive(Parser, Debug)]
pub struct GitHubLsArgs {
    /// Repository (owner/repo)
    #[arg(short, long)]
    pub repo: Option<String>,
    /// Issue state (open, closed, all)
    #[arg(short, long, default_value = "open")]
    pub state: String,
    /// Show raw JSON output
    #[arg(short, long)]
    pub raw: bool,
}

#[derive(Parser, Debug)]
pub struct GitHubNewArgs {
    /// Repository (owner/repo)
    #[arg(short, long)]
    pub repo: Option<String>,
    /// Issue title
    pub title: String,
    /// Issue body
    #[arg(short, long)]
    pub body: Option<String>,
}

#[derive(Parser, Debug)]
pub struct GitHubAuthArgs {
    /// Authentication method (token, oauth)
    #[arg(short, long)]
    pub method: Option<String>,
    /// Token or Client ID
    pub value: Option<String>,
    /// Client Secret (for OAuth)
    #[arg(short, long)]
    pub secret: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum GitHubRepoCommands {
    /// Set default repository
    Set(GitHubRepoSetArgs),
    /// Show current repository
    List,
}

#[derive(Parser, Debug)]
pub struct GitHubRepoSetArgs {
    pub repo: String,
}

#[derive(Parser, Debug)]
pub struct GitHubReposArgs {
    /// Show raw JSON output
    #[arg(short, long)]
    pub raw: bool,
}

#[derive(Subcommand, Debug)]
pub enum LinearCommands {
    /// List issues
    Ls(LinearLsArgs),
    /// Create an issue
    New(LinearNewArgs),
    /// Authentication setup
    Auth(LinearAuthArgs),
    /// Workspace management
    #[command(subcommand)]
    Workspace(LinearWorkspaceCommands),
    /// List workspaces
    Workspaces(LinearWorkspacesArgs),
}

#[derive(Parser, Debug)]
pub struct LinearLsArgs {
    /// Workspace
    #[arg(short, long)]
    pub workspace: Option<String>,
    /// Show raw JSON output
    #[arg(short, long)]
    pub raw: bool,
}

#[derive(Parser, Debug)]
pub struct LinearNewArgs {
    /// Workspace
    #[arg(short, long)]
    pub workspace: Option<String>,
    /// Issue title
    pub title: String,
    /// Issue body
    #[arg(short, long)]
    pub body: Option<String>,
}

#[derive(Parser, Debug)]
pub struct LinearAuthArgs {
    /// API token
    pub token: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum LinearWorkspaceCommands {
    /// Set default workspace
    Set(LinearWorkspaceSetArgs),
    /// Show current workspace
    List,
}

#[derive(Parser, Debug)]
pub struct LinearWorkspaceSetArgs {
    pub workspace: String,
}

#[derive(Parser, Debug)]
pub struct LinearWorkspacesArgs {
    /// Show raw JSON output
    #[arg(short, long)]
    pub raw: bool,
}

#[derive(Subcommand, Debug)]
pub enum TrelloCommands {
    /// List cards
    Ls(TrelloLsArgs),
    /// Create a card
    New(TrelloNewArgs),
    /// Authentication setup
    Auth(TrelloAuthArgs),
    /// Board management
    #[command(subcommand)]
    Board(TrelloBoardCommands),
    /// List boards
    Boards(TrelloBoardsArgs),
}

#[derive(Parser, Debug)]
pub struct TrelloLsArgs {
    /// Board name or ID
    #[arg(short, long)]
    pub board: Option<String>,
    /// Show raw JSON output
    #[arg(short, long)]
    pub raw: bool,
}

#[derive(Parser, Debug)]
pub struct TrelloNewArgs {
    /// Board name or ID
    #[arg(short, long)]
    pub board: Option<String>,
    /// Card name
    pub name: String,
    /// Card description
    #[arg(short, long)]
    pub description: Option<String>,
}

#[derive(Parser, Debug)]
pub struct TrelloAuthArgs {
    /// API key
    #[arg(long)]
    pub api_key: Option<String>,
    /// Token
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum TrelloBoardCommands {
    /// Set default board
    Set(TrelloBoardSetArgs),
    /// Show current board
    List,
}

#[derive(Parser, Debug)]
pub struct TrelloBoardSetArgs {
    pub board: String,
}

#[derive(Parser, Debug)]
pub struct TrelloBoardsArgs {
    /// Show raw JSON output
    #[arg(short, long)]
    pub raw: bool,
}

#[derive(Subcommand, Debug)]
pub enum JiraCommands {
    /// List issues
    Ls(JiraLsArgs),
    /// Create an issue
    New(JiraNewArgs),
    /// Authentication setup
    Auth(JiraAuthArgs),
    /// Project management
    #[command(subcommand)]
    Project(JiraProjectCommands),
    /// List projects
    Projects(JiraProjectsArgs),
}

#[derive(Parser, Debug)]
pub struct JiraLsArgs {
    /// Project key
    #[arg(short, long)]
    pub project: Option<String>,
    /// Show raw JSON output
    #[arg(short, long)]
    pub raw: bool,
}

#[derive(Parser, Debug)]
pub struct JiraNewArgs {
    /// Project key
    #[arg(short, long)]
    pub project: Option<String>,
    /// Issue summary
    pub summary: String,
    /// Issue description
    #[arg(short, long)]
    pub description: Option<String>,
}

#[derive(Parser, Debug)]
pub struct JiraAuthArgs {
    /// Server URL (e.g., https://company.atlassian.net)
    #[arg(long)]
    pub server: Option<String>,
    /// Email
    #[arg(long)]
    pub email: Option<String>,
    /// API token
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum JiraProjectCommands {
    /// Set default project
    Set(JiraProjectSetArgs),
    /// Show current project
    List,
}

#[derive(Parser, Debug)]
pub struct JiraProjectSetArgs {
    pub project: String,
}

#[derive(Parser, Debug)]
pub struct JiraProjectsArgs {
    /// Show raw JSON output
    #[arg(short, long)]
    pub raw: bool,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Show config file path
    Path,
    /// Set default provider
    Provider(ConfigProviderArgs),
}

#[derive(Parser, Debug)]
pub struct ConfigProviderArgs {
    pub provider: String,
}

use std::env;
use std::net::TcpListener;

use anyhow::Result;
use cliclack::{input, intro, outro, select};

use crate::args::*;
use cli_template_core::config::{Config, ConfigStore};
use kirei_provider_github::{
    GitHubClient, GitHubIssue,
    oauth::{start_callback_server, wait_for_callback},
};
use kirei_provider_linear::{LinearClient, LinearIssue};
use kirei_provider_trello::{TrelloCard, TrelloClient};
use kirei_provider_jira::{JiraClient, JiraIssue};

fn intro_message(section: &str) -> Result<()> {
    intro(format!("kirei {}", section))?;
    Ok(())
}

fn outro_message(message: &str) -> Result<()> {
    outro(message)?;
    Ok(())
}

pub async fn run(cli: Cli) -> Result<()> {
    let store = ConfigStore::new()?;

    match cli.command {
        Command::Ls(args) => ls_command(args, &store).await,
        Command::New(args) => new_command(args, &store).await,
        Command::Github(cmd) => github_command(cmd, &store).await,
        Command::Linear(cmd) => linear_command(cmd, &store).await,
        Command::Trello(cmd) => trello_command(cmd, &store).await,
        Command::Jira(cmd) => jira_command(cmd, &store).await,
        Command::Config(cmd) => config_command(cmd, &store),
    }
}

async fn ls_command(args: ListArgs, store: &ConfigStore) -> Result<()> {
    let config = store.load_or_default()?;

    if let Some(provider) = args.provider {
        match provider.as_str() {
            "github" => {
                let token = resolve_github_token(&config)?;
                let client = GitHubClient::new(token, config.github.default_repo.clone());
                let issues = client.list_issues(None, None).await?;
                display_github_issues(&issues, args.raw)?;
            }
            "linear" => {
                let token = resolve_linear_token(&config)?;
                let client = LinearClient::new(token, config.linear.default_workspace.clone());
                let issues = client.list_issues(None).await?;
                display_linear_issues(&issues, args.raw)?;
            }
            "trello" => {
                let token = resolve_trello_token(&config)?;
                let api_key = config.trello.api_key.clone().unwrap_or_default();
                let client = TrelloClient::new(token, api_key, config.trello.default_board.clone());
                let cards = client.list_cards(None).await?;
                display_trello_cards(&cards, args.raw)?;
            }
            "jira" => {
                let token = resolve_jira_token(&config)?;
                let server_url = config.jira.server_url.clone().unwrap_or_default();
                let client = JiraClient::new(token, server_url, config.jira.default_project.clone());
                let issues = client.list_issues(None).await?;
                display_jira_issues(&issues, args.raw)?;
            }
            _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider)),
        }
    } else {
        intro_message("all providers")?;

        // GitHub
        if let Ok(token) = resolve_github_token(&config) {
            let client = GitHubClient::new(token, config.github.default_repo.clone());
            if let Ok(issues) = client.list_issues(None, None).await {
                println!("\n\x1b[1mGitHub Issues:\x1b[0m");
                display_github_issues(&issues, false)?;
            }
        }

        // Linear
        if let Ok(token) = resolve_linear_token(&config) {
            let client = LinearClient::new(token, config.linear.default_workspace.clone());
            if let Ok(issues) = client.list_issues(None).await {
                println!("\n\x1b[1mLinear Issues:\x1b[0m");
                display_linear_issues(&issues, false)?;
            }
        }

        // Trello
        if let (Ok(token), Ok(api_key)) = (
            resolve_trello_token(&config),
            config.trello.api_key.clone().ok_or_else(|| anyhow::anyhow!("no api key"))
        ) {
            let client = TrelloClient::new(token, api_key, config.trello.default_board.clone());
            if let Ok(cards) = client.list_cards(None).await {
                println!("\n\x1b[1mTrello Cards:\x1b[0m");
                display_trello_cards(&cards, false)?;
            }
        }

        // Jira
        if let Ok(token) = resolve_jira_token(&config) {
            let server_url = config.jira.server_url.clone().unwrap_or_default();
            let client = JiraClient::new(token, server_url, config.jira.default_project.clone());
            if let Ok(issues) = client.list_issues(None).await {
                println!("\n\x1b[1mJira Issues:\x1b[0m");
                display_jira_issues(&issues, false)?;
            }
        }
    }

    Ok(())
}

async fn new_command(args: CreateArgs, store: &ConfigStore) -> Result<()> {
    let config = store.load_or_default()?;

    let provider = args.provider.unwrap_or_else(|| config.default_provider.clone());

    match provider.as_str() {
        "github" => {
            let token = resolve_github_token(&config)?;
            let client = GitHubClient::new(token, config.github.default_repo.clone());
            let issue = client.create_issue(None, &args.title, args.body.as_deref()).await?;
            println!("Created GitHub issue #{}: {}", issue.number, issue.title);
            if let Some(url) = issue.html_url {
                println!("URL: {}", url);
            }
        }
        "linear" => {
            let token = resolve_linear_token(&config)?;
            let client = LinearClient::new(token, config.linear.default_workspace.clone());
            let issue = client.create_issue(None, &args.title, args.body.as_deref()).await?;
            println!("Created Linear issue {}: {}", issue.id, issue.title);
            if let Some(url) = issue.url {
                println!("URL: {}", url);
            }
        }
        "trello" => {
            let token = resolve_trello_token(&config)?;
            let api_key = config.trello.api_key.clone().unwrap_or_default();
            let client = TrelloClient::new(token, api_key, config.trello.default_board.clone());
            let card = client.create_card(None, &args.title, args.body.as_deref()).await?;
            println!("Created Trello card: {}", card.name);
            if let Some(url) = card.url {
                println!("URL: {}", url);
            }
        }
        "jira" => {
            let token = resolve_jira_token(&config)?;
            let server_url = config.jira.server_url.clone().unwrap_or_default();
            let client = JiraClient::new(token, server_url, config.jira.default_project.clone());
            let issue = client.create_issue(None, &args.title, args.body.as_deref()).await?;
            println!("Created Jira issue {}: {}", issue.key, issue.summary);
            if let Some(url) = issue.url {
                println!("URL: {}", url);
            }
        }
        _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider)),
    }

    Ok(())
}

async fn github_command(cmd: GitHubCommands, store: &ConfigStore) -> Result<()> {
    let config = store.load_or_default()?;

    match cmd {
        GitHubCommands::Ls(args) => {
            intro_message("github ls")?;
            let token = resolve_github_token(&config)?;
            let client = GitHubClient::new(token, config.github.default_repo.clone());
            let issues = client.list_issues(None, Some(&args.state)).await?;
            display_github_issues(&issues, args.raw)?;
            outro_message("Done")?;
        }
        GitHubCommands::New(args) => {
            intro_message("github new")?;
            let token = resolve_github_token(&config)?;
            let client = GitHubClient::new(token, config.github.default_repo.clone());
            let issue = client.create_issue(None, &args.title, args.body.as_deref()).await?;
            println!("Created GitHub issue #{}: {}", issue.number, issue.title);
        }
        GitHubCommands::Auth(args) => github_auth(args, store).await?,
        GitHubCommands::Repo(cmd) => github_repo_command(cmd, store)?,
        GitHubCommands::Repos(_args) => {
            intro_message("github repos")?;
            let token = resolve_github_token(&config)?;
            let client = GitHubClient::new(token, config.github.default_repo.clone());
            let repos = client.list_repositories().await?;
            for repo in repos {
                println!("{} - {}", repo.full_name, repo.description.unwrap_or_default());
            }
        }
    }
    Ok(())
}

async fn github_auth(args: GitHubAuthArgs, store: &ConfigStore) -> Result<()> {
    intro_message("github auth")?;
    let mut config = store.load_or_default()?;

    let method: String = if let Some(m) = args.method {
        m
    } else {
        select("Choose authentication method:")
            .item("token", "Personal Access Token", "Enter a GitHub PAT")
            .item("oauth", "OAuth", "Authenticate via browser")
            .interact()?
            .to_string()
    };

    match method.as_str() {
        "token" => {
            let token = args.value
                .or_else(|| {
                    input("GitHub personal access token:")
                        .validate(|v: &String| {
                            if v.trim().is_empty() { Err("Token cannot be empty") } else { Ok(()) }
                        })
                        .interact()
                        .ok()
                })
                .map(|s| s.trim().to_string());

            if let Some(token) = token {
                config.github.token = Some(token.clone());
                store.save(&config)?;
                println!("Token saved successfully.");
            }
        }
        "oauth" => {
            let client_id = if let Some(id) = args.value.clone() {
                id
            } else if let Some(id) = config.github.client_id.clone() {
                id
            } else {
                input("GitHub OAuth App Client ID (will be saved):")
                    .validate(|v: &String| {
                        if v.trim().is_empty() { Err("Client ID cannot be empty") } else { Ok(()) }
                    })
                    .interact()?
            };

            let client_secret = if let Some(secret) = args.secret.clone() {
                secret
            } else if let Some(secret) = config.github.client_secret.clone() {
                secret
            } else {
                input("GitHub OAuth App Client Secret (will be saved):")
                    .validate(|v: &String| {
                        if v.trim().is_empty() { Err("Client Secret cannot be empty") } else { Ok(()) }
                    })
                    .interact()?
            };

            config.github.client_id = Some(client_id.clone());
            config.github.client_secret = Some(client_secret.clone());

            let port = get_available_port()?;
            let oauth = kirei_provider_github::oauth::GitHubOAuth::new(client_id, client_secret);
            let auth_url = oauth.get_authorization_url(port);

            println!("\n\x1b[1mVisit this link to authenticate:\x1b[0m");
            println!("{}\n", auth_url);

            let (code_rx, close_rx) = start_callback_server(port)?;
            println!("Waiting for authorization... (press Ctrl+C to cancel)");
            
            if !wait_for_callback(close_rx, 300) {
                return Err(anyhow::anyhow!("Authorization timed out"));
            }

            let code = code_rx.recv().map_err(|_| anyhow::anyhow!("Failed to receive code"))?;

            let token = kirei_provider_github::exchange_code_for_token(
                config.github.client_id.as_ref().unwrap(),
                config.github.client_secret.as_ref().unwrap(),
                &code,
            ).await?;

            config.github.token = Some(token);
            store.save(&config)?;
            println!("Authentication successful!");
        }
        _ => return Err(anyhow::anyhow!("Unknown auth method: {}", method)),
    }

    outro_message("Done")?;
    Ok(())
}

fn github_repo_command(cmd: GitHubRepoCommands, store: &ConfigStore) -> Result<()> {
    let mut config = store.load_or_default()?;

    match cmd {
        GitHubRepoCommands::Set(args) => {
            config.github.default_repo = Some(args.repo);
            store.save(&config)?;
            println!("Default repository set.");
        }
        GitHubRepoCommands::List => {
            if let Some(repo) = &config.github.default_repo {
                println!("Default repository: {}", repo);
            } else {
                println!("No default repository set.");
            }
        }
    }
    Ok(())
}

async fn linear_command(cmd: LinearCommands, store: &ConfigStore) -> Result<()> {
    let config = store.load_or_default()?;

    match cmd {
        LinearCommands::Ls(args) => {
            intro_message("linear ls")?;
            let token = resolve_linear_token(&config)?;
            let client = LinearClient::new(token, config.linear.default_workspace.clone());
            let issues = client.list_issues(None).await?;
            display_linear_issues(&issues, args.raw)?;
            outro_message("Done")?;
        }
        LinearCommands::New(args) => {
            intro_message("linear new")?;
            let token = resolve_linear_token(&config)?;
            let client = LinearClient::new(token, config.linear.default_workspace.clone());
            let issue = client.create_issue(None, &args.title, args.body.as_deref()).await?;
            println!("Created Linear issue {}: {}", issue.id, issue.title);
        }
        LinearCommands::Auth(args) => linear_auth(args, store)?,
        LinearCommands::Workspace(cmd) => linear_workspace_command(cmd, store)?,
        LinearCommands::Workspaces(_args) => {
            intro_message("linear workspaces")?;
            let token = resolve_linear_token(&config)?;
            let client = LinearClient::new(token, config.linear.default_workspace.clone());
            let workspaces = client.list_workspaces().await?;
            for ws in workspaces {
                println!("{} ({})", ws.name, ws.slug);
            }
        }
    }
    Ok(())
}

fn linear_auth(args: LinearAuthArgs, store: &ConfigStore) -> Result<()> {
    intro_message("linear auth")?;
    let mut config = store.load_or_default()?;

    let token = args.token
        .or_else(|| {
            input("Linear API token:")
                .validate(|v: &String| {
                    if v.trim().is_empty() { Err("Token cannot be empty") } else { Ok(()) }
                })
                .interact()
                .ok()
        })
        .map(|s| s.trim().to_string());

    if let Some(token) = token {
        config.linear.token = Some(token);
        store.save(&config)?;
        println!("Token saved successfully.");
    }

    outro_message("Done")?;
    Ok(())
}

fn linear_workspace_command(cmd: LinearWorkspaceCommands, store: &ConfigStore) -> Result<()> {
    let mut config = store.load_or_default()?;

    match cmd {
        LinearWorkspaceCommands::Set(args) => {
            config.linear.default_workspace = Some(args.workspace);
            store.save(&config)?;
            println!("Default workspace set.");
        }
        LinearWorkspaceCommands::List => {
            if let Some(ws) = &config.linear.default_workspace {
                println!("Default workspace: {}", ws);
            } else {
                println!("No default workspace set.");
            }
        }
    }
    Ok(())
}

async fn trello_command(cmd: TrelloCommands, store: &ConfigStore) -> Result<()> {
    let config = store.load_or_default()?;

    match cmd {
        TrelloCommands::Ls(args) => {
            intro_message("trello ls")?;
            let token = resolve_trello_token(&config)?;
            let api_key = config.trello.api_key.clone().unwrap_or_default();
            let client = TrelloClient::new(token, api_key, config.trello.default_board.clone());
            let cards = client.list_cards(None).await?;
            display_trello_cards(&cards, args.raw)?;
            outro_message("Done")?;
        }
        TrelloCommands::New(args) => {
            intro_message("trello new")?;
            let token = resolve_trello_token(&config)?;
            let api_key = config.trello.api_key.clone().unwrap_or_default();
            let client = TrelloClient::new(token, api_key, config.trello.default_board.clone());
            let card = client.create_card(None, &args.name, args.description.as_deref()).await?;
            println!("Created Trello card: {}", card.name);
        }
        TrelloCommands::Auth(args) => trello_auth(args, store)?,
        TrelloCommands::Board(cmd) => trello_board_command(cmd, store)?,
        TrelloCommands::Boards(_args) => {
            intro_message("trello boards")?;
            let token = resolve_trello_token(&config)?;
            let api_key = config.trello.api_key.clone().unwrap_or_default();
            let client = TrelloClient::new(token, api_key, config.trello.default_board.clone());
            let boards = client.list_boards().await?;
            for board in boards {
                println!("{} - {}", board.name, board.url);
            }
        }
    }
    Ok(())
}

fn trello_auth(args: TrelloAuthArgs, store: &ConfigStore) -> Result<()> {
    intro_message("trello auth")?;
    let mut config = store.load_or_default()?;

    let api_key = args.api_key
        .or_else(|| config.trello.api_key.clone())
        .or_else(|| {
            input("Trello API key:")
                .validate(|v: &String| {
                    if v.trim().is_empty() { Err("API key cannot be empty") } else { Ok(()) }
                })
                .interact()
                .ok()
        })
        .map(|s| s.trim().to_string());

    let token = args.token
        .or_else(|| config.trello.token.clone())
        .or_else(|| {
            input("Trello token:")
                .validate(|v: &String| {
                    if v.trim().is_empty() { Err("Token cannot be empty") } else { Ok(()) }
                })
                .interact()
                .ok()
        })
        .map(|s| s.trim().to_string());

    if let Some(key) = api_key {
        config.trello.api_key = Some(key);
    }
    if let Some(tok) = token {
        config.trello.token = Some(tok);
    }

    store.save(&config)?;
    println!("Credentials saved successfully.");
    outro_message("Done")?;
    Ok(())
}

fn trello_board_command(cmd: TrelloBoardCommands, store: &ConfigStore) -> Result<()> {
    let mut config = store.load_or_default()?;

    match cmd {
        TrelloBoardCommands::Set(args) => {
            config.trello.default_board = Some(args.board);
            store.save(&config)?;
            println!("Default board set.");
        }
        TrelloBoardCommands::List => {
            if let Some(board) = &config.trello.default_board {
                println!("Default board: {}", board);
            } else {
                println!("No default board set.");
            }
        }
    }
    Ok(())
}

async fn jira_command(cmd: JiraCommands, store: &ConfigStore) -> Result<()> {
    let config = store.load_or_default()?;

    match cmd {
        JiraCommands::Ls(args) => {
            intro_message("jira ls")?;
            let token = resolve_jira_token(&config)?;
            let server_url = config.jira.server_url.clone().unwrap_or_default();
            let client = JiraClient::new(token, server_url, config.jira.default_project.clone());
            let issues = client.list_issues(None).await?;
            display_jira_issues(&issues, args.raw)?;
            outro_message("Done")?;
        }
        JiraCommands::New(args) => {
            intro_message("jira new")?;
            let token = resolve_jira_token(&config)?;
            let server_url = config.jira.server_url.clone().unwrap_or_default();
            let client = JiraClient::new(token, server_url, config.jira.default_project.clone());
            let issue = client.create_issue(None, &args.summary, args.description.as_deref()).await?;
            println!("Created Jira issue {}: {}", issue.key, issue.summary);
        }
        JiraCommands::Auth(args) => jira_auth(args, store)?,
        JiraCommands::Project(cmd) => jira_project_command(cmd, store)?,
        JiraCommands::Projects(_args) => {
            intro_message("jira projects")?;
            let token = resolve_jira_token(&config)?;
            let server_url = config.jira.server_url.clone().unwrap_or_default();
            let client = JiraClient::new(token, server_url, config.jira.default_project.clone());
            let projects = client.list_projects().await?;
            for project in projects {
                println!("{} - {}", project.key, project.name);
            }
        }
    }
    Ok(())
}

fn jira_auth(args: JiraAuthArgs, store: &ConfigStore) -> Result<()> {
    intro_message("jira auth")?;
    let mut config = store.load_or_default()?;

    let server = args.server
        .or_else(|| config.jira.server_url.clone())
        .or_else(|| {
            input("Jira server URL (e.g., https://company.atlassian.net):")
                .validate(|v: &String| {
                    if v.trim().is_empty() { Err("Server URL cannot be empty") } else { Ok(()) }
                })
                .interact()
                .ok()
        })
        .map(|s| s.trim().to_string());

    let email = args.email
        .or_else(|| config.jira.email.clone())
        .or_else(|| {
            input("Jira email:")
                .validate(|v: &String| {
                    if v.trim().is_empty() { Err("Email cannot be empty") } else { Ok(()) }
                })
                .interact()
                .ok()
        })
        .map(|s| s.trim().to_string());

    let token = args.token
        .or_else(|| config.jira.token.clone())
        .or_else(|| {
            input("Jira API token:")
                .validate(|v: &String| {
                    if v.trim().is_empty() { Err("Token cannot be empty") } else { Ok(()) }
                })
                .interact()
                .ok()
        })
        .map(|s| s.trim().to_string());

    if let Some(s) = server {
        config.jira.server_url = Some(s);
    }
    if let Some(e) = email {
        config.jira.email = Some(e);
    }
    if let Some(t) = token {
        config.jira.token = Some(t);
    }

    store.save(&config)?;
    println!("Credentials saved successfully.");
    outro_message("Done")?;
    Ok(())
}

fn jira_project_command(cmd: JiraProjectCommands, store: &ConfigStore) -> Result<()> {
    let mut config = store.load_or_default()?;

    match cmd {
        JiraProjectCommands::Set(args) => {
            config.jira.default_project = Some(args.project);
            store.save(&config)?;
            println!("Default project set.");
        }
        JiraProjectCommands::List => {
            if let Some(project) = &config.jira.default_project {
                println!("Default project: {}", project);
            } else {
                println!("No default project set.");
            }
        }
    }
    Ok(())
}

fn config_command(cmd: ConfigCommands, store: &ConfigStore) -> Result<()> {
    match cmd {
        ConfigCommands::Show => {
            let config = store.load_or_default()?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommands::Path => {
            println!("{}", store.path().display());
        }
        ConfigCommands::Provider(args) => {
            let mut config = store.load_or_default()?;
            config.default_provider = args.provider;
            store.save(&config)?;
            println!("Default provider set.");
        }
    }
    Ok(())
}

fn resolve_github_token(config: &Config) -> Result<String> {
    if let Ok(env_token) = env::var("KIREI_GITHUB_TOKEN") {
        if !env_token.trim().is_empty() {
            return Ok(env_token);
        }
    }
    config.github.token.clone()
        .ok_or_else(|| anyhow::anyhow!("GitHub token not configured. Run: kirei github auth"))
}

fn resolve_linear_token(config: &Config) -> Result<String> {
    if let Ok(env_token) = env::var("KIREI_LINEAR_TOKEN") {
        if !env_token.trim().is_empty() {
            return Ok(env_token);
        }
    }
    config.linear.token.clone()
        .ok_or_else(|| anyhow::anyhow!("Linear token not configured. Run: kirei linear auth"))
}

fn resolve_trello_token(config: &Config) -> Result<String> {
    if let Ok(env_token) = env::var("KIREI_TRELLO_TOKEN") {
        if !env_token.trim().is_empty() {
            return Ok(env_token);
        }
    }
    config.trello.token.clone()
        .ok_or_else(|| anyhow::anyhow!("Trello token not configured. Run: kirei trello auth"))
}

fn resolve_jira_token(config: &Config) -> Result<String> {
    if let Ok(env_token) = env::var("KIREI_JIRA_TOKEN") {
        if !env_token.trim().is_empty() {
            return Ok(env_token);
        }
    }
    config.jira.token.clone()
        .ok_or_else(|| anyhow::anyhow!("Jira token not configured. Run: kirei jira auth"))
}

fn display_github_issues(issues: &[GitHubIssue], _raw: bool) -> Result<()> {
    if issues.is_empty() {
        println!("No issues found.");
    } else {
        for issue in issues {
            println!("#{} [{}] {}", issue.number, issue.state, issue.title);
            if let Some(url) = &issue.html_url {
                println!("  {}", url);
            }
        }
    }
    Ok(())
}

fn display_linear_issues(issues: &[LinearIssue], _raw: bool) -> Result<()> {
    if issues.is_empty() {
        println!("No issues found.");
    } else {
        for issue in issues {
            println!("{} [{}] {}", issue.id, issue.state, issue.title);
            if let Some(url) = &issue.url {
                println!("  {}", url);
            }
        }
    }
    Ok(())
}

fn display_trello_cards(cards: &[TrelloCard], _raw: bool) -> Result<()> {
    if cards.is_empty() {
        println!("No cards found.");
    } else {
        for card in cards {
            println!("[{}] {}", card.list_name, card.name);
            if let Some(url) = &card.url {
                println!("  {}", url);
            }
        }
    }
    Ok(())
}

fn display_jira_issues(issues: &[JiraIssue], _raw: bool) -> Result<()> {
    if issues.is_empty() {
        println!("No issues found.");
    } else {
        for issue in issues {
            println!("{} [{}] {}", issue.key, issue.status, issue.summary);
            if let Some(url) = &issue.url {
                println!("  {}", url);
            }
        }
    }
    Ok(())
}

fn get_available_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    Ok(port)
}

use std::env;

use anyhow::Result;
use cliclack::{input, intro, note, outro};

use crate::args::{Cli, Command, ConfigCommand, GreetArgs, InitArgs, UnifiedArgs, UnifiedMode};
use cli_template_core::config::{Config, ConfigStore};
use cli_template_core::unified::{
    ProviderClient, ProviderId, UnifiedCreateParams, UnifiedError, UnifiedIssue, UnifiedListQuery,
    github::GitHubClient, jira::JiraClient, linear::LinearClient, trello::TrelloClient,
};
use serde_json::to_string_pretty;

fn prompt_user_name() -> Result<String> {
    let name: String = input("What name should the CLI remember?")
        .placeholder("Ada Lovelace")
        .validate(|value: &String| {
            if value.trim().is_empty() {
                Err("Please enter a name.")
            } else {
                Ok(())
            }
        })
        .interact()?;
    Ok(name)
}

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
        Command::Init(args) => init_command(args, &store),
        Command::Greet(args) => greet_command(args, &store),
        Command::Config(args) => config_command(args.command, &store),
        Command::Unified(args) => unified_command(args, &store).await,
    }
}

fn init_command(args: InitArgs, store: &ConfigStore) -> Result<()> {
    intro_message("init")?;
    let mut config = store.load_or_default()?;

    let user_name = match args.user_name {
        Some(value) => value,
        None => prompt_user_name()?,
    };
    config.user_name = user_name.trim().to_string();

    if let Some(provider) = args.default_provider {
        config.unified.default_provider = provider.into();
    }

    if let Some(repo) = args.default_repo {
        config.unified.default_repo = Some(repo);
    }

    if let Some(workspace) = args.default_workspace {
        config.unified.default_workspace = Some(workspace);
    }

    if let Some(token) = args.github_token {
        config
            .unified
            .tokens
            .insert(ProviderId::Github, token.trim().to_string());
    }

    if let Some(token) = args.linear_token {
        config
            .unified
            .tokens
            .insert(ProviderId::Linear, token.trim().to_string());
    }

    let saved = store.save(&config)?;
    note("Config saved", format!("{}", saved.display()))?;
    outro_message("You're all set!")?;
    Ok(())
}

fn greet_command(args: GreetArgs, store: &ConfigStore) -> Result<()> {
    let config = store.load_or_default()?;
    let name = args
        .user_name
        .unwrap_or(config.user_name)
        .trim()
        .to_string();
    println!("Hello, {name}!");
    Ok(())
}

fn config_command(command: ConfigCommand, store: &ConfigStore) -> Result<()> {
    match command {
        ConfigCommand::Show => {
            let config = store.load_or_default()?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommand::Path => {
            println!("{}", store.path().display());
        }
    }
    Ok(())
}

async fn unified_command(args: UnifiedArgs, store: &ConfigStore) -> Result<()> {
    intro_message("unified")?;
    let config = store.load_or_default()?;

    let mode = args.mode.unwrap_or(UnifiedMode::Interactive);
    match mode {
        UnifiedMode::Interactive => {
            run_unified_interactive(args, &config).await?;
        }
        mode => {
            execute_unified_mode(mode, args, &config).await?;
        }
    }

    outro_message("Done interacting with provider.")?;
    Ok(())
}

async fn run_unified_interactive(mut args: UnifiedArgs, config: &Config) -> Result<()> {
    let provider = if let Some(provider) = args.provider {
        provider.into()
    } else {
        prompt_provider(config.unified.default_provider)?
    };

    args.provider = Some(provider.into());
    let mode = prompt_operation()?;
    execute_unified_mode(mode, args, config).await
}

fn prompt_provider(default: ProviderId) -> Result<ProviderId> {
    let prompt = format!(
        "Provider (github, linear, trello, jira) [{}]:",
        default.display_name()
    );
    let selection: String = input(&prompt)
        .placeholder(default.display_name())
        .interact()?;
    let result = if selection.trim().is_empty() {
        default
    } else {
        selection
            .parse()
            .map_err(|err: String| anyhow::anyhow!(err))?
    };
    Ok(result)
}

fn prompt_operation() -> Result<UnifiedMode> {
    let selection: String = input("Operation (list/create) [list]:")
        .placeholder("list")
        .interact()?;
    let normalized = selection.trim().to_lowercase();

    let mode = match normalized.as_str() {
        "" | "list" => UnifiedMode::List,
        "create" => UnifiedMode::Create,
        other => {
            return Err(anyhow::anyhow!(
                "unknown operation '{}'. choose list or create",
                other
            ));
        }
    };
    Ok(mode)
}

async fn execute_unified_mode(mode: UnifiedMode, args: UnifiedArgs, config: &Config) -> Result<()> {
    let provider = args
        .provider
        .map(Into::into)
        .unwrap_or(config.unified.default_provider);
    let token = resolve_token(provider, config)?;
    let client = build_client(provider, token, config)?;

    match mode {
        UnifiedMode::List => {
            let query = UnifiedListQuery {
                workspace: args.workspace,
                repo: args.repo,
                search: args.search,
            };
            let issues = client.list(query).await?;
            display_issues(&issues, args.raw)?;
        }
        UnifiedMode::Create => {
            let title = args
                .title
                .or_else(|| prompt_for_title().ok())
                .unwrap_or_else(|| "Untitled issue".to_string());
            let body = args.body.or_else(|| {
                prompt_for_body()
                    .ok()
                    .filter(|body| !body.trim().is_empty())
            });
            let params = UnifiedCreateParams {
                workspace: args.workspace,
                repo: args.repo,
                title,
                body,
            };
            let issue = client.create(params).await?;
            display_issue(&issue, args.raw);
        }
        UnifiedMode::Interactive => unreachable!(),
    }

    Ok(())
}

fn prompt_for_title() -> Result<String> {
    input("Issue title:").interact()
}

fn prompt_for_body() -> Result<String> {
    input("Issue body (optional):").interact()
}

fn display_issues(issues: &[UnifiedIssue], raw: bool) -> Result<()> {
    if issues.is_empty() {
        println!("No issues returned.");
    } else {
        for issue in issues {
            display_issue(issue, raw);
        }
    }
    Ok(())
}

fn display_issue(issue: &UnifiedIssue, raw: bool) {
    println!("{}", issue.display_summary());
    if raw {
        println!(
            "{}",
            to_string_pretty(&issue.raw_payload).unwrap_or_default()
        );
    }
}

fn resolve_token(provider: ProviderId, config: &Config) -> Result<String> {
    if let Ok(env_token) = env::var(provider.env_var()) {
        if !env_token.trim().is_empty() {
            return Ok(env_token);
        }
    }

    config
        .unified
        .tokens
        .get(&provider)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!(UnifiedError::MissingToken(provider)))
}

fn build_client(
    provider: ProviderId,
    token: String,
    config: &Config,
) -> Result<Box<dyn ProviderClient>, UnifiedError> {
    match provider {
        ProviderId::Github => Ok(Box::new(GitHubClient::new(
            token,
            config.unified.default_repo.clone(),
        ))),
        ProviderId::Linear => Ok(Box::new(LinearClient::new(
            token,
            config.unified.default_workspace.clone(),
        ))),
        ProviderId::Trello => Ok(Box::new(TrelloClient::new())),
        ProviderId::Jira => Ok(Box::new(JiraClient::new())),
    }
}

async fn project_command(args: ProjectArgs, store: &ConfigStore) -> Result<()> {
    intro_message("project management")?;
    let config = store.load_or_default()?;

    let provider = args.provider.map(Into::into).unwrap_or(config.unified.default_provider);
    let token = resolve_token(provider, &config)?;
    let client = build_client(provider, token, &config)?;

    match args.command {
        ProjectSubcommand::Create => {
            create_project(args, &config)?;
        }
        ProjectSubcommand::List => {
            list_projects(args, &config)?;
        }
        ProjectSubcommand::Update => {
            update_project(args, &config)?;
        }
    }

    outro_message("Done with project management.")?;
    Ok(())
}

async fn task_command(args: TaskArgs, store: &ConfigStore) -> Result<()> {
    intro_message("task management")?;
    let config = store.load_or_default()?;

    let provider = args.provider.map(Into::into).unwrap_or(config.unified.default_provider);
    let token = resolve_token(provider, &config)?;
    let client = build_client(provider, token, &config)?;

    match args.command {
        TaskSubcommand::Create => {
            create_task(args, &config)?;
        }
        TaskSubcommand::List => {
            list_tasks(args, &config)?;
        }
        TaskSubcommand::Move => {
            move_task(args, &config)?;
        }
        TaskSubcommand::Update => {
            update_task(args, &config)?;
        }
    }

    outro_message("Done with task management.")?;
    Ok(())
}

fn create_project(args: ProjectArgs, config: &Config) -> Result<()> {
    let project = cli_template_core::unified::UnifiedProject {
        key: args.key.clone().unwrap_or_else(|| args.name.clone().unwrap_or_default()),
        name: args.name.clone().unwrap_or_default(),
        description: args.description.clone().unwrap_or_default(),
        workspace: args.workspace.clone().unwrap_or_else(|| config.unified.default_workspace.clone()),
        repo: args.repo.clone().unwrap_or_else(|| config.unified.default_repo.clone()),
    };

    println!("Creating project: {}\n{}", project.name, project.description);
    println!("Provider: {:?}", args.provider);
    println!("Workspace: {:?}", project.workspace);
    println!("Repo: {:?}", project.repo);
    println!("Key: {:?}", project.key);
    
    Ok(())
}

fn list_projects(args: ProjectArgs, config: &Config) -> Result<()> {
    println!("Listing projects...");
    println!("Provider: {:?}", args.provider);
    println!("Workspace: {:?}", args.workspace);
    println!("Repo: {:?}", args.repo);
    
    Ok(())
}

fn update_project(args: ProjectArgs, config: &Config) -> Result<()> {
    if args.key.is_none() {
        return Err(anyhow::anyhow!("Project key is required for update operation."));
    }

    let project = cli_template_core::unified::UnifiedProject {
        key: args.key.clone().unwrap_or_default(),
        name: args.name.clone().unwrap_or_default(),
        description: args.description.clone().unwrap_or_default(),
        workspace: args.workspace.clone().unwrap_or_else(|| config.unified.default_workspace.clone()),
        repo: args.repo.clone().unwrap_or_else(|| config.unified.default_repo.clone()),
    };

    println!("Updating project: {}", project.key);
    println!("New name: {}", project.name);
    println!("New description: {}", project.description);
    println!("Provider: {:?}", args.provider);
    println!("Workspace: {:?}", project.workspace);
    println!("Repo: {:?}", project.repo);
    
    Ok(())
}

fn create_task(args: TaskArgs, config: &Config) -> Result<()> {
    let task = cli_template_core::unified::UnifiedTask {
        id: args.task_id.clone().unwrap_or_default(),
        title: args.title.clone().unwrap_or_default(),
        description: args.description.clone().unwrap_or_default(),
        status: args.status.clone().unwrap_or_default(),
        project: args.project.clone().unwrap_or_default(),
        workspace: args.workspace.clone().unwrap_or_else(|| config.unified.default_workspace.clone()),
        repo: args.repo.clone().unwrap_or_else(|| config.unified.default_repo.clone()),
    };

    println!("Creating task: {}", task.title);
    println!("Description: {}", task.description);
    println!("Status: {}", task.status);
    println!("Project: {}", task.project);
    println!("Provider: {:?}", args.provider);
    println!("Workspace: {:?}", task.workspace);
    println!("Repo: {:?}", task.repo);
    
    Ok(())
}

fn list_tasks(args: TaskArgs, config: &Config) -> Result<()> {
    println!("Listing tasks...");
    println!("Provider: {:?}", args.provider);
    println!("Workspace: {:?}", args.workspace);
    println!("Repo: {:?}", args.repo);
    println!("Project: {:?}", args.project);
    
    Ok(())
}

fn move_task(args: TaskArgs, config: &Config) -> Result<()> {
    if args.task_id.is_none() {
        return Err(anyhow::anyhow!("Task ID is required for move operation."));
    }
    if args.project.is_none() {
        return Err(anyhow::anyhow!("Target project is required for move operation."));
    }

    println!("Moving task: {}", args.task_id.clone().unwrap_or_default());
    println!("To project: {}", args.project.clone().unwrap_or_default());
    println!("Provider: {:?}", args.provider);
    println!("Workspace: {:?}", args.workspace);
    println!("Repo: {:?}", args.repo);
    
    Ok(())
}

fn update_task(args: TaskArgs, config: &Config) -> Result<()> {
    if args.task_id.is_none() {
        return Err(anyhow::anyhow!("Task ID is required for update operation."));
    }

    let task = cli_template_core::unified::UnifiedTask {
        id: args.task_id.clone().unwrap_or_default(),
        title: args.title.clone().unwrap_or_default(),
        description: args.description.clone().unwrap_or_default(),
        status: args.status.clone().unwrap_or_default(),
        project: args.project.clone().unwrap_or_default(),
        workspace: args.workspace.clone().unwrap_or_else(|| config.unified.default_workspace.clone()),
        repo: args.repo.clone().unwrap_or_else(|| config.unified.default_repo.clone()),
    };

    println!("Updating task: {}", task.id);
    println!("New title: {}", task.title);
    println!("New description: {}", task.description);
    println!("New status: {}", task.status);
    println!("Provider: {:?}", args.provider);
    println!("Workspace: {:?}", task.workspace);
    println!("Repo: {:?}", task.repo);
    
    Ok(())
}

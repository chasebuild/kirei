use std::fmt::Display;

use anyhow::Result;
use cliclack::{confirm, input, intro, note, outro};
use serde_json::to_string_pretty;

use crate::args::{Cli, Command, ConfigCommand, GreetArgs, InitArgs};
use cli_template_core::config::{Config, ConfigStore};
use cli_template_core::greeting;

fn prompt_user_name() -> Result<String> {
    let name: String = input("What name should the CLI template remember?")
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

fn intro_message(section: impl Display) -> Result<()> {
    intro(format!("cli-template {}", section))?;
    Ok(())
}

fn outro_message(message: impl Display) -> Result<()> {
    outro(message)?;
    Ok(())
}

pub fn run(cli: Cli) -> Result<()> {
    let store = ConfigStore::new(
        &cli.config.config_qualifier,
        &cli.config.config_organization,
        &cli.config.config_application,
    )?;

    match cli.command {
        Command::Init(args) => init_command(args, &store),
        Command::Greet(args) => greet_command(args, &store),
        Command::Config(args) => config_command(args.command, &store),
    }
}

fn init_command(args: InitArgs, store: &ConfigStore) -> Result<()> {
    intro_message("init")?;

    let path = store.path();
    if path.exists() && !args.force {
        let overwrite = confirm("A config already exists. Overwrite it?").interact()?;
        if !overwrite {
            outro_message("Initialization canceled.")?;
            return Ok(());
        }
    }

    let user_name = match args.user_name {
        Some(name) => name,
        None => prompt_user_name()?,
    };

    let config = Config { user_name };
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
    println!("{}", greeting(&name));
    Ok(())
}

fn config_command(command: ConfigCommand, store: &ConfigStore) -> Result<()> {
    match command {
        ConfigCommand::Show => {
            let config = store.load_or_default()?;
            println!("{}", to_string_pretty(&config)?);
        }
        ConfigCommand::Path => {
            println!("{}", store.path().display());
        }
    }
    Ok(())
}

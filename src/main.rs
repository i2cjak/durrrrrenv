mod config;
mod executor;
mod parser;

use anyhow::{Context, Result};
use clap::{Parser as ClapParser, Subcommand};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use config::Config;
use executor::Executor;
use parser::Parser;

#[derive(ClapParser)]
#[command(name = "durrrrrenv")]
#[command(about = "A zsh alternative to direnv", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check current directory and output shell script if allowed
    Check {
        /// Directory to check (defaults to current directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    /// Allow the .local_environment file in the current directory
    Allow {
        /// Directory to allow (defaults to current directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    /// Deny/remove permission for the current directory
    Deny {
        /// Directory to deny (defaults to current directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    /// Show status of current directory
    Status {
        /// Directory to check (defaults to current directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    /// Show the path to the zsh hook script
    Hook,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { dir } => check_command(dir),
        Commands::Allow { dir } => allow_command(dir),
        Commands::Deny { dir } => deny_command(dir),
        Commands::Status { dir } => status_command(dir),
        Commands::Hook => hook_command(),
    }
}

fn get_working_dir(dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(d) = dir {
        Ok(d)
    } else {
        env::current_dir().context("Failed to get current directory")
    }
}

fn get_env_file_path(dir: &PathBuf) -> PathBuf {
    dir.join(".local_environment")
}

fn check_command(dir: Option<PathBuf>) -> Result<()> {
    let working_dir = get_working_dir(dir)?;
    let env_file = get_env_file_path(&working_dir);

    // If no .local_environment file exists, do nothing
    if !env_file.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&env_file)
        .context("Failed to read .local_environment file")?;

    let config = Config::load()?;

    if config.is_allowed(&working_dir, &content) {
        // Parse and execute
        let commands = Parser::parse(&content)?;
        let script = Executor::generate_shell_script(&commands, &working_dir)?;
        print!("{}", script);
    } else {
        // Prompt user to allow
        eprintln!("durrrrrenv: .local_environment file found but not allowed");
        eprintln!("durrrrrenv: Run 'eval \"$(durrrrrenv allow)\"' to allow and load it");
        eprintln!("durrrrrenv: File contents:");
        eprintln!("---");
        eprintln!("{}", content);
        eprintln!("---");
    }

    Ok(())
}

fn allow_command(dir: Option<PathBuf>) -> Result<()> {
    let working_dir = get_working_dir(dir)?;
    let env_file = get_env_file_path(&working_dir);

    if !env_file.exists() {
        anyhow::bail!("No .local_environment file found in {}", working_dir.display());
    }

    let content = fs::read_to_string(&env_file)
        .context("Failed to read .local_environment file")?;

    // Show content and ask for confirmation
    eprintln!("Contents of .local_environment:");
    eprintln!("---");
    eprintln!("{}", content);
    eprintln!("---");
    eprint!("Allow this file to be executed? [y/N]: ");
    io::stderr().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        eprintln!("Aborted.");
        return Ok(());
    }

    // Parse to validate
    let commands = Parser::parse(&content)?;

    let mut config = Config::load()?;
    config.allow(&working_dir, &content)?;

    eprintln!("Allowed .local_environment in {}", working_dir.display());

    // Generate and output the shell script to execute immediately
    let script = Executor::generate_shell_script(&commands, &working_dir)?;
    print!("{}", script);

    Ok(())
}

fn deny_command(dir: Option<PathBuf>) -> Result<()> {
    let working_dir = get_working_dir(dir)?;

    let mut config = Config::load()?;
    config.deny(&working_dir)?;

    eprintln!("Denied .local_environment in {}", working_dir.display());

    Ok(())
}

fn status_command(dir: Option<PathBuf>) -> Result<()> {
    let working_dir = get_working_dir(dir)?;
    let env_file = get_env_file_path(&working_dir);

    eprintln!("Directory: {}", working_dir.display());

    if !env_file.exists() {
        eprintln!("Status: No .local_environment file found");
        return Ok(());
    }

    let content = fs::read_to_string(&env_file)
        .context("Failed to read .local_environment file")?;

    let config = Config::load()?;

    if config.is_allowed(&working_dir, &content) {
        eprintln!("Status: Allowed");

        // Show what commands will be executed
        match Parser::parse(&content) {
            Ok(commands) => {
                eprintln!("\nCommands to execute:");
                for cmd in commands {
                    eprintln!("  {:?}", cmd);
                }
            }
            Err(e) => {
                eprintln!("Error parsing: {}", e);
            }
        }
    } else {
        eprintln!("Status: Not allowed or file has changed");
        eprintln!("\nRun 'durrrrrenv allow' to allow execution");
    }

    Ok(())
}

fn hook_command() -> Result<()> {
    // For now, just print the hook script
    let hook_script = include_str!("../hook.zsh");
    print!("{}", hook_script);
    Ok(())
}

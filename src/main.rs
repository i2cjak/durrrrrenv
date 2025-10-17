mod config;
mod executor;
mod parser;

use anyhow::{Context, Result};
use clap::{Parser as ClapParser, Subcommand};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

use config::Config;
use executor::Executor;
use parser::Parser;

/// Maximum number of parent directories to search up
const MAX_SEARCH_DEPTH: usize = 5;

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
        /// Enable verbose output with performance metrics
        #[arg(short, long)]
        verbose: bool,
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
    /// Benchmark performance
    Bench {
        /// Directory to check (defaults to current directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Number of iterations (default: 1000)
        #[arg(short = 'n', long, default_value = "1000")]
        iterations: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { dir, verbose } => check_command(dir, verbose),
        Commands::Allow { dir } => allow_command(dir),
        Commands::Deny { dir } => deny_command(dir),
        Commands::Status { dir } => status_command(dir),
        Commands::Hook => hook_command(),
        Commands::Bench { dir, iterations } => bench_command(dir, iterations),
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

/// Search up the directory tree for a .local_environment file
/// Returns (env_file_path, source_directory, depth) if found
fn find_env_file_in_parents(start_dir: &PathBuf) -> Option<(PathBuf, PathBuf, usize)> {
    let mut current = start_dir.as_path();
    let mut depth = 0;

    while depth < MAX_SEARCH_DEPTH {
        let env_file = current.join(".local_environment");

        // Fast path: check existence without extra allocations
        if env_file.exists() {
            return Some((env_file, current.to_path_buf(), depth));
        }

        // Try to go to parent directory
        match current.parent() {
            Some(parent) => current = parent,
            None => return None, // Reached root
        }

        depth += 1;
    }

    None // Exceeded max search depth
}

fn check_command(dir: Option<PathBuf>, verbose: bool) -> Result<()> {
    let start_time = if verbose { Some(Instant::now()) } else { None };

    let working_dir = get_working_dir(dir)?;

    // Search up the tree for .local_environment file
    let search_start = if verbose { Some(Instant::now()) } else { None };
    let search_result = find_env_file_in_parents(&working_dir);
    let search_duration = search_start.map(|t| t.elapsed());

    if search_result.is_none() {
        if verbose {
            eprintln!("durrrrrenv: No .local_environment file found (searched {} levels)", MAX_SEARCH_DEPTH);
            eprintln!("durrrrrenv: Search time: {:?}", search_duration.unwrap());
        }
        return Ok(());
    }

    let (env_file, source_dir, depth) = search_result.unwrap();

    if verbose {
        eprintln!("durrrrrenv: Found .local_environment at depth {}", depth);
        eprintln!("durrrrrenv: Search time: {:?}", search_duration.unwrap());
    }

    let content = fs::read_to_string(&env_file)
        .context("Failed to read .local_environment file")?;

    let config = Config::load()?;

    if config.is_allowed(&source_dir, &content) {
        // Parse and execute
        let commands = Parser::parse(&content)?;
        let script = Executor::generate_shell_script(&commands, &source_dir)?;

        // Output the source directory first (for the hook to track), then the script
        println!("DURRRRRENV_DIR={}", source_dir.display());
        print!("{}", script);

        if verbose {
            eprintln!("durrrrrenv: Total time: {:?}", start_time.unwrap().elapsed());
        }
    } else {
        // Prompt user to allow
        eprintln!("durrrrrenv: .local_environment file found in {} but not allowed", source_dir.display());
        eprintln!("durrrrrenv: Run 'cd {} && eval \"$(durrrrrenv allow)\"' to allow and load it", source_dir.display());
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

    // Output the source directory first (for the hook to track), then the script
    println!("DURRRRRENV_DIR={}", working_dir.display());
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

fn bench_command(dir: Option<PathBuf>, iterations: usize) -> Result<()> {
    let working_dir = get_working_dir(dir)?;

    eprintln!("Benchmarking durrrrrenv search performance...");
    eprintln!("Directory: {}", working_dir.display());
    eprintln!("Iterations: {}", iterations);
    eprintln!("Max search depth: {} levels", MAX_SEARCH_DEPTH);
    eprintln!();

    // Warm-up run
    let _ = find_env_file_in_parents(&working_dir);

    // Actual benchmark
    let start = Instant::now();
    let mut found_count = 0;
    let mut total_depth = 0;

    for _ in 0..iterations {
        if let Some((_env_file, _source_dir, depth)) = find_env_file_in_parents(&working_dir) {
            found_count += 1;
            total_depth += depth;
        }
    }

    let duration = start.elapsed();
    let avg_time = duration / iterations as u32;

    eprintln!("Results:");
    eprintln!("--------");
    eprintln!("Total time: {:?}", duration);
    eprintln!("Average time per search: {:?}", avg_time);
    eprintln!("Searches per second: {:.0}", iterations as f64 / duration.as_secs_f64());
    eprintln!();

    if found_count > 0 {
        eprintln!("Found .local_environment: {} times", found_count);
        eprintln!("Average depth: {:.2}", total_depth as f64 / found_count as f64);
    } else {
        eprintln!("No .local_environment file found in any iteration");
    }

    Ok(())
}

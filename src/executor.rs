use crate::parser::Command;
use anyhow::{Context, Result};
use std::path::Path;

pub struct Executor;

impl Executor {
    /// Generate shell script from parsed commands
    pub fn generate_shell_script(commands: &[Command], working_dir: &Path) -> Result<String> {
        let mut script = String::new();

        for cmd in commands {
            let line = Self::command_to_shell(cmd, working_dir)?;
            script.push_str(&line);
            script.push('\n');
        }

        Ok(script)
    }

    /// Convert a Command to a shell script line
    fn command_to_shell(cmd: &Command, working_dir: &Path) -> Result<String> {
        match cmd {
            Command::Source { path } => {
                let resolved_path = Self::resolve_path(path, working_dir)?;
                Ok(format!("source '{}'", resolved_path.display()))
            }
            Command::PythonVenv { path } => {
                let resolved_path = Self::resolve_path(path, working_dir)?;
                let activate_script = resolved_path.join("bin").join("activate");

                if !activate_script.exists() {
                    anyhow::bail!(
                        "Python venv activate script not found: {}",
                        activate_script.display()
                    );
                }

                Ok(format!("source '{}'", activate_script.display()))
            }
            Command::ProcessSubstitution { command } => {
                // For process substitution, we need to execute the command and verify it works
                // The actual substitution happens in the shell
                Ok(format!("source <({})", command))
            }
        }
    }

    /// Resolve a path relative to the working directory
    fn resolve_path(path: &str, working_dir: &Path) -> Result<std::path::PathBuf> {
        // Handle tilde expansion
        let expanded = if path.starts_with("~/") {
            let home = dirs::home_dir()
                .context("Failed to determine home directory")?;
            home.join(&path[2..])
        } else if path.starts_with('~') {
            // Handle ~username - for now just return as-is and let the shell handle it
            return Ok(std::path::PathBuf::from(path));
        } else if path.starts_with('/') {
            // Absolute path
            std::path::PathBuf::from(path)
        } else {
            // Relative path
            working_dir.join(path)
        };

        Ok(expanded)
    }

}

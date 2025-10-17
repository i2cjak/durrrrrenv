use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Source a file: source <file>
    Source { path: String },
    /// Activate Python venv: python_venv [path]
    PythonVenv { path: String },
    /// Process substitution: source <(command)
    ProcessSubstitution { command: String },
}

pub struct Parser;

impl Parser {
    /// Parse a .local_environment file
    pub fn parse(content: &str) -> Result<Vec<Command>> {
        let mut commands = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let cmd = Self::parse_line(line)
                .with_context(|| format!("Failed to parse line {}: {}", line_num + 1, line))?;

            commands.push(cmd);
        }

        Ok(commands)
    }

    /// Parse a single line
    fn parse_line(line: &str) -> Result<Command> {
        // Check for process substitution: source <(...)
        if line.starts_with("source") && line.contains("<(") && line.contains(')') {
            return Self::parse_process_substitution(line);
        }

        // Check for regular source command
        if line.starts_with("source") {
            return Self::parse_source(line);
        }

        // Check for python_venv
        if line.starts_with("python_venv") {
            return Self::parse_python_venv(line);
        }

        anyhow::bail!("Unknown command: {}", line)
    }

    /// Parse: source <file>
    fn parse_source(line: &str) -> Result<Command> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 2 {
            anyhow::bail!("source command expects exactly one argument");
        }

        Ok(Command::Source {
            path: parts[1].to_string(),
        })
    }

    /// Parse: python_venv [path]
    fn parse_python_venv(line: &str) -> Result<Command> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let path = if parts.len() == 1 {
            ".venv".to_string()
        } else if parts.len() == 2 {
            parts[1].to_string()
        } else {
            anyhow::bail!("python_venv command expects zero or one argument");
        };

        Ok(Command::PythonVenv { path })
    }

    /// Parse: source <(command args...)
    fn parse_process_substitution(line: &str) -> Result<Command> {
        // Find the positions of <( and )
        let start = line.find("<(")
            .context("Expected '<(' in process substitution")?;
        let end = line.rfind(')')
            .context("Expected ')' in process substitution")?;

        if end <= start + 2 {
            anyhow::bail!("Empty process substitution command");
        }

        let command = &line[start + 2..end];

        Ok(Command::ProcessSubstitution {
            command: command.trim().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_source() {
        let cmd = Parser::parse_line("source ~/.bashrc").unwrap();
        assert_eq!(
            cmd,
            Command::Source {
                path: "~/.bashrc".to_string()
            }
        );
    }

    #[test]
    fn test_parse_python_venv_default() {
        let cmd = Parser::parse_line("python_venv").unwrap();
        assert_eq!(
            cmd,
            Command::PythonVenv {
                path: ".venv".to_string()
            }
        );
    }

    #[test]
    fn test_parse_python_venv_custom() {
        let cmd = Parser::parse_line("python_venv venv").unwrap();
        assert_eq!(
            cmd,
            Command::PythonVenv {
                path: "venv".to_string()
            }
        );
    }

    #[test]
    fn test_parse_process_substitution() {
        let cmd = Parser::parse_line("source <(west completion zsh)").unwrap();
        assert_eq!(
            cmd,
            Command::ProcessSubstitution {
                command: "west completion zsh".to_string()
            }
        );
    }

    #[test]
    fn test_parse_multi_line() {
        let content = r#"
# This is a comment
source ~/.bashrc

python_venv .venv
source <(west completion zsh)
        "#;

        let commands = Parser::parse(content).unwrap();
        assert_eq!(commands.len(), 3);
    }
}

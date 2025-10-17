use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    /// Map of directory hash -> allowed status and metadata
    pub allowed_dirs: HashMap<String, DirInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirInfo {
    /// Full canonical path to the directory
    pub path: PathBuf,
    /// Hash of the .local_environment file content when it was allowed
    pub file_hash: String,
    /// Timestamp when it was allowed
    pub allowed_at: u64,
}

impl Config {
    /// Get the path to the config file
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join("durrrrrenv");

        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(config_dir.join("allowed.json"))
    }

    /// Load config from disk, or create a new one if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let config: Config = serde_json::from_str(&contents)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, contents)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Check if a directory is allowed and if the file hasn't changed
    pub fn is_allowed(&self, dir: &Path, file_content: &str) -> bool {
        let dir_key = Self::hash_path(dir);

        if let Some(info) = self.allowed_dirs.get(&dir_key) {
            let current_hash = Self::hash_content(file_content);
            return info.file_hash == current_hash;
        }

        false
    }

    /// Add a directory to the allowed list
    pub fn allow(&mut self, dir: &Path, file_content: &str) -> Result<()> {
        let dir_key = Self::hash_path(dir);
        let file_hash = Self::hash_content(file_content);
        let canonical_path = fs::canonicalize(dir)
            .context("Failed to canonicalize directory path")?;

        let info = DirInfo {
            path: canonical_path,
            file_hash,
            allowed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.allowed_dirs.insert(dir_key, info);
        self.save()?;

        Ok(())
    }

    /// Remove a directory from the allowed list
    pub fn deny(&mut self, dir: &Path) -> Result<()> {
        let dir_key = Self::hash_path(dir);
        self.allowed_dirs.remove(&dir_key);
        self.save()?;

        Ok(())
    }

    /// Hash a directory path for use as a key
    fn hash_path(path: &Path) -> String {
        let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let mut hasher = Sha256::new();
        hasher.update(canonical.to_string_lossy().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Hash file content
    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

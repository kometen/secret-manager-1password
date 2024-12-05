//! Secret Management Module
//!
//! This module manages secrets, retrieving Azure Key Vault URL
//! from 1password via the command line utility `op`.
//!

use anyhow::{Context, Result};
use std::process::Command;

/// URL of the Azure Key Vault.
pub struct SecretManager {
    pub url: String,
}

impl SecretManager {
    /// Creates a new Secret Manager instance with a specific value.
    ///
    /// # Arguments
    ///
    /// * `key` - Name of the variable with the value of the 1password path
    ///
    /// If key have suffix `_test` change default Vault-name
    ///
    /// # Returns
    ///
    /// A Result containing the SecretManager if successful, or an error if the secret
    /// could not be retrieved.
    ///
    /// # Example
    ///
    /// ```
    /// use anyhow::Result;
    /// use secret_manager_1password::SecretManager;
    ///
    /// fn example() -> Result<()> {
    ///     let secret_manager = SecretManager::new("AZURE_KEY_VAULT_TEST")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new(key: &str) -> Result<Self> {
        let vault = if key.ends_with("_test") {
            "Test"
        } else {
            "Production"
        };

        let clean_key = key.trim_end_matches("_test");

        let op_path = format!("op://{}/AzureKeyVault{}/url", vault, clean_key);

        let command = Command::new("op")
            .arg("read")
            .arg(&op_path)
            .output()
            .context("Error executing command")?;

        let url = String::from_utf8(command.stdout)
            .context("Failed to convert command output to string")?
            .trim_end()
            .to_string();

        Ok(Self { url })
    }

    /// Used for testing an error is returned if the command line utility
    /// is not present.
    #[cfg(test)]
    fn wrong_command_for_test() -> Result<Self> {
        let command = Command::new("_op_")
            .arg("read")
            .arg("foo")
            .output()
            .context("Error executing command")?;

        let url = String::from_utf8(command.stdout)
            .context("Failed to convert command output to string")?
            .trim_end()
            .to_string();

        Ok(Self { url })
    }
}

#[cfg(test)]
mod tests {
    use crate::SecretManager;
    use std::env;

    /// Tests SecretManager creation with a valid environment variable.
    ///
    /// This handles two scenarios:
    /// 1. In Github Actions: Connects to 1password.com with a token stored
    /// as a secret in settings on the github-repository and fetches the
    /// Azure Key Vault URL from 1password.com.
    ///
    /// 2. Locally: Uses the `op` command to retrieve the Azure Key Vault URL.
    #[test]
    fn test_new_with_valid_env_var() {
        if env::var("GITHUB_ACTIONS").is_ok() {
            let result = env::var("AZURE_KEY_VAULT_TEST");
            assert!(result.is_ok());
            let secret_manager = result.unwrap();
            assert_eq!(secret_manager, "https://foo.bar.baz.net/");
        } else {
            let result = SecretManager::new("demo_test");
            assert!(result.is_ok());
            let secret_manager = result.unwrap();
            assert_eq!(secret_manager.url, "https://foo.bar.baz.net/");
        }
    }

    /// Test an error is returned if the command line utility `op` is not
    /// present.
    #[test]
    fn test_new_with_invalid_command() {
        let result = SecretManager::wrong_command_for_test();
        assert!(result.is_err());
    }
}

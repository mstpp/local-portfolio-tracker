use crate::Cli;
use anyhow::{Context, Result};
use config::Config;
use serde::{Deserialize, Serialize};
use shellexpand::tilde;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_portfolio_dir")]
    pub portfolio_dir: PathBuf,

    // TODO add new type (currency that can be base)
    // for now, validate it's in small set (USD,BTC)
    #[serde(default = "default_base_currency")]
    pub base_currency: String,
}

fn default_portfolio_dir() -> PathBuf {
    PathBuf::from("./portfolios")
}
fn default_base_currency() -> String {
    "USD".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            portfolio_dir: default_portfolio_dir(),
            base_currency: default_base_currency(),
        }
    }
}

impl Settings {
    /// Load configuration with proper priority:
    /// defaults → dotfile → env → CLI
    pub fn load(cli: &Cli) -> Result<Self> {
        let mut builder = Config::builder();

        // Layer 1: Built-in defaults (via serde defaults)

        // Layer 2: Dotfile (optional, won't fail if missing)
        let dotfile_path = tilde("~/.local/share/csvpt/config.toml").to_string();
        if std::fs::exists(&dotfile_path).unwrap_or(false) {
            println!("Loading config from: {}", dotfile_path);
            builder = builder.add_source(config::File::with_name(&dotfile_path).required(false));
        } else {
            println!("No config file found at {}, using defaults", dotfile_path);
        }

        // Layer 3: Environment variables (LPT_PORTFOLIO_DIR, LPT_BASE_CURRENCY, etc.)
        // LPT = Local Portfolio Tracker
        builder = builder.add_source(
            config::Environment::with_prefix("LPT")
                .prefix_separator("_")
                .try_parsing(true),
        );

        // Layer 4: CLI arguments (highest priority)
        if let Some(portfolio_dir) = &cli.portfolio_dir {
            println!("CLI orverride for portfolio dir: {portfolio_dir}");
            builder = builder.set_override("portfolio_dir", portfolio_dir.to_string())?;
        }

        // Build and deserialize
        let config = builder.build()?;
        let mut settings: Settings = config
            .try_deserialize()
            .with_context(|| "Failed to deserialize configuration")?;

        // Validate and show warnings
        let warnings = settings.validate();
        for warning in warnings {
            eprintln!("⚠️  Config warning: {}", warning);
        }

        println!("DEBUG: builder: {:?}", &settings);
        Ok(settings)
    }

    /// Validate settings and return warnings for invalid values
    fn validate(&mut self) -> Vec<String> {
        // TODO handle through type
        let allowed_base_currency = std::collections::HashSet::from(["USD", "BTC"]);

        let mut warnings = Vec::new();

        // Validate base_currency (should be 3-letter code)
        if !allowed_base_currency.contains(self.base_currency.as_str()) {
            warnings.push(format!(
                "Invalid base_currency '{}', using default '{}'",
                self.base_currency,
                default_base_currency()
            ));
            self.base_currency = default_base_currency();
        }

        // Validate data_dir (attempt to create if doesn't exist)
        if let Err(e) = std::fs::create_dir_all(&self.portfolio_dir) {
            warnings.push(format!(
                "Cannot create data_dir '{}': {}, using default '{}'",
                self.portfolio_dir.display(),
                e,
                default_portfolio_dir().display()
            ));
            self.portfolio_dir = default_portfolio_dir();
        }

        warnings
    }
}

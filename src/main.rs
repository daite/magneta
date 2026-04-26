use clap::{Parser, Subcommand};
use config::{Config, Value};
use indicatif::{ProgressBar, ProgressStyle};
use log::warn;
use magneta::http::{build_client, validate_base_url};
use prettytable::{format, row, Table};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::time::Duration;

mod site_registry;

/// CLI argument parsing
#[derive(Parser)]
#[command(name = "magneta")]
#[command(version = "0.1.0")]
#[command(about = "Multi-site torrent search CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for a torrent using a keyword
    Search {
        /// The keyword to search for
        keyword: String,
    },
    /// Run diagnostics on torrent sites
    Doctor,
}

/// Load configuration from config.toml located next to the binary
fn get_config() -> anyhow::Result<Config> {
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let config_path = exe_dir.join("config.toml");

    Config::builder()
        .add_source(config::File::from(config_path))
        .build()
        .map_err(Into::into)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    // Load configuration
    let settings = get_config()?;

    match &cli.command {
        Commands::Search { keyword } => {
            let sites = site_registry::create_sites(&settings);

            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            pb.enable_steady_tick(Duration::from_millis(100));
            pb.set_message("Searching torrents...");

            let mut results = Vec::new();
            for (site_name, site) in sites {
                match site.search(keyword).await {
                    Ok(r) if !r.is_empty() => {
                        results = r;
                        break;
                    }
                    Ok(_) => continue,
                    Err(e) => {
                        let message = if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>()
                        {
                            if reqwest_err.is_connect() {
                                "❌ connection failed".to_string()
                            } else if reqwest_err.is_timeout() {
                                "⏱️ timeout".to_string()
                            } else {
                                let s = reqwest_err.to_string();
                                s.split(':')
                                    .next()
                                    .unwrap_or("❌ unknown error")
                                    .to_string()
                            }
                        } else {
                            let s = e.to_string();
                            s.split(':')
                                .next()
                                .unwrap_or("❌ unknown error")
                                .to_string()
                        };

                        warn!("Error searching site '{}': {}", site_name, message);
                        continue;
                    }
                }
            }

            pb.finish_with_message("✅ Search completed!");

            if results.is_empty() {
                println!("No results found for keyword: {}", keyword);
                return Ok(());
            }

            let mut table = Table::new();
            table.set_titles(row!["Title", "Magnet"]);
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

            for r in results {
                if !r.title.trim().is_empty() && !r.magnet.trim().is_empty() {
                    table.add_row(row![r.title, r.magnet]);
                }
            }

            table.printstd();
        }
        Commands::Doctor => {
            let config_map: HashMap<String, Value> = settings.try_deserialize()?;

            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            pb.enable_steady_tick(Duration::from_millis(100));
            pb.set_message("Running diagnostics for torrent sites...");

            let mut table = Table::new();
            table.set_titles(row!["Site Name", "Site URL", "Status"]);
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

            let client = build_client()?;

            for (site_name, site_config) in config_map {
                if let Ok(table_val) = site_config.into_table() {
                    if let Some(url) = table_val
                        .get("base_url")
                        .and_then(|v| v.clone().into_string().ok())
                    {
                        let status = match validate_base_url(&url) {
                            Ok(url) => match client.get(url).send().await {
                                Ok(resp) if resp.status().is_success() => {
                                    format!("✅ {}", resp.status())
                                }
                                Ok(resp) => format!("⚠️ {}", resp.status()),
                                Err(err) => {
                                    if err.is_connect() {
                                        "❌ connection failed".to_string()
                                    } else if err.is_timeout() {
                                        "⏱️ timeout".to_string()
                                    } else {
                                        format!(
                                            "❌ {}",
                                            err.to_string()
                                                .split(':')
                                                .next()
                                                .unwrap_or("unknown error")
                                        )
                                    }
                                }
                            },
                            Err(err) => format!("❌ {err}"),
                        };
                        table.add_row(row![site_name, url, status]);
                    } else {
                        table.add_row(row![site_name, "no base_url", "❌ Invalid"]);
                    }
                }
            }

            pb.finish_with_message("✅ Diagnostics completed!");
            table.printstd();
        }
    }

    Ok(())
}

//! bizkit — business-trend crawler CLI: EDGAR M&A signals, idea
//! taxonomy reports.

use anyhow::{Context, Result};
use bizkit_core::edgar::{EdgarClient, FtsQuery};
use bizkit_core::taxonomy::{catalog, InvestmentLevel, RiskLevel};
use bizkit_core::Store;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Business-trend crawler: EDGAR M&A signals, idea taxonomy reports.
#[derive(Debug, Parser)]
#[command(name = "bizkit", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Crawl EDGAR full-text search for filing signals.
    Crawl {
        /// Free-text query (e.g. "acquisition of", "merger agreement").
        #[arg(long, default_value = "acquisition")]
        query: String,
        /// Form filter (8-K, S-1, DEFM14A, SC 13D).
        #[arg(long, default_value = "8-K")]
        forms: String,
        /// Inclusive start date YYYY-MM-DD.
        #[arg(long, default_value = "2026-10-17")]
        startdt: String,
        /// Inclusive end date YYYY-MM-DD.
        #[arg(long, default_value = "2026-10-24")]
        enddt: String,
        /// Max hits per crawl.
        #[arg(long, default_value_t = 100)]
        limit: usize,
        /// SQLite store path.
        #[arg(long, default_value = "bizkit.db")]
        db: PathBuf,
    },
    /// Report filings by form type and top repeat filers.
    Report {
        /// SQLite store path.
        #[arg(long, default_value = "bizkit.db")]
        db: PathBuf,
        /// Top filers to show.
        #[arg(long, default_value_t = 10)]
        top: i64,
    },
    /// Print the investment-by-risk idea taxonomy.
    Ideas {
        /// Filter by investment level (near-zero, low, high).
        #[arg(long)]
        investment: Option<String>,
        /// Filter by risk level (low, medium, high).
        #[arg(long)]
        risk: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Crawl {
            query,
            forms,
            startdt,
            enddt,
            limit,
            db,
        } => {
            let client = EdgarClient::new();
            let query = FtsQuery {
                q: query,
                forms: Some(forms),
                startdt,
                enddt,
            };
            let hits = client
                .search(&query, limit)
                .await
                .context("edgar search failed")?;
            println!("fetched {} filing hits", hits.len());
            let store = Store::open(&db).context("open store")?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let stored = store
                .record_filings(&hits, &query.q, now)
                .context("record filings failed")?;
            println!("stored {stored} filings into {db:?}");
        }
        Command::Report { db, top } => {
            let store = Store::open(&db).context("open store")?;
            println!("=== filings by form ===");
            for (form, hits, first, last) in store.form_summary()? {
                println!("{form:<10} {hits:>5} filings  (seen {first}..{last})");
            }
            println!("\n=== top repeat filers (acquisition signal) ===");
            for (name, form, count) in store.top_filers(top)? {
                println!("{count:>4}x [{form}] {name}");
            }
        }
        Command::Ideas { investment, risk } => {
            let inv = investment.as_deref().and_then(InvestmentLevel::parse);
            let risk = risk.as_deref().and_then(RiskLevel::parse);
            for idea in catalog() {
                if let Some(inv) = &inv {
                    if idea.investment != *inv {
                        continue;
                    }
                }
                if let Some(risk) = &risk {
                    if idea.risk != *risk {
                        continue;
                    }
                }
                println!(
                    "{} [{}] [{} risk] — {}",
                    idea.name,
                    idea.sector,
                    idea.cell(),
                    idea.summary
                );
                println!("  signals: {}", idea.signals.join(", "));
                println!("  primary risk: {}", idea.primary_risk);
            }
        }
    }
    Ok(())
}

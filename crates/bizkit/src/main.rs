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
    /// Run the curated daily M&A watch: fixed queries over a rolling
    /// window, recorded into the store (deduped on filing id).
    Watch {
        /// Rolling window in days back from today.
        #[arg(long, default_value_t = 14)]
        window: i64,
        /// Max hits per query.
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
        Command::Watch { window, limit, db } => {
            // The curated M&A watchlist: phrase plus form filter, one pass
            // per day from the fleet timer. Three queries stays inside the
            // SEC fair-access policy.
            const WATCHLIST: [(&str, &str); 3] = [
                ("\"merger agreement\"", "8-K,DEFM14A,425,S-4"),
                ("\"acquisition of\"", "8-K,DEFM14A,425,S-4,SC 13D,SC 13G"),
                ("\"tender offer\"", "SC TO-T,SC 14D9,425,8-K"),
            ];
            let client = EdgarClient::new();
            let store = Store::open(&db).context("open store")?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let enddt = iso_days_from_epoch(now);
            let startdt = iso_days_from_epoch(now - window * 86_400);
            let mut total_stored = 0usize;
            for (phrase, forms) in WATCHLIST {
                let query = FtsQuery {
                    q: phrase.to_string(),
                    forms: Some(forms.to_string()),
                    startdt: startdt.clone(),
                    enddt: enddt.clone(),
                };
                let hits = client
                    .search(&query, limit)
                    .await
                    .with_context(|| format!("edgar search failed: {phrase}"))?;
                let stored = store.record_filings(&hits, phrase, now)?;
                total_stored += stored;
                println!(
                    "{phrase}: {} hits, {stored} new -> {}",
                    hits.len(),
                    db.display()
                );
                tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            }
            println!("watch complete: {total_stored} new filings, window {startdt}..{enddt}");
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

/// Unix seconds as `YYYY-MM-DD` (UTC). Hinnant's civil_from_days; no
/// date crate pulled in for one string.
fn iso_days_from_epoch(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

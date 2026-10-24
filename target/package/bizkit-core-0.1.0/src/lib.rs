//! bizkit-core — EDGAR filing signals, the idea taxonomy, and sector
//! trend scoring for bizkit.

pub mod edgar;
pub mod error;
pub mod store;
pub mod taxonomy;

pub use edgar::{parse_fts_json, EdgarClient, FilingHit, FtsQuery};
pub use error::{BizError, Result};
pub use store::Store;
pub use taxonomy::{catalog, BusinessIdea, InvestmentLevel, RiskLevel};

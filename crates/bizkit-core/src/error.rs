//! Errors surfaced by bizkit-core.

/// Fallible result alias used throughout bizkit-core.
pub type Result<T> = std::result::Result<T, BizError>;

/// Errors surfaced by bizkit-core.
#[derive(Debug, thiserror::Error)]
pub enum BizError {
    /// Transport failure (network, timeout, non-2xx from fetch-kit).
    #[error("fetch error: {0}")]
    Fetch(#[from] fetch_kit::FetchError),

    /// Response body was not the JSON shape we expected.
    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),

    /// SQLite persistence failure.
    #[error("store error: {0}")]
    Store(#[from] rusqlite::Error),

    /// EDGAR-specific failure (parse or policy).
    #[error("edgar error: {0}")]
    Edgar(String),

    /// Anything else.
    #[error("{0}")]
    Other(String),
}

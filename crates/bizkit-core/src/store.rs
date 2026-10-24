//! SQLite persistence for EDGAR filing observations.

use rusqlite::Connection;

use crate::edgar::FilingHit;
use crate::error::{BizError, Result};

/// A handle to the bizkit SQLite database.
#[derive(Debug)]
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open (and migrate) the database at `path`.
    ///
    /// # Errors
    /// Fails when the file cannot be opened or the schema cannot
    /// migrate.
    pub fn open(path: &std::path::Path) -> Result<Store> {
        let conn = Connection::open(path).map_err(BizError::from)?;
        let store = Store { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Open an in-memory database (tests).
    ///
    /// # Errors
    /// Fails when the in-memory database cannot be created.
    pub fn open_in_memory() -> Result<Store> {
        let conn = Connection::open_in_memory().map_err(BizError::from)?;
        let store = Store { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS filings (
                    doc          TEXT PRIMARY KEY,
                    cik          TEXT NOT NULL,
                    display_name TEXT NOT NULL,
                    form         TEXT NOT NULL,
                    period_ending TEXT,
                    score        REAL NOT NULL,
                    query        TEXT NOT NULL,
                    first_seen   INTEGER NOT NULL,
                    last_seen    INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_filings_form ON filings(form);
                CREATE INDEX IF NOT EXISTS idx_filings_query ON filings(query);
                ",
            )
            .map_err(BizError::from)?;
        Ok(())
    }

    /// Upsert filing hits from one crawl, deduplicating by document id
    /// and tracking first/last seen.
    ///
    /// # Errors
    /// Propagates SQLite failures.
    pub fn record_filings(&self, hits: &[FilingHit], query: &str, now: i64) -> Result<usize> {
        let mut stored = 0;
        for hit in hits {
            self.conn
                .execute(
                    "
                    INSERT INTO filings
                        (doc, cik, display_name, form, period_ending, score,
                         query, first_seen, last_seen)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
                    ON CONFLICT(doc) DO UPDATE SET
                        score = excluded.score,
                        last_seen = excluded.last_seen
                    ",
                    rusqlite::params![
                        hit.doc,
                        hit.cik,
                        hit.display_name,
                        hit.form,
                        hit.period_ending,
                        hit.score,
                        query,
                        now,
                    ],
                )
                .map_err(BizError::from)?;
            stored += 1;
        }
        Ok(stored)
    }

    /// Filings grouped by form type with first/last seen and hit counts,
    /// for the M&A velocity report.
    ///
    /// # Errors
    /// Propagates SQLite failures.
    pub fn form_summary(&self) -> Result<Vec<(String, i64, i64, i64)>> {
        let mut stmt = self
            .conn
            .prepare(
                "
                SELECT form, COUNT(*) AS hits, MIN(first_seen), MAX(last_seen)
                FROM filings GROUP BY form ORDER BY hits DESC
                ",
            )
            .map_err(BizError::from)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(BizError::from)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(BizError::from)?);
        }
        Ok(out)
    }

    /// Top filers by appearance count — repeat filers are the acquisition
    /// signal (a company filing multiple 8-Ks mentioning acquisitions is
    /// actively rolling up).
    ///
    /// # Errors
    /// Propagates SQLite failures.
    pub fn top_filers(&self, limit: i64) -> Result<Vec<(String, String, i64)>> {
        let mut stmt = self
            .conn
            .prepare(
                "
                SELECT display_name, form, COUNT(*) AS c
                FROM filings
                GROUP BY cik, form
                ORDER BY c DESC
                LIMIT ?1
                ",
            )
            .map_err(BizError::from)?;
        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(BizError::from)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(BizError::from)?);
        }
        Ok(out)
    }
}

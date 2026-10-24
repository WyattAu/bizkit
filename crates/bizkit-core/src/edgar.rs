//! EDGAR full-text search client — the free, authoritative M&A and
//! funding-signal source.

use fetch_kit::Client;
use serde::Deserialize;

use crate::error::{BizError, Result};

const FTS_URL: &str = "https://efts.sec.gov/LATEST/search-index";
/// SEC fair-access policy asks research clients to declare contact info
/// in the user agent.
const USER_AGENT: &str = "bizkit/0.1 research contact@wyattau.dev";

/// One filing hit from EDGAR full-text search.
#[derive(Debug, Clone, PartialEq)]
pub struct FilingHit {
    /// CIK of the filer (zero-padded string).
    pub cik: String,
    /// Human display name from the filing index.
    pub display_name: String,
    /// Form type: `8-K` (material event), `S-1` (IPO), `DEFM14A`
    /// (merger proxy), `SC 13D` (stake acquisition), and so on.
    pub form: String,
    /// Filing period end date, `YYYY-MM-DD`.
    pub period_ending: String,
    /// Document id (`accession:file`).
    pub doc: String,
    /// Relevance score from EDGAR.
    pub score: f64,
}

#[derive(Debug, Deserialize)]
struct FtsResponse {
    #[serde(default)]
    hits: FtsHits,
}

#[derive(Debug, Deserialize, Default)]
struct FtsHits {
    #[serde(default)]
    hits: Vec<FtsHit>,
}

#[derive(Debug, Deserialize)]
struct FtsHit {
    #[serde(rename = "_source", default)]
    source: FtsSource,
    #[serde(rename = "_score", default)]
    score: f64,
    #[serde(rename = "_id", default)]
    id: String,
}

#[derive(Debug, Deserialize, Default)]
struct FtsSource {
    #[serde(default, rename = "ciks")]
    ciks: Vec<String>,
    #[serde(default, rename = "display_names")]
    display_names: Vec<String>,
    #[serde(default, rename = "file_type")]
    file_type: Option<String>,
    #[serde(default, rename = "period_ending")]
    period_ending: Option<String>,
}

/// Query parameters for EDGAR full-text search.
#[derive(Debug, Clone)]
pub struct FtsQuery {
    /// Free-text query (quoted phrases and field filters supported).
    pub q: String,
    /// Form-type filter, e.g. `8-K`, `S-1`, `DEFM14A`.
    pub forms: Option<String>,
    /// Inclusive start date `YYYY-MM-DD`.
    pub startdt: String,
    /// Inclusive end date `YYYY-MM-DD`.
    pub enddt: String,
}

/// Client for the EDGAR full-text search endpoint.
#[derive(Debug, Clone)]
pub struct EdgarClient {
    http: Client,
}

impl Default for EdgarClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgarClient {
    /// New client. The SEC fair-access policy asks research clients to
    /// declare contact info in the user agent — this one does.
    #[must_use]
    pub fn new() -> EdgarClient {
        EdgarClient {
            http: Client::builder().user_agent(USER_AGENT).build(),
        }
    }

    /// Run a full-text search, returning up to `limit` filing hits.
    ///
    /// # Errors
    /// Fails on transport errors and unparseable payloads.
    pub async fn search(&self, query: &FtsQuery, limit: usize) -> Result<Vec<FilingHit>> {
        let url = format!(
            "{FTS_URL}?q={}&forms={}&dateRange=custom&startdt={}&enddt={}",
            percent_encode(&query.q),
            percent_encode(query.forms.as_deref().unwrap_or("")),
            percent_encode(&query.startdt),
            percent_encode(&query.enddt),
        );
        let response: FtsResponse = self.http.get(&url).json_response().await?;
        let hits = response
            .hits
            .hits
            .into_iter()
            .take(limit)
            .map(|h| FilingHit {
                cik: h.source.ciks.first().cloned().unwrap_or_default(),
                display_name: h.source.display_names.first().cloned().unwrap_or_default(),
                form: h.source.file_type.unwrap_or_default(),
                period_ending: h.source.period_ending.unwrap_or_default(),
                doc: h.id,
                score: h.score,
            })
            .collect();
        Ok(hits)
    }
}

fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Parse an EDGAR FTS JSON body into hits (exposed for tests).
///
/// # Errors
/// Fails when the payload is not the FTS shape.
pub fn parse_fts_json(body: &str) -> Result<Vec<FilingHit>> {
    let value: FtsResponse =
        serde_json::from_str(body).map_err(|e| BizError::Edgar(format!("fts parse: {e}")))?;
    Ok(value
        .hits
        .hits
        .into_iter()
        .map(|h| FilingHit {
            cik: h.source.ciks.first().cloned().unwrap_or_default(),
            display_name: h.source.display_names.first().cloned().unwrap_or_default(),
            form: h.source.file_type.unwrap_or_default(),
            period_ending: h.source.period_ending.unwrap_or_default(),
            doc: h.id,
            score: h.score,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    const SAMPLE: &str = r#"{
        "hits": {
            "total": { "value": 2 },
            "hits": [
                {
                    "_id": "0001213900-26-112185:doc1.htm",
                    "_score": 3.8,
                    "_source": {
                        "ciks": ["0001998768"],
                        "display_names": ["Proficient Auto Logistics, Inc"],
                        "file_type": "8-K",
                        "period_ending": "2026-08-13"
                    }
                },
                {
                    "_id": "bare",
                    "_source": {}
                }
            ]
        }
    }"#;

    #[test]
    fn parse_fts_maps_sources() {
        let hits = parse_fts_json(SAMPLE).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].cik, "0001998768");
        assert_eq!(hits[0].display_name, "Proficient Auto Logistics, Inc");
        assert_eq!(hits[0].form, "8-K");
        assert_eq!(hits[0].period_ending, "2026-08-13");
        assert!((hits[0].score - 3.8).abs() < 1e-9);
        // Degenerate hit: all defaults.
        assert_eq!(hits[1].cik, "");
        assert_eq!(hits[1].form, "");
    }

    #[test]
    fn parse_fts_rejects_garbage() {
        assert!(parse_fts_json("not json").is_err());
    }

    #[test]
    fn percent_encode_matches_url_conventions() {
        assert_eq!(percent_encode("merger agreement"), "merger+agreement");
        assert_eq!(percent_encode("acquisition"), "acquisition");
        assert_eq!(percent_encode("a&b"), "a%26b");
    }
}

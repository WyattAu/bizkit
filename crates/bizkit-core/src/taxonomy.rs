//! The investment-by-risk idea taxonomy.
//!
//! Every idea cites the measured trend subjects that justify it — the
//! crawler is the evidence, the taxonomy is the decision layer. Levels
//! are honest about what they cost and what can kill them.

use std::fmt;

/// Capital required to reach first revenue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvestmentLevel {
    /// Time and a domain account; no inventory, no infra spend.
    NearZero,
    /// Modest build cost, paid tooling, or contract labor.
    Low,
    /// Serious capital: hardware, inventory, regulated infrastructure.
    High,
}

impl InvestmentLevel {
    /// Parse from a report string.
    #[must_use]
    pub fn parse(s: &str) -> Option<InvestmentLevel> {
        match s {
            "near-zero" => Some(InvestmentLevel::NearZero),
            "low" => Some(InvestmentLevel::Low),
            "high" => Some(InvestmentLevel::High),
            _ => None,
        }
    }

    /// Display string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            InvestmentLevel::NearZero => "near-zero",
            InvestmentLevel::Low => "low",
            InvestmentLevel::High => "high",
        }
    }
}

impl fmt::Display for InvestmentLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What can kill the idea.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Demand proven, distribution owned, no regulatory exposure.
    Low,
    /// Competition or platform dependency, but survivable.
    Medium,
    /// Legal gray zones, vendor cat-and-mouse, or unproven demand.
    High,
}

impl RiskLevel {
    /// Parse from a report string.
    #[must_use]
    pub fn parse(s: &str) -> Option<RiskLevel> {
        match s {
            "low" => Some(RiskLevel::Low),
            "medium" => Some(RiskLevel::Medium),
            "high" => Some(RiskLevel::High),
            _ => None,
        }
    }

    /// Display string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
        }
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One business idea, scored into the investment-by-risk taxonomy and
/// backed by measured trend signals.
#[derive(Debug, Clone, PartialEq)]
pub struct BusinessIdea {
    /// Short name.
    pub name: String,
    /// Sector or category.
    pub sector: String,
    /// One-paragraph summary of the offer.
    pub summary: String,
    /// Capital required.
    pub investment: InvestmentLevel,
    /// What can kill it.
    pub risk: RiskLevel,
    /// The primary risk in one line.
    pub primary_risk: String,
    /// trendkit subject keys that evidence the demand.
    pub signals: Vec<String>,
}

impl BusinessIdea {
    /// The taxonomy cell this idea occupies.
    #[must_use]
    pub fn cell(&self) -> String {
        format!("investment: {} | risk: {}", self.investment, self.risk)
    }
}

/// The seeded idea catalog. Each entry cites measured trendkit subjects;
/// entries without live evidence do not belong here.
#[must_use]
pub fn catalog() -> Vec<BusinessIdea> {
    vec![
        BusinessIdea {
            name: "agent-skill verification service".to_owned(),
            sector: "agent-skill economy".to_owned(),
            summary: "Independent verification and trust scoring for third-party \
                      agent skills: structural lint, engagement-integrity audit, \
                      provenance checks. Rides the skill gold rush where clone \
                      farms and inflated stars are already documented."
                .to_owned(),
            investment: InvestmentLevel::NearZero,
            risk: RiskLevel::Medium,
            primary_risk: "Platform-native verification (Anthropic-shipped) \
                           compresses the niche"
                .to_owned(),
            signals: vec![
                "cloudflare/security-audit-skill".to_owned(),
                "MadsLorentzen/ai-job-search".to_owned(),
                "storytold/wordcraft".to_owned(),
            ],
        },
        BusinessIdea {
            name: "ComfyUI workflow template packs".to_owned(),
            sector: "local-AI image generation".to_owned(),
            summary: "Version-pinned, validated workflow packs for the fastest \
                      accelerating image models (Qwen-Image-2.1), schema-checked \
                      before import. Sells to the ComfyUI user base already \
                      measured at 140k stars and accelerating downloads."
                .to_owned(),
            investment: InvestmentLevel::NearZero,
            risk: RiskLevel::Medium,
            primary_risk: "Upstream workflow format churn; free alternatives".to_owned(),
            signals: vec![
                "Comfy-Org/ComfyUI".to_owned(),
                "Comfy-Org/Qwen-Image-2.1".to_owned(),
            ],
        },
        BusinessIdea {
            name: "supply-chain advisory alerting".to_owned(),
            sector: "rust / infrastructure".to_owned(),
            summary: "Managed RustSec + cargo-deny monitoring with same-day \
                      advisory routing. The rustls/zip advisory wave of \
                      2026-10 broke CI across the ecosystem within 48 hours; \
                      most teams found out from their build."
                .to_owned(),
            investment: InvestmentLevel::Low,
            risk: RiskLevel::Low,
            primary_risk: "Commoditized by cargo-audit + GitHub dependabot".to_owned(),
            signals: vec!["hyper-rustls download acceleration (cratesio)".to_owned()],
        },
        BusinessIdea {
            name: "agent memory and handoff tooling".to_owned(),
            sector: "agent infrastructure".to_owned(),
            summary: "Long-term memory and session-handoff layers for coding \
                      agents — the akitaonrails/ai-memory wave is Rust, \
                      accelerating three days straight, and the handoff gap \
                      between CLI sessions is the documented pain."
                .to_owned(),
            investment: InvestmentLevel::Low,
            risk: RiskLevel::Medium,
            primary_risk: "Agent vendors shipping memory natively".to_owned(),
            signals: vec![
                "akitaonrails/ai-memory".to_owned(),
                "anomalyco/opencode".to_owned(),
                "stablyai/orca".to_owned(),
            ],
        },
        BusinessIdea {
            name: "acquisition-ready roll-up screening".to_owned(),
            sector: "private equity / search funds".to_owned(),
            summary: "EDGAR-measured M&A velocity by sector and SIC, screened \
                      into acquisition-ready target lists for search funds — \
                      the crawler's filing signals are the raw material and \
                      PE acquisition trends are the demand."
                .to_owned(),
            investment: InvestmentLevel::Low,
            risk: RiskLevel::Medium,
            primary_risk: "Incumbent data vendors (Crunchbase, PitchBook)".to_owned(),
            signals: vec!["edgar 8-K acquisition filings (measured)".to_owned()],
        },
        BusinessIdea {
            name: "consumer TV debloat tooling".to_owned(),
            sector: "consumer software".to_owned(),
            summary: "Android/Google TV debloating and launcher replacement — \
                      an organic consumer wave (cleanyourtv 24.8k stars in six \
                      days, multiple independent authors, moderate fork \
                      ratios) with ad-removal as the documented pain."
                .to_owned(),
            investment: InvestmentLevel::Low,
            risk: RiskLevel::High,
            primary_risk: "Vendor firmware countermeasures; app-store policy \
                           and legal exposure"
                .to_owned(),
            signals: vec![
                "Gathin23/cleanyourtv".to_owned(),
                "simbasjungles/free-my-tv".to_owned(),
                "pengchujin/aerial-tv".to_owned(),
            ],
        },
        BusinessIdea {
            name: "vertical AI inference platform".to_owned(),
            sector: "AI infrastructure".to_owned(),
            summary: "Vertical inference platform on the DeepInfer model — \
                      high capital for GPU capacity and enterprise sales; \
                      only justified if the local-inference wave keeps \
                      accelerating for another quarter."
                .to_owned(),
            investment: InvestmentLevel::High,
            risk: RiskLevel::High,
            primary_risk: "Frontier-lab price war (DeepSeek/Mistral/Gemini \
                           racing to the bottom on inference pricing)"
                .to_owned(),
            signals: vec![
                "deepseek-ai/DeepInfer".to_owned(),
                "Mistral Large 4 (hn 2042 points)".to_owned(),
                "Gemini 4 Argon (hn 1465 points)".to_owned(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn every_idea_cites_signals() {
        for idea in catalog() {
            assert!(
                !idea.signals.is_empty(),
                "{} has no measured signals",
                idea.name
            );
            assert!(
                !idea.primary_risk.is_empty(),
                "{} has no primary risk",
                idea.name
            );
        }
    }

    #[test]
    fn taxonomy_covers_the_full_matrix() {
        let cells: std::collections::BTreeSet<String> =
            catalog().iter().map(|i| i.cell()).collect();
        // The near-zero/medium, low/low, low/medium, low/high, high/high
        // cells are all represented in the current catalog.
        for cell in [
            "investment: near-zero | risk: medium",
            "investment: low | risk: low",
            "investment: low | risk: medium",
            "investment: low | risk: high",
            "investment: high | risk: high",
        ] {
            assert!(cells.contains(cell), "missing cell: {cell}");
        }
    }

    #[test]
    fn levels_round_trip_through_parse() {
        for level in [
            InvestmentLevel::NearZero,
            InvestmentLevel::Low,
            InvestmentLevel::High,
        ] {
            assert_eq!(InvestmentLevel::parse(level.as_str()), Some(level));
        }
        assert_eq!(InvestmentLevel::parse("nope"), None);
        for risk in [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High] {
            assert_eq!(RiskLevel::parse(risk.as_str()), Some(risk));
        }
    }
}

/// SIC code division → business sector (top-level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sector {
    /// Agriculture, Forestry, Fishing (01-09)
    Agriculture,
    /// Mining, Oil & Gas (10-14)
    Mining,
    /// Construction (15-17)
    Construction,
    /// Manufacturing (20-39)
    Manufacturing,
    /// Transport, Communications, Utilities (40-49)
    Transport,
    /// Wholesale Trade (50-51)
    Wholesale,
    /// Retail Trade (52-59)
    Retail,
    /// Finance, Insurance, Real Estate (60-67)
    Finance,
    /// Services incl. Software (70-89, 91-97)
    Services,
    /// Public Administration (91-99)
    PublicAdmin,
    /// Unclassified
    Other,
}

impl Sector {
    /// Map a SIC division code (first 2 digits) to a business sector.
    #[must_use]
    pub fn from_division(code: u32) -> Sector {
        match code {
            1..=9 => Sector::Agriculture,
            10..=14 => Sector::Mining,
            15..=17 => Sector::Construction,
            20..=39 => Sector::Manufacturing,
            40..=49 => Sector::Transport,
            50..=51 => Sector::Wholesale,
            52..=59 => Sector::Retail,
            60..=67 => Sector::Finance,
            70..=89 | 91..=97 => Sector::Services,
            _ => Sector::Other,
        }
    }

    /// Display string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Sector::Agriculture => "Agriculture",
            Sector::Mining => "Mining",
            Sector::Construction => "Construction",
            Sector::Manufacturing => "Manufacturing",
            Sector::Transport => "Transport",
            Sector::Wholesale => "Wholesale",
            Sector::Retail => "Retail",
            Sector::Finance => "Finance",
            Sector::Services => "Services",
            Sector::PublicAdmin => "Public Administration",
            Sector::Other => "Other",
        }
    }
}

impl std::fmt::Display for Sector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Add `Other` variant for unmapped SIC codes.
impl Sector {
    /// See [`Sector::as_str`].
    #[must_use]
    pub fn all() -> [Sector; 11] {
        [
            Sector::Agriculture,
            Sector::Mining,
            Sector::Construction,
            Sector::Manufacturing,
            Sector::Transport,
            Sector::Wholesale,
            Sector::Retail,
            Sector::Finance,
            Sector::Services,
            Sector::PublicAdmin,
            Sector::Other,
        ]
    }
}

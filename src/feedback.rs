//! What a piece of feedback is: its status, the signals attached to it, and
//! the shapes the grouping, the summary and the ranking answer with.
//!
//! The operations are the submodules: reading one piece as given, grouping
//! duplicates, summarising a set, and ranking by a declared policy.

pub mod dedupe;
pub mod normalize;
pub mod ranking;
pub mod summary;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackStatus {
    New,
    Reviewed,
    Planned,
    Shipped,
    Declined,
}

impl Default for FeedbackStatus {
    fn default() -> Self {
        Self::New
    }
}

impl FeedbackStatus {
    fn name(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Reviewed => "reviewed",
            Self::Planned => "planned",
            Self::Shipped => "shipped",
            Self::Declined => "declined",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackSignals {
    #[serde(default)]
    pub votes: f64,
    #[serde(default)]
    pub affected_accounts: f64,
    #[serde(default)]
    pub severity: f64,
    #[serde(default)]
    pub revenue_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    pub id: String,
    pub submitted_at: String,
    pub source: String,
    pub text: String,
    #[serde(default)]
    pub status: FeedbackStatus,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub product_area: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub evidence_url: Option<String>,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub signals: FeedbackSignals,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackGroup {
    pub key: String,
    pub canonical: Feedback,
    pub duplicates: Vec<Feedback>,
    pub record_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CountEntry {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackSummary {
    pub records: usize,
    pub duplicate_groups: usize,
    pub unique_accounts: usize,
    pub unique_users: usize,
    pub sources: Vec<CountEntry>,
    pub product_areas: Vec<CountEntry>,
    pub tags: Vec<CountEntry>,
    pub statuses: Vec<CountEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingWeights {
    #[serde(default = "one")]
    pub records: f64,
    #[serde(default = "one")]
    pub votes: f64,
    #[serde(default = "one")]
    pub affected_accounts: f64,
    #[serde(default = "one")]
    pub severity: f64,
    #[serde(default)]
    pub revenue_impact: f64,
}

fn one() -> f64 {
    1.0
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            records: 1.0,
            votes: 1.0,
            affected_accounts: 1.0,
            severity: 1.0,
            revenue_impact: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RankingPolicy {
    #[serde(default)]
    pub weights: RankingWeights,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingComponents {
    pub records: f64,
    pub votes: f64,
    pub affected_accounts: f64,
    pub severity: f64,
    pub revenue_impact: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedFeedback {
    pub key: String,
    pub canonical: Feedback,
    pub record_ids: Vec<String>,
    pub components: RankingComponents,
    pub contributions: RankingComponents,
    pub score: f64,
}

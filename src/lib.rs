use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, SecondsFormat, Utc};
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

fn required(value: &str, label: &str) -> Result<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        bail!("{label} must be a non-empty string");
    }
    Ok(normalized)
}

fn optional(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let normalized = item.trim();
        (!normalized.is_empty()).then(|| normalized.to_owned())
    })
}

fn parse_date(value: &str, label: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("{label} must be a valid ISO-8601 date"))
        .map(|date| date.with_timezone(&Utc))
}

fn format_date(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn non_negative(value: f64, label: &str) -> Result<f64> {
    if !value.is_finite() || value < 0.0 {
        bail!("{label} must be a non-negative number");
    }
    Ok(value)
}

fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>> {
    let mut normalized = BTreeSet::new();
    for tag in tags {
        normalized.insert(required(&tag, "tag")?.to_lowercase());
    }
    Ok(normalized.into_iter().collect())
}

pub fn normalize_feedback(mut feedback: Feedback) -> Result<Feedback> {
    feedback.id = required(&feedback.id, "feedback.id")?;
    feedback.submitted_at =
        format_date(parse_date(&feedback.submitted_at, "feedback.submittedAt")?);
    feedback.source = required(&feedback.source, "feedback.source")?.to_lowercase();
    feedback.text = required(&feedback.text, "feedback.text")?;
    feedback.tags = normalize_tags(feedback.tags)?;
    feedback.product_area = optional(feedback.product_area);
    feedback.account_id = optional(feedback.account_id);
    feedback.user_id = optional(feedback.user_id);
    feedback.evidence_url = optional(feedback.evidence_url);
    feedback.external_id = optional(feedback.external_id);
    feedback.signals.votes = non_negative(feedback.signals.votes, "feedback.signals.votes")?;
    feedback.signals.affected_accounts = non_negative(
        feedback.signals.affected_accounts,
        "feedback.signals.affectedAccounts",
    )?;
    feedback.signals.severity =
        non_negative(feedback.signals.severity, "feedback.signals.severity")?;
    if feedback.signals.severity > 5.0 {
        bail!("feedback.signals.severity must be between 0 and 5");
    }
    feedback.signals.revenue_impact = non_negative(
        feedback.signals.revenue_impact,
        "feedback.signals.revenueImpact",
    )?;
    Ok(feedback)
}

fn canonical_key(feedback: &Feedback) -> String {
    match &feedback.external_id {
        Some(external_id) => format!("external:{}:{external_id}", feedback.source),
        None => format!("text:{}", feedback.text.to_lowercase()),
    }
}

pub fn deduplicate_feedback(inputs: Vec<Feedback>) -> Result<Vec<FeedbackGroup>> {
    let mut groups: HashMap<String, Vec<Feedback>> = HashMap::new();
    for input in inputs {
        let feedback = normalize_feedback(input)?;
        groups
            .entry(canonical_key(&feedback))
            .or_default()
            .push(feedback);
    }
    let mut result = Vec::with_capacity(groups.len());
    for (key, mut records) in groups {
        let canonical = records.remove(0);
        let mut record_ids = Vec::with_capacity(records.len() + 1);
        record_ids.push(canonical.id.clone());
        record_ids.extend(records.iter().map(|record| record.id.clone()));
        record_ids.sort();
        result.push(FeedbackGroup {
            key,
            canonical,
            duplicates: records,
            record_ids,
        });
    }
    result.sort_by(|left, right| {
        left.canonical
            .submitted_at
            .cmp(&right.canonical.submitted_at)
            .then_with(|| left.key.cmp(&right.key))
    });
    Ok(result)
}

fn count_entries(counts: BTreeMap<String, usize>) -> Vec<CountEntry> {
    let mut entries: Vec<_> = counts
        .into_iter()
        .map(|(key, count)| CountEntry { key, count })
        .collect();
    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.key.cmp(&right.key))
    });
    entries
}

fn increment(counts: &mut BTreeMap<String, usize>, key: Option<&str>) {
    if let Some(key) = key {
        *counts.entry(key.to_owned()).or_default() += 1;
    }
}

pub fn summarize_feedback(inputs: Vec<Feedback>) -> Result<FeedbackSummary> {
    let records: Vec<_> = inputs
        .into_iter()
        .map(normalize_feedback)
        .collect::<Result<_>>()?;
    let groups = deduplicate_feedback(records.clone())?;
    let mut sources = BTreeMap::new();
    let mut product_areas = BTreeMap::new();
    let mut tags = BTreeMap::new();
    let mut statuses = BTreeMap::new();
    let mut accounts = HashSet::new();
    let mut users = HashSet::new();
    for record in &records {
        increment(&mut sources, Some(&record.source));
        increment(&mut product_areas, record.product_area.as_deref());
        increment(&mut statuses, Some(record.status.name()));
        for tag in &record.tags {
            increment(&mut tags, Some(tag));
        }
        accounts.extend(record.account_id.iter().cloned());
        users.extend(record.user_id.iter().cloned());
    }
    Ok(FeedbackSummary {
        records: records.len(),
        duplicate_groups: groups
            .iter()
            .filter(|group| !group.duplicates.is_empty())
            .count(),
        unique_accounts: accounts.len(),
        unique_users: users.len(),
        sources: count_entries(sources),
        product_areas: count_entries(product_areas),
        tags: count_entries(tags),
        statuses: count_entries(statuses),
    })
}

fn validate_weights(weights: &RankingWeights) -> Result<()> {
    non_negative(weights.records, "policy.weights.records")?;
    non_negative(weights.votes, "policy.weights.votes")?;
    non_negative(weights.affected_accounts, "policy.weights.affectedAccounts")?;
    non_negative(weights.severity, "policy.weights.severity")?;
    non_negative(weights.revenue_impact, "policy.weights.revenueImpact")?;
    Ok(())
}

pub fn rank_feedback(inputs: Vec<Feedback>, policy: RankingPolicy) -> Result<Vec<RankedFeedback>> {
    validate_weights(&policy.weights)?;
    let mut ranked = Vec::new();
    for group in deduplicate_feedback(inputs)? {
        let mut records = Vec::with_capacity(group.duplicates.len() + 1);
        records.push(group.canonical.clone());
        records.extend(group.duplicates);
        let unique_accounts = records
            .iter()
            .filter_map(|record| record.account_id.as_deref())
            .collect::<HashSet<_>>()
            .len() as f64;
        let components = RankingComponents {
            records: records.len() as f64,
            votes: records.iter().map(|record| record.signals.votes).sum(),
            affected_accounts: records
                .iter()
                .map(|record| record.signals.affected_accounts)
                .fold(unique_accounts, f64::max),
            severity: records
                .iter()
                .map(|record| record.signals.severity)
                .fold(0.0, f64::max),
            revenue_impact: records
                .iter()
                .map(|record| record.signals.revenue_impact)
                .sum(),
        };
        let contributions = RankingComponents {
            records: components.records * policy.weights.records,
            votes: components.votes * policy.weights.votes,
            affected_accounts: components.affected_accounts * policy.weights.affected_accounts,
            severity: components.severity * policy.weights.severity,
            revenue_impact: components.revenue_impact * policy.weights.revenue_impact,
        };
        let score = contributions.records
            + contributions.votes
            + contributions.affected_accounts
            + contributions.severity
            + contributions.revenue_impact;
        ranked.push(RankedFeedback {
            key: group.key,
            canonical: group.canonical,
            record_ids: group.record_ids,
            components,
            contributions,
            score,
        });
    }
    ranked.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| {
                left.canonical
                    .submitted_at
                    .cmp(&right.canonical.submitted_at)
            })
            .then_with(|| left.key.cmp(&right.key))
    });
    Ok(ranked)
}

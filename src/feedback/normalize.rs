//! Reading one piece of feedback as given: the fields it must carry, its
//! dates, its numbers and its tags.

use std::collections::BTreeSet;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, SecondsFormat, Utc};

use super::Feedback;

pub(crate) fn required(value: &str, label: &str) -> Result<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        bail!("{label} must be a non-empty string");
    }
    Ok(normalized)
}

pub(crate) fn optional(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let normalized = item.trim();
        (!normalized.is_empty()).then(|| normalized.to_owned())
    })
}

pub(crate) fn parse_date(value: &str, label: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("{label} must be a valid ISO-8601 date"))
        .map(|date| date.with_timezone(&Utc))
}

pub(crate) fn format_date(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub(crate) fn non_negative(value: f64, label: &str) -> Result<f64> {
    if !value.is_finite() || value < 0.0 {
        bail!("{label} must be a non-negative number");
    }
    Ok(value)
}

pub(crate) fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>> {
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

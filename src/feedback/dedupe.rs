//! Grouping the pieces that say the same thing, and what each group is
//! keyed by.

use std::collections::HashMap;

use anyhow::Result;

use super::{Feedback, FeedbackGroup};
use super::normalize::normalize_feedback;

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

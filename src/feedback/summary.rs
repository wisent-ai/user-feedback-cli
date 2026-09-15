//! Summarising a set of feedback: how many of each value, and the totals a
//! reader asks for first.

use std::collections::{BTreeMap, HashSet};

use anyhow::Result;

use super::{CountEntry, Feedback, FeedbackSummary};
use super::dedupe::deduplicate_feedback;
use super::normalize::normalize_feedback;

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

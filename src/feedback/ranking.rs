//! Ranking by a declared policy: the weights it must satisfy, the
//! components of each score, and the order they produce.

use std::collections::HashSet;

use anyhow::Result;

use super::{Feedback, RankedFeedback, RankingComponents, RankingPolicy, RankingWeights};
use super::dedupe::deduplicate_feedback;
use super::normalize::non_negative;

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

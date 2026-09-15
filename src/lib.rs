//! User feedback: one shape for a piece of it, and the four questions asked
//! of a set — read it, group the duplicates, summarise it, rank it.
//!
//! The shapes and the operations live in `feedback`; this file is the surface
//! the CLI and any other caller uses, so the crate's API is one list.

pub mod feedback;

pub use feedback::{
    CountEntry, Feedback, FeedbackGroup, FeedbackSignals, FeedbackStatus, FeedbackSummary,
    RankedFeedback, RankingComponents, RankingPolicy, RankingWeights,
};
pub use feedback::dedupe::deduplicate_feedback;
pub use feedback::normalize::normalize_feedback;
pub use feedback::ranking::rank_feedback;
pub use feedback::summary::summarize_feedback;

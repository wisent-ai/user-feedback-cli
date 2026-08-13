use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Serialize, de::DeserializeOwned};
use user_feedback_cli::{
    Feedback, RankingPolicy, deduplicate_feedback, normalize_feedback, rank_feedback,
    summarize_feedback,
};

#[derive(Parser)]
#[command(
    name = "user-feedback",
    version,
    about = "Evidence-preserving user feedback normalization and prioritization CLI",
    after_help = "The CLI preserves submitted text and ranks only explicit numeric signals."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Normalize {
        #[arg(long)]
        feedback: PathBuf,
    },
    Dedupe {
        #[arg(long)]
        feedback: PathBuf,
    },
    Summarize {
        #[arg(long)]
        feedback: PathBuf,
    },
    Rank {
        #[arg(long)]
        feedback: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
    },
}

fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let input = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&input).with_context(|| format!("invalid JSON in {}", path.display()))
}

fn output<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Normalize { feedback } => {
            output(&normalize_feedback(read_json::<Feedback>(&feedback)?)?)
        }
        Command::Dedupe { feedback } => output(&deduplicate_feedback(read_json::<Vec<Feedback>>(
            &feedback,
        )?)?),
        Command::Summarize { feedback } => {
            output(&summarize_feedback(read_json::<Vec<Feedback>>(&feedback)?)?)
        }
        Command::Rank { feedback, policy } => {
            let policy = policy
                .as_ref()
                .map(read_json::<RankingPolicy>)
                .transpose()?
                .unwrap_or_default();
            output(&rank_feedback(
                read_json::<Vec<Feedback>>(&feedback)?,
                policy,
            )?)
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

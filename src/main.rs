use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use user_feedback_cli::{
    Feedback, RankingPolicy, deduplicate_feedback, normalize_feedback, rank_feedback,
    summarize_feedback,
};

#[derive(Parser)]
#[command(
    name = "user-feedback",
    version,
    about = "Evidence-preserving user feedback normalization and prioritization CLI",
    after_help = "The CLI preserves submitted text and ranks only explicit numeric signals. Every command prints JSON, or with --text one `path: value` line per field from the same data. Exit 2: the invocation is wrong; exit 1: an input could not be read or was refused."
)]
struct Cli {
    /// Print one `path: value` line per field instead of JSON.
    #[arg(long, global = true)]
    text: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read one feedback record and print it in canonical form.
    Normalize {
        /// The feedback record JSON file.
        #[arg(long)]
        feedback: PathBuf,
    },
    /// Group records that report the same thing.
    Dedupe {
        /// A JSON array of feedback records.
        #[arg(long)]
        feedback: PathBuf,
    },
    /// Count records by their fields.
    Summarize {
        /// A JSON array of feedback records.
        #[arg(long)]
        feedback: PathBuf,
    },
    /// Order feedback by its explicit numeric signals.
    Rank {
        /// A JSON array of feedback records.
        #[arg(long)]
        feedback: PathBuf,
        /// A ranking policy JSON file with the signal weights; the built-in weights when omitted.
        #[arg(long)]
        policy: Option<PathBuf>,
    },
}

fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let input = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&input).with_context(|| format!("invalid JSON in {}", path.display()))
}

/// JSON, or with `--text` the same document as one `path: value` line per
/// field (cli.md rule 13).
fn output<T: Serialize>(value: &T, text: bool) -> Result<()> {
    let document = serde_json::to_value(value)?;
    if !text {
        println!("{}", serde_json::to_string_pretty(&document)?);
        return Ok(());
    }
    let mut lines = Vec::new();
    text_lines(&document, String::new(), &mut lines);
    println!("{}", lines.join("\n"));
    Ok(())
}

fn text_lines(node: &Value, path: String, lines: &mut Vec<String>) {
    let child = |key: &str| if path.is_empty() { key.to_owned() } else { format!("{path}.{key}") };
    match node {
        Value::Array(items) if !items.is_empty() => {
            for (index, item) in items.iter().enumerate() {
                text_lines(item, format!("{path}[{index}]"), lines);
            }
        }
        Value::Object(fields) if !fields.is_empty() => {
            for (key, item) in fields {
                text_lines(item, child(key), lines);
            }
        }
        Value::String(value) => lines.push(format!("{path}: {value}")),
        Value::Null | Value::Array(_) | Value::Object(_) => lines.push(format!("{path}: -")),
        other => lines.push(format!("{path}: {other}")),
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let text = cli.text;
    match cli.command {
        Command::Normalize { feedback } => {
            output(&normalize_feedback(read_json::<Feedback>(&feedback)?)?, text)
        }
        Command::Dedupe { feedback } => output(&deduplicate_feedback(read_json::<Vec<Feedback>>(
            &feedback,
        )?)?, text),
        Command::Summarize { feedback } => {
            output(&summarize_feedback(read_json::<Vec<Feedback>>(&feedback)?)?, text)
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
            )?, text)
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

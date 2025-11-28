use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use policy_diff_viewer::{
    diff_files, PolicyDiffEntry, PolicyDiffFormat, PolicyDiffKind, PolicyDiffError,
};

#[derive(Debug, Parser)]
#[command(
    name = "policy_diff_viewer",
    about = "Diff OverFlow policy artifacts (JSON/YAML/text) for human review"
)]
struct Cli {
    /// Current (live) policy file
    #[arg(long)]
    current: PathBuf,

    /// Proposed or mutated policy file
    #[arg(long)]
    proposed: PathBuf,

    /// Explicit format override: json | yaml | text
    #[arg(long, value_enum)]
    format: Option<FormatArg>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum FormatArg {
    Json,
    Yaml,
    Text,
}

impl From<FormatArg> for PolicyDiffFormat {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::Json => PolicyDiffFormat::Json,
            FormatArg::Yaml => PolicyDiffFormat::Yaml,
            FormatArg::Text => PolicyDiffFormat::Text,
        }
    }
}

fn print_entry(entry: &PolicyDiffEntry) {
    match entry.kind {
        PolicyDiffKind::Added => {
            println!(
                "+ {:<40} => {}",
                entry.path,
                entry
                    .new
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "null".to_string())
            );
        }
        PolicyDiffKind::Removed => {
            println!(
                "- {:<40} => {}",
                entry.path,
                entry
                    .old
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "null".to_string())
            );
        }
        PolicyDiffKind::Modified => {
            let old = entry
                .old
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string());
            let new = entry
                .new
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string());
            println!("~ {:<40} {}  ->  {}", entry.path, old, new);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let format = cli.format.map(PolicyDiffFormat::from);

    let result = diff_files(&cli.current, &cli.proposed, format);

    match result {
        Ok(entries) => {
            if entries.is_empty() {
                println!("No differences detected.");
            } else {
                for entry in &entries {
                    print_entry(entry);
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {err}");
            // Map to non-zero exit code for shell integration.
            let _ = match err {
                PolicyDiffError::Io(_) => 1,
                PolicyDiffError::Json(_) => 2,
                PolicyDiffError::Yaml(_) => 3,
                PolicyDiffError::UnsupportedFormat(_) => 4,
            };
            std::process::exit(1);
        }
    }
}
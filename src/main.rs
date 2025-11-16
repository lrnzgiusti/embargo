use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::time::Instant;

mod core;
mod formatters;
mod parsers;

use crate::core::CodebaseAnalyzer;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "embargo",
    version = "0.1.0",
    author = "embargo developers",
    about = "Ultrafast codebase dependency extractor - Sub-1s analysis"
)]
struct Cli {
    /// Input directory to analyze
    #[arg(short, long, value_name = "PATH")]
    input: PathBuf,

    /// Output file path
    #[arg(short, long, value_name = "FILE", default_value = "EMBARGO.md")]
    output: PathBuf,

    /// Comma-separated list of languages to analyze
    #[arg(
        short,
        long,
        value_name = "LANGS",
        value_delimiter = ',',
        default_value = "python,typescript,javascript,cpp,rust,java,go,csharp"
    )]
    languages: Vec<String>,

    /// Output format: markdown, llm-optimized, json-compact
    #[arg(short, long, value_name = "FORMAT", value_enum, default_value_t = OutputFormat::LlmOptimized)]
    format: OutputFormat,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum OutputFormat {
    Markdown,
    LlmOptimized,
    JsonCompact,
}

impl OutputFormat {
    fn as_str(self) -> &'static str {
        match self {
            OutputFormat::Markdown => "markdown",
            OutputFormat::LlmOptimized => "llm-optimized",
            OutputFormat::JsonCompact => "json-compact",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> Result<()> {
    let Cli {
        input,
        output,
        languages,
        format,
    } = cli;

    let start_time = Instant::now();

    let normalized_languages: Vec<String> = languages
        .into_iter()
        .map(|lang| lang.trim().to_string())
        .filter(|lang| !lang.is_empty())
        .collect();
    let language_refs: Vec<&str> = normalized_languages.iter().map(String::as_str).collect();

    println!("EMBARGO - Ultrafast Codebase Analysis");
    println!("Input: {} (targeting <1s)", input.display());
    println!("Output: {}", output.display());
    println!("Format: {}", format.as_str());
    println!("Languages: {:?}", normalized_languages);

    let analysis_start = Instant::now();

    let mut analyzer = CodebaseAnalyzer::new();
    let dependency_graph = analyzer.analyze(&input, &language_refs)?;

    let analysis_time = analysis_start.elapsed();
    println!(
        "Analysis completed in {:.2}s",
        analysis_time.as_secs_f64()
    );

    let mut generated_output = output.clone();

    match format {
        OutputFormat::Markdown => {
            use crate::formatters::EmbargoFormatter;
            EmbargoFormatter::new().format_to_file(&dependency_graph, &output)?;
        }
        OutputFormat::LlmOptimized => {
            use crate::formatters::LLMOptimizedFormatter;
            let formatter = if language_refs.iter().any(|lang| *lang == "python") {
                LLMOptimizedFormatter::for_python()
            } else {
                LLMOptimizedFormatter::new()
            }
            .with_hierarchical(true)
            .with_compressed_ids(true);
            formatter.format_to_file(&dependency_graph, &output)?;
        }
        OutputFormat::JsonCompact => {
            use crate::formatters::JsonCompactFormatter;
            let formatter = JsonCompactFormatter::new();
            generated_output = output.with_extension("json");
            formatter.format_to_file(&dependency_graph, &generated_output)?;
            println!("JSON output: {}", generated_output.display());
        }
    }

    let total_time = start_time.elapsed();
    println!(
        "Analysis complete. Generated {}",
        generated_output.display()
    );
    println!("Total execution time: {:.2}s", total_time.as_secs_f64());

    if total_time.as_secs_f64() < 1.0 {
        println!("Sub-1 second execution achieved.");
    } else {
        println!(
            "Execution time: {:.2}s (optimizations in progress)",
            total_time.as_secs_f64()
        );
    }

    Ok(())
}

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use markdown_inspector::{
    extract_section, find_section, format_outline_entry, get_section_range, get_subsections,
    parse_headings,
};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mdi",
    about = "Markdown Inspector - explore markdown document structure"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show document outline with line numbers
    Outline {
        /// Markdown file to inspect (use - for stdin)
        file: PathBuf,

        /// Maximum heading depth to show (1-6)
        #[arg(short, long, default_value = "6")]
        depth: u8,
    },

    /// Read a specific section
    Read {
        /// Markdown file to inspect (use - for stdin)
        file: PathBuf,

        /// Section to read: line number or heading text (partial match)
        section: String,

        /// Show only the outline of subsections instead of full content
        #[arg(short, long)]
        outline: bool,

        /// Maximum heading depth for outline mode (1-6)
        #[arg(short, long, default_value = "6")]
        depth: u8,
    },
}

fn read_input(file: &PathBuf) -> Result<String> {
    if file.as_os_str() == "-" {
        let mut content = String::new();
        io::stdin()
            .read_to_string(&mut content)
            .context("Failed to read from stdin")?;
        Ok(content)
    } else {
        fs::read_to_string(file).with_context(|| format!("Failed to read file: {:?}", file))
    }
}

fn print_outline(headings: &[&markdown_inspector::Heading], max_depth: u8) {
    for h in headings {
        if h.level <= max_depth {
            println!("{}", format_outline_entry(h));
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Outline { file, depth } => {
            let content = read_input(&file)?;
            let headings = parse_headings(&content);
            let heading_refs: Vec<_> = headings.iter().collect();
            print_outline(&heading_refs, depth);
        }

        Commands::Read {
            file,
            section,
            outline,
            depth,
        } => {
            let content = read_input(&file)?;
            let headings = parse_headings(&content);

            let heading = find_section(&headings, &section)
                .with_context(|| format!("Section not found: {}", section))?;

            let (start, end) = get_section_range(&headings, heading);

            if outline {
                let subsections = get_subsections(&headings, start, end, depth);
                print_outline(&subsections, depth);
            } else {
                let section_content = extract_section(&content, start, end);
                print!("{}", section_content);
                if !section_content.ends_with('\n') {
                    println!();
                }
            }
        }
    }

    Ok(())
}

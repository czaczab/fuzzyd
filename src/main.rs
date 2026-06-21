use clap::Parser;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::env;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug)]
#[command(version, about, long_about = "I am better than cd and zoxide")]
struct CLI {
    #[arg(short = 'l', long = "list", action = clap::ArgAction::SetTrue)]
    list: bool,

    input: Option<String>,
}

fn main() {
    let cli = CLI::parse();
    let list_mode = cli.list;

    let input = cli.input.unwrap_or_else(|| {
        if list_mode {
            eprintln!("No query povided to list.");
            std::process::exit(1);
        }
        env::var("HOME").unwrap_or_else(|_| ".".to_string())
    });

    // Scenario 1: Got an actual path that exists in the system
    let potential_path = PathBuf::from(&input);
    // ignore if the list mode is on
    if potential_path.exists() && !list_mode {
        let absolute_path = potential_path.canonicalize().unwrap_or(potential_path);
        cd(absolute_path);
        return;
    }

    // Scenario 2: Got a search query (not a path)
    let results = perform_search(&input);

    if list_mode {
        print_results(results);
    } else {
        cd(PathBuf::from(results[0].0.path()));
    }
}

fn print_results(results: Vec<(walkdir::DirEntry, i64)>) {
    for (entry, score) in results {
        println!("{} ({})", entry.path().display(), score);
    }
}

fn perform_search(query: &String) -> Vec<(DirEntry, i64)> {
    let matcher = SkimMatcherV2::default();
    let current_dir = env::current_dir().expect("Failed to get current directory");

    let mut results: Vec<(DirEntry, i64)> = WalkDir::new(current_dir)
        .max_depth(2)
        .into_iter()
        .filter_entry(|entry| {
            // Only look inside direcories
            if !entry.file_type().is_dir() {
                return false;
            }
            // Ignore current and parent directories
            if entry.file_name() == "." || entry.file_name() == ".." {
                return false;
            }
            true
        })
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            matcher
                .fuzzy_match(&name, query.as_str())
                .map(|score| (entry, score))
        })
        .collect();

    results.sort_by_key(|a| std::cmp::Reverse(a.1));

    if results.is_empty() {
        eprintln!("No match found for: {}", query);
        std::process::exit(1);
    }
    results
}

fn cd(path: PathBuf) {
    // Prints the directory path to go to for ~/.zshrc to pick up
    // Added only for code readability
    println!("{}", path.display());
}

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::env;
use std::fs::{DirEntry, read_dir};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for list mode
    let list_mode = args.iter().any(|arg| arg == "-l");

    // Remove flags from args iterator
    let clean_args: Vec<&String> = args.iter().skip(1).filter(|arg| *arg != "-l").collect();

    // Scenario 1: If no args left, go to home directory
    if clean_args.is_empty() {
        if list_mode {
            eprintln!("No query povided to list.");
            std::process::exit(1);
        }
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        println!("{}", home);
        return;
    }

    let input = clean_args[0];

    // Scenario 2: Got an actual path that exists in the system
    let potential_path = PathBuf::from(input);

    if potential_path.exists() && !list_mode {
        let absolute_path = potential_path.canonicalize().unwrap_or(potential_path);
        println!("{}", absolute_path.display());
        return;
    }

    // Scenario 3: Got a search query (not a path)
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let results = perform_search(&current_dir, input);

    if results.is_empty() {
        eprintln!("No match found for: {}", input);
        std::process::exit(1);
    }

    if list_mode {
        print_results(results);
    } else {
        println!("{}", results[0].0.path().display())
    }
}

fn print_results(results: Vec<(std::fs::DirEntry, i64)>) {
    for (entry, score) in results {
        println!("{} ({})", entry.path().display(), score);
    }
}

fn perform_search(path: &PathBuf, query: &str) -> Vec<(DirEntry, i64)> {
    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(DirEntry, i64)> = read_dir(path)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            matcher
                .fuzzy_match(&name, query)
                .map(|score| (entry, score))
        })
        .collect();

    results.sort_by_key(|a| std::cmp::Reverse(a.1));
    results
}

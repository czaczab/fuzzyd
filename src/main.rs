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

    #[arg(num_args = 0..)]
    input: Vec<String>,
}

fn main() {
    let cli = CLI::parse();
    let list_mode = cli.list;

    match cli.input.len() {
        0 => handle_path(PathBuf::from(
            env::var("HOME").unwrap_or_else(|_| ".".to_string()),
        )),
        1 => {
            let potential_path = PathBuf::from(&cli.input[0]);
            if potential_path.exists() {
                handle_path(potential_path);
            } else {
                handle_singe_query(cli.input[0].clone(), list_mode);
            }
        }
        _ => handle_multiple_querys(cli.input, list_mode),
    };
}

fn handle_path(path: PathBuf) {
    let absolute_path = path.canonicalize().unwrap_or(path);
    cd(absolute_path);
}

fn handle_singe_query(query: String, list_mode: bool) {
    let results = perform_search(&query, None);

    if results.is_empty() {
        eprintln!("No results found for query: {}", query);
        std::process::exit(0);
    }

    if list_mode {
        print_results(results);
    } else {
        cd(PathBuf::from(results[0].0.path()));
    }
}

fn handle_multiple_querys(mut querys: Vec<String>, list_mode: bool) {
    search_layer(&mut querys, None, list_mode);

    fn search_layer(query_vec: &mut Vec<String>, path: Option<PathBuf>, list_mode: bool) {
        let results = perform_search(&query_vec[0], path);

        if results.is_empty() {
            eprintln!("No results found for query: {}", query_vec[0]);
            std::process::exit(0);
        }

        query_vec.remove(0);

        if query_vec.is_empty() {
            if list_mode {
                print_results(results);
            } else {
                cd(PathBuf::from(results[0].0.path()));
            }
        } else {
            for result in results {
                search_layer(
                    &mut query_vec.clone(),
                    Some(PathBuf::from(result.0.path())),
                    list_mode,
                );
            }
        }
    }
}

fn print_results(results: Vec<(walkdir::DirEntry, i64)>) {
    for (entry, score) in results {
        println!("{} ({})", entry.path().display(), score);
    }
}

fn perform_search(query: &str, path: Option<PathBuf>) -> Vec<(DirEntry, i64)> {
    let matcher = SkimMatcherV2::default();
    let current_dir =
        path.unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));

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
            let name = name.to_string_lossy().to_lowercase();
            matcher
                .fuzzy_match(&name, query)
                // Subtracts the depth to prioritize shallower directories
                .map(|score| (entry.clone(), score - entry.depth() as i64))
        })
        .collect();

    results.sort_by_key(|a| std::cmp::Reverse(a.1));
    results
}

fn cd(path: PathBuf) {
    // Prints the directory path to go to for ~/.zshrc to pick up
    // Added only for code readability
    println!("{}", path.display());
    std::process::exit(0);
}

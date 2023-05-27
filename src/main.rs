use std::path::PathBuf;

use clap::Parser;
use git2::Repository;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

fn find_git_root() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let repo = match Repository::discover(&current_dir) {
        Ok(repo) => repo,
        Err(_) => return None,
    };

    let git_dir = repo.path().parent()?.to_path_buf();
    Some(git_dir)
}

fn get_branch_names(repo: &Repository) -> Vec<String> {
    let branches = match repo.branches(None) {
        Ok(branches) => branches,
        Err(e) => panic!("Failed to get branch iterator: {}", e),
    };

    let branch_names = branches.map(|branch| {
        let branch = match branch {
            Ok((branch, _)) => branch,
            Err(e) => panic!("Failed to get branch: {}", e),
        };

        match branch.name() {
            Ok(Some(name)) => name.to_string(),
            Ok(None) => panic!("Failed to get branch name"),
            Err(e) => panic!("Failed to get branch name: {}", e),
        }
    });

    branch_names.collect::<Vec<String>>()
}

fn main() {
    Args::parse();

    let git_root = match find_git_root() {
        Some(git_root) => git_root,
        None => panic!("Failed to find git root"),
    };

    let repo = match Repository::open(git_root) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to open repository: {}", e),
    };

    let branch_names = get_branch_names(&repo);
    println!("Branches: {:?}", branch_names);
}

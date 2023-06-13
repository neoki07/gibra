use clap::Parser;
use git2::Repository;
use skim::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tuikit::prelude::*;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[derive(Clone, Debug)]
struct Branch {
    name: String,
}

impl SkimItem for Branch {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }
}

fn find_git_root() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let repo = match Repository::discover(&current_dir) {
        Ok(repo) => repo,
        Err(_) => return None,
    };

    let git_dir = repo.path().parent()?.to_path_buf();
    Some(git_dir)
}

fn get_current_branch(repo: &Repository) -> Branch {
    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => panic!("Failed to get HEAD: {}", e),
    };

    let branch = match head.shorthand() {
        Some(branch) => branch,
        None => panic!("Failed to get branch name"),
    };

    Branch {
        name: branch.to_string(),
    }
}

fn get_branches(repo: &Repository) -> Vec<Branch> {
    let branches = match repo.branches(None) {
        Ok(branches) => branches,
        Err(e) => panic!("Failed to get branch iterator: {}", e),
    };

    branches
        .map(|branch| {
            let branch = match branch {
                Ok((branch, _)) => branch,
                Err(e) => panic!("Failed to get branch: {}", e),
            };

            match branch.name() {
                Ok(Some(name)) => Branch {
                    name: name.to_string(),
                },
                Ok(None) => panic!("Failed to get branch name: Empty name"),
                Err(e) => panic!("Failed to get branch name: {}", e),
            }
        })
        .collect()
}

fn checkout(branch_name: String) {
    Command::new("git")
        .args(&["checkout", branch_name.as_str()])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to execute command");
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

    let current_branch = get_current_branch(&repo);
    let branches = get_branches(&repo);

    let options = SkimOptionsBuilder::default().build().unwrap();

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    let _ = tx.send(Arc::new(current_branch.name.clone()));
    branches
        .iter()
        .filter(|branch| branch.name != current_branch.name)
        .for_each(|branch| {
            let _ = tx.send(Arc::new(branch.clone()));
        });

    drop(tx);

    Skim::run_with(&options, Some(rx)).map(|out| match out.final_key {
        Key::Enter | Key::DoubleClick(MouseButton::Left, _, _) => {
            let selected_branch_name = out
                .selected_items
                .first()
                .expect("Failed to get selected item")
                .output()
                .to_string();

            checkout(selected_branch_name);
        }
        _ => (),
    });
}

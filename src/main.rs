use clap::Parser;
use git2::{BranchType, Repository};
use skim::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const REMOTE_BRANCH_NAME_PREFIX: &str = "origin/";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Show only remote branches
    #[clap(short = 'r', long)]
    remote_only: bool,

    /// Show only local branches
    #[clap(short = 'l', long)]
    local_only: bool,
}

#[derive(Clone, Debug)]
struct Branch {
    name: String,
    branch_type: BranchType,
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
        branch_type: BranchType::Local,
    }
}

fn get_branches(repo: &Repository, branch_filter: Option<BranchType>) -> Vec<Branch> {
    let get_local_branches = || match repo.branches(Some(BranchType::Local)) {
        Ok(branches) => branches,
        Err(e) => panic!("Failed to get branch iterator: {}", e),
    };

    let get_remote_branches = || match repo.branches(Some(BranchType::Remote)) {
        Ok(branches) => branches,
        Err(e) => panic!("Failed to get branch iterator: {}", e),
    };

    match branch_filter {
        Some(BranchType::Local) => get_local_branches()
            .map(|branch| {
                let branch = match branch {
                    Ok((branch, _)) => branch,
                    Err(e) => panic!("Failed to get branch: {}", e),
                };

                match branch.name() {
                    Ok(Some(name)) => Branch {
                        name: name.to_string(),
                        branch_type: BranchType::Local,
                    },
                    Ok(None) => panic!("Failed to get branch name: Empty name"),
                    Err(e) => panic!("Failed to get branch name: {}", e),
                }
            })
            .collect(),
        Some(BranchType::Remote) => get_remote_branches()
            .map(|branch| {
                let branch = match branch {
                    Ok((branch, _)) => branch,
                    Err(e) => panic!("Failed to get branch: {}", e),
                };

                match branch.name() {
                    Ok(Some(name)) => Branch {
                        name: name.to_string(),
                        branch_type: BranchType::Local,
                    },
                    Ok(None) => panic!("Failed to get branch name: Empty name"),
                    Err(e) => panic!("Failed to get branch name: {}", e),
                }
            })
            .collect(),
        None => get_local_branches()
            .chain(get_remote_branches())
            .map(|branch| {
                let (branch, branch_type) = match branch {
                    Ok((branch, branch_type)) => (branch, branch_type),
                    Err(e) => panic!("Failed to get branch: {}", e),
                };

                match branch.name() {
                    Ok(Some(name)) => Branch {
                        name: name.to_string(),
                        branch_type,
                    },
                    Ok(None) => panic!("Failed to get branch name: Empty name"),
                    Err(e) => panic!("Failed to get branch name: {}", e),
                }
            })
            .collect(),
    }
}

fn remote_to_local(branch_name: &str) -> String {
    println!("branch_name: {}", branch_name);
    branch_name
        .strip_prefix(REMOTE_BRANCH_NAME_PREFIX)
        .expect("Failed to strip prefix")
        .to_string()
}

fn checkout_local_branch(branch: &Branch) {
    Command::new("git")
        .args(&["checkout", &branch.name])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to execute checkout command");
}

fn checkout_remote_branch(branch: &Branch) {
    let local_branch_name = remote_to_local(&branch.name);

    let repo = Repository::open(find_git_root().expect("Failed to find git root"))
        .expect("Failed to open repository");

    let local_branch_exists = repo
        .find_branch(&local_branch_name, BranchType::Local)
        .is_ok();

    if !local_branch_exists {
        Command::new("git")
            .args(&["checkout", "-b", &local_branch_name, &branch.name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to execute checkout command");
    } else {
        Command::new("git")
            .args(&["checkout", &local_branch_name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to execute checkout command");
    }
}

fn checkout(branch: &Branch) {
    match branch.branch_type {
        BranchType::Local => checkout_local_branch(branch),
        BranchType::Remote => checkout_remote_branch(branch),
    }
}

fn main() {
    let args = Args::parse();

    let branch_filter;
    if args.remote_only && args.local_only {
        panic!("Cannot specify both --remote-only and --local-only");
    } else if args.remote_only {
        branch_filter = Some(BranchType::Remote);
    } else if args.local_only {
        branch_filter = Some(BranchType::Local);
    } else {
        branch_filter = None;
    }

    let git_root = match find_git_root() {
        Some(git_root) => git_root,
        None => panic!("Failed to find git root"),
    };

    let repo = match Repository::open(git_root) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to open repository: {}", e),
    };

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    let current_branch = get_current_branch(&repo);
    if !args.remote_only {
        let _ = tx.send(Arc::new(current_branch.clone()));
    }

    get_branches(&repo, branch_filter)
        .iter()
        .filter(|branch| branch.name != current_branch.name)
        .for_each(|branch| {
            let _ = tx.send(Arc::new(Branch {
                name: branch.name.clone(),
                branch_type: branch.branch_type,
            }));
        });

    drop(tx);

    let options = SkimOptionsBuilder::default().build().unwrap();

    let selected_branch = Skim::run_with(&options, Some(rx))
        .map(|out| out.selected_items)
        .unwrap_or_else(Vec::new)
        .first()
        .map(|selected_item| {
            (**selected_item)
                .as_any()
                .downcast_ref::<Branch>()
                .unwrap()
                .to_owned()
        })
        .expect("Failed to get selected item");

    checkout(&selected_branch);
}

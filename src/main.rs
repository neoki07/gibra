#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use crate::skim::{event::Event, prelude::*};
use anyhow::{Context, Result};
use clap::Parser;
use git2::{BranchType, Repository};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

mod skim;

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

fn find_git_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let repo = Repository::discover(&current_dir)?;
    let git_dir = repo
        .path()
        .parent()
        .with_context(|| "Failed to get parent")?
        .to_path_buf();

    Ok(git_dir)
}

fn get_current_branch(repo: &Repository) -> Result<Branch> {
    let head = repo.head().with_context(|| "Failed to get HEAD")?;
    let current_branch = head
        .shorthand()
        .with_context(|| "Failed to get branch name")?;

    Ok(Branch {
        name: current_branch.to_string(),
        branch_type: BranchType::Local,
    })
}

fn get_branches(repo: &Repository, branch_filter: Option<BranchType>) -> Result<Vec<Branch>> {
    let get_local_branches = || {
        repo.branches(Some(BranchType::Local))
            .with_context(|| "Failed to get local branches")
    };

    let get_remote_branches = || {
        repo.branches(Some(BranchType::Remote))
            .with_context(|| "Failed to get remote branches")
    };

    let branches: Vec<Result<Branch, anyhow::Error>> = match branch_filter {
        Some(BranchType::Local) => get_local_branches()?
            .map(|branch| {
                let branch = match branch {
                    Ok((branch, _)) => branch,
                    Err(e) => return Err(anyhow::Error::msg(e.message().to_string())),
                };

                match branch.name() {
                    Ok(Some(name)) => Ok(Branch {
                        name: name.to_string(),
                        branch_type: BranchType::Local,
                    }),
                    Ok(None) => Err(anyhow::Error::msg("Branch name is empty")),
                    Err(e) => Err(anyhow::Error::msg(e.message().to_string())),
                }
            })
            .collect(),
        Some(BranchType::Remote) => get_remote_branches()?
            .map(|branch| {
                let branch = match branch {
                    Ok((branch, _)) => branch,
                    Err(e) => return Err(anyhow::Error::msg(e.message().to_string())),
                };

                match branch.name() {
                    Ok(Some(name)) => Ok(Branch {
                        name: name.to_string(),
                        branch_type: BranchType::Local,
                    }),
                    Ok(None) => Err(anyhow::Error::msg("Branch name is empty")),
                    Err(e) => Err(anyhow::Error::msg(e.message().to_string())),
                }
            })
            .collect(),
        None => get_local_branches()?
            .chain(get_remote_branches()?)
            .map(|branch| {
                let (branch, branch_type) = match branch {
                    Ok((branch, branch_type)) => (branch, branch_type),
                    Err(e) => return Err(anyhow::Error::msg(e.message().to_string())),
                };

                match branch.name() {
                    Ok(Some(name)) => Ok(Branch {
                        name: name.to_string(),
                        branch_type,
                    }),
                    Ok(None) => Err(anyhow::Error::msg("Branch name is empty")),
                    Err(e) => Err(anyhow::Error::msg(e.message().to_string())),
                }
            })
            .collect(),
    };

    let branches = branches.into_iter().collect::<Result<Vec<Branch>>>()?;
    Ok(branches)
}

fn remote_to_local(branch_name: &str) -> Result<String> {
    let local_branch_name = branch_name
        .strip_prefix(REMOTE_BRANCH_NAME_PREFIX)
        .with_context(|| "Failed to strip prefix")?
        .to_string();

    Ok(local_branch_name)
}

fn checkout_local_branch(branch: &Branch) -> Result<()> {
    Command::new("git")
        .args(&["checkout", &branch.name])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| "Failed to execute checkout command")?;

    Ok(())
}

fn checkout_remote_branch(branch: &Branch) -> Result<()> {
    let local_branch_name = remote_to_local(&branch.name)?;

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
            .with_context(|| "Failed to execute checkout command")?;
    } else {
        Command::new("git")
            .args(&["checkout", &local_branch_name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .with_context(|| "Failed to execute checkout command")?;
    }

    Ok(())
}

fn checkout(branch: &Branch) -> Result<()> {
    match branch.branch_type {
        BranchType::Local => checkout_local_branch(branch),
        BranchType::Remote => checkout_remote_branch(branch),
    }
}

fn main() -> Result<()> {
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

    let git_root = find_git_root().with_context(|| "Failed to find git root")?;
    let repo = Repository::open(git_root.clone()).with_context(|| "Failed to open repository")?;

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    let current_branch =
        get_current_branch(&repo).with_context(|| "Failed to get current branch")?;
    if !args.remote_only {
        let _ = tx.send(Arc::new(current_branch.clone()));
    }

    get_branches(&repo, branch_filter)
        .with_context(|| "Failed to get branches")?
        .iter()
        .filter(|branch| branch.name != current_branch.name)
        .for_each(|branch| {
            let _ = tx.send(Arc::new(Branch {
                name: branch.name.clone(),
                branch_type: branch.branch_type,
            }));
        });

    drop(tx);

    let options = SkimOptionsBuilder::default()
        .build()
        .with_context(|| "Failed to set up")?;

    let selected_branch = Skim::run_with(&options, Some(rx))
        .map(|out| match out.final_event {
            Event::EvActAbort => std::process::exit(130),
            _ => out.selected_items,
        })
        .unwrap_or_else(Vec::new)
        .first()
        .map(|selected_item| {
            (**selected_item)
                .as_any()
                .downcast_ref::<Branch>()
                .with_context(|| "Failed to get selected branch")
                .map(|selected_item| selected_item.to_owned())
        })
        .with_context(|| "Failed to get selected branch")??;

    checkout(&selected_branch).with_context(|| "Failed to checkout branch")?;

    Ok(())
}

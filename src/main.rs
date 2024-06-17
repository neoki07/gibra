#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use crate::skim::{event::Event, prelude::*};
use anyhow::{Context, Result};
use clap::Parser;
use git2::{BranchType, Repository};
use std::{
    collections::HashMap,
    path::PathBuf,
    process::{Command, Stdio},
};

mod skim;

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
struct LocalBranch {
    name: String,
    remote_name: Option<String>,
}

#[derive(Clone, Debug)]
struct RemoteBranch {
    name: String,
    local_name: Option<String>,
}

#[derive(Clone, Debug)]
enum Branch {
    Local(LocalBranch),
    Remote(RemoteBranch),
}

impl Branch {
    fn name(self) -> String {
        match self {
            Branch::Local(local_branch) => local_branch.name,
            Branch::Remote(remote_branch) => remote_branch.name,
        }
    }
}

impl SkimItem for Branch {
    fn text(&self) -> Cow<str> {
        match self {
            Branch::Local(local_branch) => Cow::Borrowed(&local_branch.name),
            Branch::Remote(remote_branch) => Cow::Borrowed(&remote_branch.name),
        }
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

    Ok(Branch::Local(LocalBranch {
        name: current_branch.to_string(),
        remote_name: None,
    }))
}

fn get_branches(repo: &Repository, branch_filter: Option<BranchType>) -> Result<Vec<Branch>> {
    let local_branches: Vec<Branch> = repo
        .branches(Some(BranchType::Local))
        .with_context(|| "Failed to get local branches")?
        .filter_map(|branch| {
            let branch = match branch {
                Ok((branch, _)) => branch,
                Err(_) => return None,
            };

            let branch_name = match branch.name() {
                Ok(Some(name)) => name.to_string(),
                Ok(None) => return None,
                Err(_) => return None,
            };

            let remote_branch_name = match branch.upstream() {
                Ok(upstream) => match upstream.name() {
                    Ok(Some(name)) => Some(name.to_string()),
                    Ok(None) => None,
                    Err(_) => return None,
                },
                Err(_) => return None,
            };

            Some(Branch::Local(LocalBranch {
                name: branch_name,
                remote_name: remote_branch_name,
            }))
        })
        .collect();

    let remote_to_local_map: HashMap<_, _> = local_branches
        .iter()
        .filter_map(|branch| match branch {
            Branch::Local(LocalBranch { name, remote_name }) => match remote_name {
                Some(remote_name) => Some((remote_name, name)),
                _ => None,
            },
            _ => None,
        })
        .collect();

    if let Some(BranchType::Local) = branch_filter {
        return Ok(local_branches);
    }

    let remote_branches: Vec<Branch> = repo
        .branches(Some(BranchType::Remote))
        .with_context(|| "Failed to get remote branches")?
        .filter_map(|branch| {
            let branch = match branch {
                Ok((branch, _)) => branch,
                Err(_) => return None,
            };

            let branch_name = match branch.name() {
                Ok(Some(name)) => name.to_string(),
                Ok(None) => return None,
                Err(_) => return None,
            };

            let local_branch_name = remote_to_local_map
                .get(&branch_name)
                .map(|name| name.to_string());

            Some(Branch::Remote(RemoteBranch {
                name: branch_name,
                local_name: local_branch_name,
            }))
        })
        .collect();

    if let Some(BranchType::Remote) = branch_filter {
        return Ok(remote_branches);
    }

    let branches = local_branches
        .into_iter()
        .chain(remote_branches.into_iter())
        .collect();

    Ok(branches)
}

fn checkout_local_branch(branch: &LocalBranch) -> Result<()> {
    Command::new("git")
        .args(&["checkout", &branch.name])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| "Failed to execute checkout command")?;

    Ok(())
}

fn checkout_remote_branch(branch: &RemoteBranch) -> Result<()> {
    match branch.local_name.clone() {
        Some(local_branch_name) => {
            Command::new("git")
                .args(&["checkout", &local_branch_name])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .with_context(|| "Failed to execute checkout command")?;
        }
        None => {
            Command::new("git")
                .args(&["checkout", "-b", &branch.name])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .with_context(|| "Failed to execute checkout command")?;
        }
    }

    Ok(())
}

fn checkout(branch: &Branch) -> Result<()> {
    match branch {
        Branch::Local(branch) => checkout_local_branch(branch),
        Branch::Remote(branch) => checkout_remote_branch(branch),
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
        .into_iter()
        .filter(|branch| (*branch).clone().name() != current_branch.clone().name())
        .for_each(|branch| {
            let _ = tx.send(Arc::new(branch.clone()));
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

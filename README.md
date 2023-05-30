# [WIP] git-co

`git-co` is a command-line tool that replaces the `git checkout` command, enabling you to switch branches by selecting from the list of branch names.

## Features

- Effortlessly switch between branches using intuitive key-based navigation
- Simplifies the branch selection process by presenting a list of available branches

## Installation

You can install `git-co` using `cargo`, the package manager for `Rust`:

```sh
cargo install git-co
```

Make sure you have Rust and Cargo installed on your system before running the above command.

## Usage

To use `git-co`, simply execute the command. This will launch the branch selection screen where you can choose the branch you want to check out.

```sh
git-co
```

The branch selection screen will display a list of available branches. Use the arrow keys or other supported keys to navigate through the list. Once you've highlighted your desired branch, press `Enter` to check it out.

## License

[MIT License](https://github.com/ot07/git-co/blob/main/LICENSE)

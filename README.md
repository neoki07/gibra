# gibra

`gibra` is a command-line tool that replaces the `git checkout <BRANCH_NAME>` command, enabling you to switch branches by selecting from the list of branch names.

## Features

- Effortlessly switch between branches using intuitive key-based navigation
- Simplifies the branch selection process by presenting a list of available branches

## Installation

You can install `gibra` using `cargo`, the package manager for `Rust`:

```sh
cargo install gibra
```

Make sure you have Rust and Cargo installed on your system before running the above command.

## Usage

To use `gibra`, simply execute the command. This will launch the branch selection screen where you can choose the branch you want to check out.

```sh
gibra
```

The branch selection screen will display a list of available branches. Use the arrow keys or other supported keys to navigate through the list. Once you've highlighted your desired branch, press `Enter` to check it out.

## License

[MIT License](https://github.com/ne-oki/gibra/blob/main/LICENSE)

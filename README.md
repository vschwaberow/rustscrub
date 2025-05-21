# rustscrub

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A CLI tool for scrubbing sensitive data from files.

## Description

# rustscrub

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A CLI tool for scrubbing sensitive data from files.

**Version:** 0.1.0
**Author:** volker schwaberow <volker@schwaberow.de>
**Homepage:** [https://schwaberow.de](https://schwaberow.de)
**Repository:** [https://github.com/vschwaberow/rustscrub](https://github.com/vschwaberow/rustscrub)

## Description

`rustscrub` is a command-line interface (CLI) tool built with Rust, designed to remove comments from source code files. This can be useful for preparing code for distribution, reducing file size, or cleaning up codebases.

The tool focuses on identifying and stripping out comment blocks and lines. It is particularly useful for developers who want to share their code without exposing comments that may contain sensitive information, such as TODOs, internal notes, or any other non-essential information.

## Installation

### Prerequisites

*   Rust programming language (latest stable version recommended). You can install Rust via [rustup](https://rustup.rs/).

### Building from source

1.  Clone the repository:
    ```bash
    git clone https://github.com/vschwaberow/rustscrub.git
    cd rustscrub
    ```

2.  Build the project:
    ```bash
    cargo build
    ```
    For a release build (optimized):
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/debug/rustscrub` or `target/release/rustscrub`.

3.  (Optional) Install the binary to a directory in your PATH:
    ```bash
    cp target/release/rustscrub ~/.local/bin/  # Adjust path as needed, e.g., /usr/local/bin
    ```

### Via Crates.io (Once Published)

Once the crate is published to [crates.io](https://crates.io/), you will be able to install it using:
```bash
cargo install rustscrub
```

## Usage

The primary way to use `rustscrub` is by providing an input file. The processed output can either be directed to an output file or, if no output file is specified, it might print to standard output (this behavior should be clarified as development progresses).

Basic syntax:
```bash
rustscrub <input_file_path> [OPTIONS]
```

**Arguments:**

*   `<input_file_path>`: (Required) The path to the file that needs to be processed.

**Options:**

*   `-o, --output <output_file_path>`: Specifies the path for the output file. If not provided, the behavior might be to print to standard output.
*   `-H, --header-lines <number>`: Specifies the number of header lines to preserve from the input file. Defaults to `0`.
*   `-v, --verbose`: Enables verbose output, providing more details about the scrubbing process.
*   `-d, --dry-run`: Performs a dry run. It will show what would be changed without actually modifying any files or producing an output file.
*   `--help`: Displays a help message with all available commands and options.

**Examples:**

1.  Scrub comments from `source.rs` and save the result to `scrubbed_source.rs`:
    ```bash
    rustscrub source.rs -o scrubbed_source.rs
    ```

2.  Scrub comments from `main.rs`, preserving the first 5 header lines, and show verbose output:
    ```bash
    rustscrub main.rs -H 5 -v -o cleaned_main.rs
    ```

3.  Perform a dry run on `utils.rs` to see what would be scrubbed:
    ```bash
    rustscrub utils.rs -d
    ```

4.  Display help information:
    ```bash
    rustscrub --help
    ```

## Development

### Dependencies

This project uses `clap` for command-line argument parsing.
```toml
[dependencies]
clap = { version = "4.4.8", features = ["derive"] }
```

### Running Tests
```bash
cargo test
```

### Linting and Formatting
It's recommended to use `rustfmt` for formatting and `clippy` for linting.
```bash
cargo fmt
cargo clippy
```

## Contributing

Contributions are welcome! If you have an idea for a new feature, a bug fix, or improvements to the documentation, please feel free to:

1.  Fork the repository.
2.  Create a new branch (`git checkout -b feature/your-feature-name`).
3.  Make your changes.
4.  Commit your changes (`git commit -am 'Add some feature'`).
5.  Push to the branch (`git push origin feature/your-feature-name`).
6.  Open a Pull Request.

Please ensure your code adheres to the existing style and that all tests pass.

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.
(You will need to create a `LICENSE` file containing the text of the MIT license).

## Keywords

cli, rust, scrub, security, privacy, data sanitization

## Categories

Command-line utilities

## Installation

### Prerequisites

*   Rust programming language (latest stable version recommended). You can install Rust via [rustup](https://rustup.rs/).

### Building from source

1.  Clone the repository:
    ```bash
    git clone https://github.com/vschwaberow/rustscrub.git
    cd rustscrub
    ```

2.  Build the project:
    ```bash
    cargo build
    ```
    For a release build (optimized):
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/debug/rustscrub` or `target/release/rustscrub`.

3.  (Optional) Install the binary to a directory in your PATH:
    ```bash
    cp target/release/rustscrub ~/.local/bin/  # Adjust path as needed, e.g., /usr/local/bin
    ```

### Via Crates.io (Once Published)

Once the crate is published to [crates.io](https://crates.io/), you will be able to install it using:
```bash
cargo install rustscrub
```

## Usage

The primary way to use `rustscrub` is by providing an input file. The processed output can either be directed to an output file or, if no output file is specified, it might print to standard output (this behavior should be clarified as development progresses).

Basic syntax:
```bash
rustscrub <input_file_path> [OPTIONS]
```

**Arguments:**

*   `<input_file_path>`: (Required) The path to the file that needs to be processed.

**Options:**

*   `-o, --output <output_file_path>`: Specifies the path for the output file. If not provided, the behavior might be to print to standard output.
*   `-H, --header-lines <number>`: Specifies the number of header lines to preserve from the input file. Defaults to `0`.
*   `-v, --verbose`: Enables verbose output, providing more details about the scrubbing process.
*   `-d, --dry-run`: Performs a dry run. It will show what would be changed without actually modifying any files or producing an output file.
*   `--help`: Displays a help message with all available commands and options.

**Examples:**

1.  Scrub comments from `source.rs` and save the result to `scrubbed_source.rs`:
    ```bash
    rustscrub source.rs -o scrubbed_source.rs
    ```

2.  Scrub comments from `main.rs`, preserving the first 5 header lines, and show verbose output:
    ```bash
    rustscrub main.rs -H 5 -v -o cleaned_main.rs
    ```

3.  Perform a dry run on `utils.rs` to see what would be scrubbed:
    ```bash
    rustscrub utils.rs -d
    ```

4.  Display help information:
    ```bash
    rustscrub --help
    ```

## Development

### Dependencies

This project uses `clap` for command-line argument parsing.
```toml
[dependencies]
clap = { version = "4.4.8", features = ["derive"] }
```

### Running Tests
```bash
cargo test
```

### Linting and Formatting
It's recommended to use `rustfmt` for formatting and `clippy` for linting.
```bash
cargo fmt
cargo clippy
```

## Contributing

Contributions are welcome! If you have an idea for a new feature, a bug fix, or improvements to the documentation, please feel free to:

1.  Fork the repository.
2.  Create a new branch (`git checkout -b feature/your-feature-name`).
3.  Make your changes.
4.  Commit your changes (`git commit -am 'Add some feature'`).
5.  Push to the branch (`git push origin feature/your-feature-name`).
6.  Open a Pull Request.

Please ensure your code adheres to the existing style and that all tests pass.

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.
(You will need to create a `LICENSE` file containing the text of the MIT license).

## Keywords

cli, rust, scrub, security, privacy, data sanitization

## Categories

Command-line utilities

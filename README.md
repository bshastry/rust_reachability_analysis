# Public API Extractor

This repository contains a Rust application that scans a specified codebase directory and extracts all public types and functions. It is designed to help developers quickly identify the public API surface of their Rust projects.

## Features

- Recursively scans a specified directory for Rust source files (`.rs`).
- Extracts public structs, traits, and functions.
- Outputs the results in a clear and organized format.

## Getting Started

### Prerequisites

- Rust (1.50 or later) installed on your machine. You can install Rust using [rustup](https://rustup.rs/).

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/bshastry/rust_reachability_analysis.git
   cd rust_reachability_analysis
   ```

2. Build the application:

   ```bash
   cargo build --release
   ```

### Usage

1. Modify the `path` variable in `src/main.rs` to point to the directory of the Rust codebase you want to analyze. For example:

   ```rust
   let path = "../lighthouse"; // Change this to your codebase path
   ```

2. Run the application:

   ```bash
   cargo run --release
   ```

3. The output will display all public types and functions found in the specified directory.

### Example Output

```
Public Types:
- MyStruct
- MyTrait

Public Functions:
- MyTrait::my_function
```

## Code Structure

- `src/main.rs`: The main application code that handles directory traversal and parsing of Rust files.
- `Cargo.toml`: The configuration file for the Rust package.

## Dependencies

This application uses the following dependencies:

- `syn`: A parsing library for Rust code.

You can find the full list of dependencies in the `Cargo.toml` file.

## Contributing

Contributions are welcome! If you have suggestions for improvements or new features, please open an issue or submit a pull request.

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Make your changes and commit them (`git commit -m 'Add new feature'`).
4. Push to the branch (`git push origin feature-branch`).
5. Open a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Thanks to the Rust community for their support and contributions to the ecosystem.
- Special thanks to the authors of the `syn` library for providing a powerful tool for parsing Rust code.

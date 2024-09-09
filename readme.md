# Can I Use CLI

A simple Rust-based CLI tool that fetches browser compatibility data for web technologies from the [Can I Use](https://caniuse.com) API. This tool allows users to search for feature support across various browsers and outputs the result in a human-readable format.

## Table of Contents
- [Can I Use CLI](#can-i-use-cli)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Example](#example)
  - [Contributing](#contributing)
  - [License](#license)

## Features

- Fetches feature IDs based on a search term.
- Retrieves detailed browser support data for the selected feature IDs.
- Outputs browser compatibility in a tabular form with appropriate emojis for better readability.
- Offers additional metadata like description, spec URL, and MDN URL.

## Prerequisites

Before you begin, ensure you have met the following requirements:

- **Rust**: Install Rust from [rust-lang.org](https://www.rust-lang.org/).
- **Cargo**: Cargo is the Rust package manager, which comes installed with Rust.

## Installation

To install dependencies and set up the project, run the following commands:

```sh
git clone https://github.com/yourusername/caniuse_cli.git
cd caniuse_cli
cargo build --release
```

## Usage

To use the CLI tool, run:

```sh
./target/release/caniuse_cli <search_term>
```

Where `<search_term>` is the feature or technology you want to search for in the Can I Use database.

### Example

```sh
./target/release/caniuse_cli websocket
```

This command will search for features related to "websocket" and display the corresponding browser compatibility data.

The output will look something like this:

```sh
🔍 Search term: websocket

🏷️  Selected feature IDs:
  • mdn-api_websocketstream

📊 Feature data:

🔹 Feature 1:
  📌 Title: WebSocketStream API
  📝 Description: 
  📘 Spec: 
  🔗 MDN URL: 

  🖥️  Browser Compatibility:
  +------------+---------+-------+
  | browser    | support | notes |
  +------------+---------+-------+
  | ✅ chrome  | 124     |       |
  | ❌ firefox | false   |       |
  | ✅ edge    | 124     |       |
  | ❌ safari  | false   |       |
  | ...        | ...     | ...   |
  +------------+---------+-------+

  ℹ️  Extra information:
    • path: "api/WebSocketStream.json"
    • amountOfBrowsersWithData: 13
    • mdnStatus: {"deprecated":false,"experimental":true,"standard_track":true}
    • children: [{"id":"mdn-api_websocketstream_websocketstream","title":"WebSocketStream API: `WebSocketStream()` constructor"}, ... ]

```

### Debug Mode

You can enable debug mode to get more detailed logs about the HTTP requests and responses:

```sh
RUST_LOG=debug ./target/release/caniuse_cli <search_term>
```

## Contributing

Contributions are always welcome! Please follow these steps:

1. Fork the repository.
2. Create a new feature branch (`git checkout -b feature/awesome-feature`).
3. Commit your changes (`git commit -am 'Add an awesome feature'`).
4. Push to the branch (`git push origin feature/awesome-feature`).
5. Create a new Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
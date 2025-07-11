# LocalShare

LocalShare is a Rust-based application designed to facilitate secure and efficient file sharing on your local network.

## Features

- Fast and secure file transfers
- Simple command-line interface
- Cross-platform support

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)

### Installation

Clone the repository and build the project:

```sh
git clone https://github.com/erbekin/localshare.git
cd localshare
cargo build --release
cargo install --path . # if you like to install
```

### Usage

Run the application:

```sh
cargo run -- <options>

```
Replace `<options>` with your desired command-line arguments.
> **Note:**  
> In first run in a new uploads directory, pass `-x` flag to localshare, this will extract `.html`files to static folder.
> For a full list of available options, run:
> 
> ```sh
> cargo run -- --help
> ```

## Contributing

Contributions are welcome! Please open issues or submit pull requests.

## License

This project is licensed under the MIT License.
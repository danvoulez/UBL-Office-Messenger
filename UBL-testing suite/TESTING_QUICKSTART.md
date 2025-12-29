# Office Testing Quick Start

Get started with testing in 5 minutes. 

## Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install test tools
cargo install cargo-tarpaulin  # Coverage
cargo install cargo-make       # Task runner
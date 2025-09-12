#!/usr/bin/env bash
set -euo pipefail

# Load .env if present
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi

# Example: run the daemon (adjust the binary path/args as needed)
cargo run -p daemon -- --config ./configs/phase1-observe-only.toml

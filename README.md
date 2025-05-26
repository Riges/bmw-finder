# BMW Finder 🚗

[![Rust](https://img.shields.io/badge/Rust-🦀-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Test Suite](https://github.com/Riges/bmw-finder/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/Riges/bmw-finder/actions/workflows/ci.yml)

Find new and used BMWs, display their details, and sort them by price.

## Features

- 🔎 **Search** for new or used vehicles
- 💰 **Sort** results by price
- 📜 **List** formatted car details

## Quick Start

```bash
git clone https://github.com/your_username/bmw_finder.git
cd bmw_finder
cargo run
```

## Run Examples

```bash
# Default: new vehicles, model iX2_U10E
cargo run

# Search for used vehicles
cargo run -- --used

# Specify one or more models
cargo run -- --model i4_G26E --model iX1_U11E --model iX2_U10E

# Limit the number of results
cargo run -- --limit 5
# or short form
cargo run -- -l 5

# Filter by equipment/pack name (repeatable)
cargo run -- --equipment-name "Pack M Sport" --equipment-name "Pack Innovation"

# Combined example
cargo run -- --model iX1_U11E --used -l 3 \
  --equipment-name "Pack M Sport"

# Output only the number of vehicles and search parameters (default)
cargo run -- --model iX1_U11E --used -l 3 --equipment-name "Pack M Sport"

# Output full vehicle details as text
cargo run -- --model iX1_U11E --used -l 3 --equipment-name "Pack M Sport" --output text

# Output filtered vehicles as JSON
cargo run -- --model iX1_U11E --used -l 3 --equipment-name "Pack M Sport" --output json
```

## Options

| Flag                      | Description                                         | Default    |
| ------------------------- | --------------------------------------------------- | ---------- |
| `--model <MODEL>`         | Models to search for (repeatable)                   | `iX2_U10E` |
| `--used`                  | Search for used vehicles                            | `false`    |
| `-l`, `--limit <NUMBER>`  | Maximum number of results                           | none       |
| `--equipment-name <NAME>` | Filter by equipment/pack name (repeatable, by name) | none       |
| `--output <MODE>`         | Output mode: `ui` (default), `text`, or `json`      | `ui`       |

---

Made with ❤️ using [Rust](https://www.rust-lang.org/).

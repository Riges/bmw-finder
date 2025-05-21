# BMW Finder 🚗

[![Rust](https://img.shields.io/badge/Rust-🦀-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Find new and used BMWs, display their details, and sort them by price.

## Features

- 🔎 **Search** for new or used cars
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
# Default: new cars, model iX2_U10E
cargo run

# Search for used cars
cargo run -- --used

# Specify one or more models
cargo run -- --model i4_G26E --model iX1_U11E --model iX2_U10E

# Limit the number of results
cargo run -- --limit 5
# or short form
cargo run -- -l 5

# Filter by equipment (repeatable)
cargo run -- --filter-equipment "Pack M Sport" --filter-equipment "Pack Innovation"

# Combined example
cargo run -- --model iX1_U11E --used -l 3 \
  --filter-equipment "Pack M Sport"
```

## Options

| Flag                             | Description                       | Default    |
| -------------------------------- | --------------------------------- | ---------- |
| `--model <MODEL>`                | Models to search for (repeatable) | `iX2_U10E` |
| `--used`                         | Search for used cars              | `false`    |
| `-l`, `--limit <NUMBER>`         | Maximum number of results         | none       |
| `--filter-equipment <EQUIPMENT>` | Filter by equipment (repeatable)  | none       |

---

Made with ❤️ using [Rust](https://www.rust-lang.org/).

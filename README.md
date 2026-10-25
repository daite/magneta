![build](https://github.com/daite/magneta/workflows/Rust/badge.svg)
# Magneta

A fast, extensible CLI tool for searching torrents across multiple torrent sites with failover support and pluggable site parsers.

## Background

> Korean torrent sites are often filled with aggressive advertisements, popups, and scripts.  
> Searching for a single show can take many minutes and multiple clicks.  

## Features

- 🔍 Search torrents by keyword
- 🌐 Supports multiple torrent sites with **failover fallback** (Multiplexer N→1)
- 🧩 Pluggable site parsers (e.g., TorrentTop, TorrentSome, TorrentRJ)
- 🛠️ Built with pure Rust, 100% Rust project
- 🧪 Full offline unit testing (HTML samples, GitHub Actions CI)
- 📦 Pretty table CLI output
- 🩺 `doctor` subcommand to check site health
- 🪵 Integrated logging (`RUST_LOG=warn` for errors)

## Requirements

- Rust (stable)
- Cargo

Install Rust:  
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build and Run

### Example

```bash
make run search "하얀 마피아의 시대"
```

<details>
<summary>⠏ ✅ Search completed! (click to expand)</summary>

<pre>
+------------------------------------------------------------------------------------+--------------------------------------------------------------+
| Title                                                                              | Magnet                                                       |
+------------------------------------------------------------------------------------+--------------------------------------------------------------+
| 닥터X - 하얀 마피아의 시대.E06.261024.720p-NEXT                                          | magnet:?xt=urn:btih:d5559d0083f3438c6aa3401a2eba324482a05882 |
| 닥터X - 하얀 마피아의 시대.E05.261023.720p-NEXT                                          | magnet:?xt=urn:btih:9e0922dfb63e1957a3d4cb66be0f599087a2b6bb |
| 닥터X - 하얀 마피아의 시대.E04.261017.720p-NEXT                                          | magnet:?xt=urn:btih:41577f476ebbbfa6d04b6353dcb538de5a2caccf |
| 닥터X - 하얀 마피아의 시대.E03.261016.720p-NEXT                                          | magnet:?xt=urn:btih:91635bee74e8077e20874ec51d7006cc33d08792 |
| 닥터X - 하얀 마피아의 시대.E02.261010.720p-NEXT                                          | magnet:?xt=urn:btih:bb6c9702d780f1425d0f62a060e2a665a0cca8db |
| 닥터X - 하얀 마피아의 시대.E01.261009.720p-NEXT                                          | magnet:?xt=urn:btih:133121101010fbfe434e0fe29f232b90924fe046 |
+------------------------------------------------------------------------------------+--------------------------------------------------------------+
</pre>

</details>

```bash
# Build the project
make build

# Run the project
make run search "keyword"

# Example
make run search "하얀 마피아의 시대"

# Run diagnostics
make run doctor

# Run all tests
make test
```

Or using Cargo directly:

```bash
cargo run -- search "keyword"
cargo run -- doctor
cargo test
```

## Project Structure

```text
magneta/
.
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── Makefile
├── README.md
├── config.toml
├── src
│   ├── lib.rs
│   ├── main.rs
│   ├── site_registry.rs
│   └── sites
│       ├── mod.rs
│       ├── torrenttop.rs
│       ├── torrentsome.rs
│       └── torrentrj.rs
└── tests
    ├── data
    │   ├── torrenttop
    │   └── torrentsome
    └── torrenttop_test.rs
```

## Site Diagnostics

Use the `doctor` subcommand to check which torrent sites are currently reachable:

```bash
cargo run -- doctor
```

**Example Output:**

```
🔍 Running diagnostics for torrent sites:
+-------------+----------------------------+----------------------+
| Site Name   | Site URL                   | Status               |
+-------------+----------------------------+----------------------+
| torrenttop  | https://torrenttop151.com  | ❌ connection failed |
| torrentrj   | https://torrentrj199.com   | ✅ 200 OK            |
| torrentsome | https://torrentsome192.com | ❌ connection failed |
+-------------+----------------------------+----------------------+
```

## Testing

This project uses **offline HTML samples** for testing:

```bash
make test
```

No external internet access required.  
Tests are fully reproducible even in CI environments.

## Contributing

Pull requests are welcome!  
Please follow Rust's code style and run:

```bash
make fmt
make lint
```
before submitting a PR.

## License

This project is licensed under the MIT License.

# AGENTS.md

`selune` is a Rust crate that models AI providers and models in the
Catwalk format (https://github.com/charmbracelet/catwalk). It ships a small
embedded catalog plus a client that fetches a hosted catalog JSON. This file
orients AI agents (and humans) to the layout and conventions.

## What this crate is

- **Types** (`src/types.rs`) — the wire contract, mirroring Catwalk's
  `provider.go`: `Provider`, `Model`, `ModelOptions`, `ProviderType`, and the
  `InferenceProvider` id wrapper. Field names are snake_case and match the JSON
  exactly, so serde needs no per-field renames.
- **Client** (`src/client.rs`) — a blocking HTTP client that fetches
  `/v2/providers` from a base URL. The base URL is a **placeholder**
  (`DEFAULT_URL`); the user hosts the configs themselves and points the client
  at them via the `CATALOG_URL` env var or `Client::new_with_url`.
- **Embedded catalog** (`src/embedded.rs`) — the sample provider configs in
  `configs/*.json` compiled in via `include_str!`, exposed as `embedded::all()`.
- **Python generator** (`python/generate.py`) — reads `configs/*.json` and
  emits a combined `catalog.json` (a JSON array of providers) that the user
  hosts. This is the artifact the Rust client fetches.

## Layout

| Path | Role |
|---|---|
| `src/types.rs` | `Provider`/`Model`/`ModelOptions`/`ProviderType`/`InferenceProvider` + lookup helpers |
| `src/client.rs` | `Client` (fetch providers), `ClientError`, `DEFAULT_URL` |
| `src/embedded.rs` | `embedded::all()` — parses the embedded `configs/*.json` |
| `src/lib.rs` | Barrel re-exports |
| `src/tests.rs` | Unit tests (embedded parse, default-model validity, serde round-trip, lookup) |
| `configs/*.json` | One provider config per file, in Catwalk format |
| `python/generate.py` | Static JSON generator → `catalog.json` |
| `Cargo.toml` | Standalone crate (has its own `[workspace]`; not a member of the parent workspace) |

## Conventions

- **Standalone crate.** `Cargo.toml` declares an empty `[workspace]` table and
  explicit package metadata so it builds on its own and can be split into its
  own repo. Do not add it to the parent workspace's `members`.
- **Catwalk parity.** Keep the JSON shape and field names identical to Catwalk
  so configs are interchangeable. When Catwalk adds a field, mirror it here.
- **Serde cases.** Struct fields are snake_case and match JSON directly. The
  `ProviderType` enum uses `#[serde(rename_all = "kebab-case")]` with variants
  named so kebab-case yields the exact Catwalk strings (`Openai` → `openai`,
  `OpenaiCompat` → `openai-compat`, `Openrouter` → `openrouter`). Prefer
  `rename_all` over per-field `#[serde(rename = ...)]`.
- **Optional fields** use `Option<T>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`; collections use `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.
- **Errors**: `thiserror` enum (`ClientError`). Box large variants (e.g.
  `reqwest::Error`) to keep the error type small.
- **Comments**: do not add comments unless requested.
- **Formatting/lint**: `cargo fmt` and `cargo clippy --all-targets` must pass.

## Build & test

```sh
cargo build
cargo test
cargo clippy --all-targets
cargo fmt --check
```

## Adding a provider config

1. Add `configs/<id>.json` in Catwalk format (see `configs/anthropic.json`).
2. Add it to `src/embedded.rs` (`include_str!` + the `all()` array) and to the
   `ORDER` list in `python/generate.py`.
3. Add a test in `src/tests.rs` if it exercises new behavior.
4. Regenerate the hosted artifact: `python3 python/generate.py`.

## Hosting

The user hosts the generated `catalog.json` themselves. The Rust client's
`DEFAULT_URL` is a placeholder; point it at the real URL (or set `CATALOG_URL`)
once hosting is in place.

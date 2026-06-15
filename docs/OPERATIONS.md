# Rumble Canvas — local operations runbook

## Scope

This runbook covers the deployable/local MVP slice: Canvas produces local structured truth, immutable packages, planning-only handoffs, and delegates validation/planning to `cosmatic`.

Canvas still does **not** execute implementation work.

## Prerequisites

- Rust toolchain with Cargo.
- `cosmatic` on `PATH` for validation and dry-run planning.

For local development with the sibling repo:

```bash
cd ../cos-matic
cargo build -p cos-matic-cli
export PATH="$PWD/target/debug:$PATH"
```

## Local end-to-end flow

```bash
cargo run -p rumble-canvas -- workspace sample --store target/canvas.json
cargo run -p rumble-canvas -- package build --store target/canvas.json --out target/package.json
cargo run -p rumble-canvas -- handoff build --store target/canvas.json --out target/handoff.json
cargo run -p rumble-canvas -- handoff validate --store target/canvas.json --json
cargo run -p rumble-canvas -- handoff plan --store target/canvas.json --json
cargo run -p rumble-canvas -- workspace show --store target/canvas.json
```

Expected final state:

```text
packages: 1
handoffs: 1
validation reports: 1
dry-run plans: 1
```

## Safety guarantees

- `handoff plan` always calls `cosmatic handoff plan ... --dry-run`.
- No Canvas command creates branches, writes target repositories, starts agents, or deploys code.
- Handoff generation rejects `allow_execution = true` through Canvas-side invariants.
- `cosmatic` remains the authority for Bolt validation/planning.

## Release checks

```bash
cargo fmt --check
cargo test --workspace
cargo build --release -p rumble-canvas
```

Optional, with `cosmatic` available:

```bash
cargo run -p rumble-canvas -- workspace sample --store target/release-check/canvas.json
cargo run -p rumble-canvas -- package build --store target/release-check/canvas.json --out target/release-check/package.json
cargo run -p rumble-canvas -- handoff build --store target/release-check/canvas.json --out target/release-check/handoff.json
cargo run -p rumble-canvas -- handoff validate --store target/release-check/canvas.json --json
cargo run -p rumble-canvas -- handoff plan --store target/release-check/canvas.json --json
```

## Deployment modes

### CLI artifact

The currently deployable unit is a CLI binary:

```bash
cargo build --release -p rumble-canvas
install -m 0755 target/release/rumble-canvas /usr/local/bin/rumble-canvas
```

### Future UI

A Dioxus UI can consume the same crates:

- `rumble-canvas-domain` for structured truth;
- `rumble-canvas-package` for package hashing/immutability;
- `rumble-canvas-handoff` for planning-only payloads;
- `rumble-canvas-store` for local persistence.

Do not add UI execution controls. UI actions must call the same planning-only handoff path.

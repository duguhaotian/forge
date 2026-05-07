# forge

Forge is an agent-oriented compute broker and workspace runtime. It provides a unified CLI for discovering, leasing, and operating compute resources across local machines, VMs, and remote hosts.

It is designed for AI coding agents: compute should be a programmable resource that an agent can discover, inspect, allocate, and operate through stable commands.

## Documentation

- `docs/design.md` — architecture, data model, provider boundaries, and extension points
- `docs/usage.md` — installation, SSH node management, import formats, and command examples
- `README_CN.md` — Chinese overview and quick start

## Current Scope

The current Rust implementation focuses on SSH Provider node management:

- Initialize local Forge state
- Add, import, list, show, and remove SSH nodes
- Test node connectivity
- Inspect host capability information
- Execute remote commands through SSH

`lease` and `workspace` are present as command entries, but their lifecycle logic is still pending.

## Install

```bash
cargo build
cargo run -- --help
```

To install the CLI from this checkout:

```bash
cargo install --path .
forge --help
```

## Quick Start

```bash
forge init
forge provider list
forge node list ssh
```

Add a key-based SSH node:

```bash
forge node add ssh dev-key \
  --host 1.2.3.5 \
  --user ubuntu \
  --auth key-pair \
  --key ~/.ssh/id_ed25519
```

Verify and inspect it:

```bash
forge node ping ssh dev-key
forge node inspect ssh dev-key
forge node exec ssh dev-key -- uname -a
```

## Command Overview

- `forge init` — create `~/.forge/` and default provider files
- `forge provider list` — list available providers
- `forge node add ssh` — add one SSH node
- `forge node import ssh` — import SSH nodes from YAML or JSON
- `forge node list [ssh]` — list configured nodes
- `forge node show ssh <name>` — print one node as YAML with secrets redacted
- `forge node remove ssh <name>` — remove one configured node
- `forge node ping ssh <name>` — verify SSH connectivity
- `forge node inspect ssh <name>` — collect hostname, OS, CPU, memory, disk, and uptime
- `forge node exec ssh <name> -- <command>` — run a remote command and pass through its exit code

## Storage

Forge stores local state under `~/.forge/`:

```text
~/.forge/
├── config.yaml
└── providers/
    └── ssh/
        └── nodes.yaml
```

SSH Provider owns its node store, so future providers can use independent configuration sources and storage formats.

## SSH Authentication

Password authentication depends on `sshpass`. Key and certificate authentication depend on the system `ssh` client.

Supported `password_ref` forms include:

- `env:NAME`
- `prompt`
- `plain:VALUE`
- any other literal string

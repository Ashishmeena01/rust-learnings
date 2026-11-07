# Rust learning projects

This repository contains two small Rust projects:

- `terminal-ui`: a Ratatui calendar with keyboard navigation, themes, and an event inspector.
- `macro-learning`: a procedural-macro library and a binary that demonstrates using it.

Each project can be built from its own directory with Cargo. See its README for project-specific instructions.

The root `Cargo.toml` is a workspace manifest. From the repository root, use `cargo build --workspace` to build all workspace packages, or pass `-p terminal-ui` / `-p macro-test-1` to build one application.

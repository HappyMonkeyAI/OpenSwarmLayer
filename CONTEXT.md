# TensorSwarm Context

## Purpose

TensorSwarm is an open-source, tensor-native distribution engine for GGUF and Safetensors artifacts. It is intended to reduce repeated centralized downloads while retaining HTTPS compatibility and ordinary runtime file formats.

## Current state

The local end-to-end MVP is accepted with limitations. M0 through M3 and M5
are complete; M4's proxy, LAN, WebSeed, materialization, and verification paths
are implemented and tested. The post-MVP M6 track has an authenticated
`ts-daemon` and a Tauri v2 shell that starts the Rust engine automatically on
desktop launch, uses an app-data cache and per-session token, and exposes a
visible retry path after startup failure. Windows release/API and NSIS build
smokes pass; Linux DEB/RPM/AppImage artifacts build and DEB install/remove has
been exercised in WSL, but native Linux graphical acceptance is incomplete.
In-app engine recovery and close-to-background behavior have live Windows UI
evidence; explicit tray Quit remains unverified. Model import/sharing is still
in progress. Transparent `HTTP_PROXY`/`HTTPS_PROXY` rewriting remains deferred
as post-MVP work. See `TASKS.md` and `PROGRESS.md` for acceptance boundaries.

## Agents Protocol adoption

The repository uses `AGENTS.md` for workflow and acceptance rules, `MCP.md` for portable intent routing, `BOOTSTRAP.md` for live MCP discovery, and `docs/adr/0002` through `0005` for the adopted protocol decisions. TensorSwarm remains the application authority; protocol documents must not overwrite its implementation roadmap or existing ADR-0001.

## MVP constraints

- Rust and Tokio are the implementation baseline.
- Integrity must be verifiable before data enters durable storage.
- The first network target is two nodes on one LAN.
- The first client compatibility target is ordinary HTTP and HTTP Range behavior.
- Existing runtime compatibility takes priority over transparent lazy mmap behavior.
- MVP uses fixed-size chunks and SHA-256; dynamic quantization is deferred.

## What not to do yet

- Do not build a global relay/bootstrap service before the local vertical slice works.
- Do not add a custom runtime or inference engine.
- Do not claim tensor identity alone can reconstruct a file; the manifest must include a file layout recipe.
- Do not commit or cache unverified remote bytes.

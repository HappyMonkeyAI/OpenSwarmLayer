# TensorSwarm Context

## Purpose

TensorSwarm is an open-source, tensor-native distribution engine for GGUF and Safetensors artifacts. It is intended to reduce repeated centralized downloads while retaining HTTPS compatibility and ordinary runtime file formats.

## Current state

The local end-to-end MVP is accepted with limitations. M0 through M3 and M5
are complete; M4's proxy, LAN, WebSeed, materialization, and verification paths
are implemented and tested. The post-MVP M6 track has an authenticated
`ts-daemon` and a Tauri v2 shell that starts the Rust engine automatically on
desktop launch, uses an app-data cache and per-session token, and exposes a
visible retry path after startup failure. Windows release/API and NSIS install,
import, preparation, persistence, uninstall, occupied-port recovery,
close-to-background, and native tray-menu Quit evidence pass. The installed
Windows UI also imported and prepared a synthetic GGUF fixture byte-identically.
Ubuntu WSLg packaged-DEB acceptance verified live UI state, Safetensors import,
restart persistence, and an independent CLI P2P fetch with matching payload
bytes/hash; this is not native Linux package/tray acceptance. A native Deepin
DEB built from the current tracked source and passed metadata/content/dependency
checks and a current-source DEB build passed on 2026-09-27, including dependency
and GLIBC 2.38 preflight; installation/removal remains sudo-gated. A later
isolated extracted-package launch could not be read back after SSH to Deepin
timed out. Native Linux package install, runtime, and tray behavior remain open.
M6-07 import/verify/prepare/
share acceptance is complete for the verified slices. Transparent
`HTTP_PROXY`/`HTTPS_PROXY` rewriting remains deferred as post-MVP work. See
`TASKS.md` and `PROGRESS.md` for exact evidence boundaries.

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
- Do not launch a public model directory or global tracker before rights/provenance declarations, abuse policy, reporting/takedown and appeal handling, privacy rules, and operational controls are designed and reviewed.
- Do not treat a matching hash, supported model format, or publisher signature as proof of safety, legality, or redistribution rights.
- Do not add a custom runtime or inference engine.
- Do not claim tensor identity alone can reconstruct a file; the manifest must include a file layout recipe.
- Do not commit or cache unverified remote bytes.

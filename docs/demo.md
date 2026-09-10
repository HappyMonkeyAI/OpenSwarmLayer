# TensorSwarm two-node demo

This guide uses a clean checkout and a small local model fixture. The commands use
separate terminals so the provider node remains running while the proxy is tested.

## Verify the checkout

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

## Build a manifest and seed a node

Create a supported GGUF or Safetensors model as `model.safetensors`, then create a
manifest and a verified object store:

```text
target/debug/ts-cli manifest model.safetensors model.tswarm
target/debug/ts-cli node model.tswarm .tswarm-cache
```

The node publishes the manifest and tensor provider keys and listens on its libp2p
TCP and QUIC addresses. Record the peer ID and a reachable listen address from the
node output.

## Run the manifest proxy

In a second terminal, configure the proxy with the provider peer identity and its
full libp2p multiaddress:

```text
target/debug/ts-cli proxy-manifest 127.0.0.1:9090 model.tswarm .tswarm-cache https://model.example/model.safetensors <peer-id> <multiaddress>
```

Request the manifest path directly. A missing local chunk is fetched from the LAN
peer, verified, stored in the local CAS, and returned to the client:

```text
curl -v -H "Range: bytes=0-1048575" http://127.0.0.1:9090/file/model.safetensors -o model.part
```

The proxy currently supports explicit `/file/<manifest-path>` URLs. It does not yet
rewrite arbitrary origin URLs supplied through `HTTP_PROXY` or `HTTPS_PROXY`.

## Deterministic acceptance test

The clean-checkout equivalent of the two-process transfer and proxy response check is:

```text
cargo test -p ts-proxy manifest_router_fetches_missing_tensor_from_lan_peer -- --nocapture
cargo test -p ts-p2p two_local_nodes_transfer_a_verified_chunk -- --nocapture
```

These tests create loopback nodes, transfer verified content, and assert the HTTP
status, range headers, and returned bytes. A physical LAN run should additionally
record the node peer ID, listen address, and client host details.

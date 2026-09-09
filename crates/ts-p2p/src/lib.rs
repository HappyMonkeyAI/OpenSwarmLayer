//! TensorSwarm peer protocol primitives.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use ts_core::{sha256, Hash32, Manifest};

#[derive(libp2p::swarm::NetworkBehaviour)]
#[behaviour(prelude = "libp2p::swarm::derive_prelude")]
pub struct LanBehaviour {
    pub identify: libp2p::identify::Behaviour,
    pub mdns: libp2p::mdns::tokio::Behaviour,
    pub kad: libp2p::kad::Behaviour<libp2p::kad::store::MemoryStore>,
    pub request_response: libp2p::request_response::cbor::Behaviour<PeerRequest, PeerResponse>,
}

pub type LanSwarm = libp2p::Swarm<LanBehaviour>;

#[derive(Clone, Debug, Default)]
pub struct ChunkProvider {
    manifests: HashMap<Hash32, Vec<u8>>,
    chunks: HashMap<(Hash32, u32), Vec<u8>>,
}

impl ChunkProvider {
    pub fn from_manifest(
        store: &ts_store::ObjectStore,
        manifest: &Manifest,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(manifest.verify_root(), "manifest root verification failed");
        let mut provider = Self::default();
        provider.insert_manifest(manifest.root, manifest.to_bytes());
        for tensor in &manifest.tensors {
            for chunk in &tensor.chunks {
                let bytes = store.get(chunk.hash)?;
                anyhow::ensure!(
                    bytes.len() as u64 == chunk.length,
                    "stored chunk length mismatch"
                );
                provider
                    .insert_chunk(tensor.tensor_hash, chunk.index, chunk.hash, bytes)
                    .map_err(|error| anyhow::anyhow!("invalid stored chunk: {error:?}"))?;
            }
        }
        Ok(provider)
    }

    pub fn insert_manifest(&mut self, hash: Hash32, bytes: Vec<u8>) {
        self.manifests.insert(hash, bytes);
    }

    pub fn insert_chunk(
        &mut self,
        tensor_hash: Hash32,
        index: u32,
        expected: Hash32,
        bytes: Vec<u8>,
    ) -> Result<(), PeerErrorCode> {
        if sha256(&bytes) != expected {
            return Err(PeerErrorCode::InvalidRequest);
        }
        self.chunks.insert((tensor_hash, index), bytes);
        Ok(())
    }

    pub fn respond(&self, request: PeerRequest) -> PeerResponse {
        match request {
            PeerRequest::GetManifest { manifest_hash } => self
                .manifests
                .get(&manifest_hash)
                .map(|bytes| PeerResponse::Manifest {
                    manifest_hash,
                    bytes: bytes.clone(),
                })
                .unwrap_or(PeerResponse::Error {
                    request_id: None,
                    code: PeerErrorCode::NotFound,
                }),
            PeerRequest::GetChunks {
                tensor_hash,
                chunks,
            } => {
                if chunks.is_empty() || chunks.len() > MAX_CHUNKS_PER_REQUEST {
                    return PeerResponse::Error {
                        request_id: None,
                        code: PeerErrorCode::TooLarge,
                    };
                }
                let Some(request) = chunks.into_iter().next() else {
                    return PeerResponse::Error {
                        request_id: None,
                        code: PeerErrorCode::InvalidRequest,
                    };
                };
                match self.chunks.get(&(tensor_hash, request.chunk_index)) {
                    Some(payload)
                        if payload.len() <= MAX_CHUNK_BYTES
                            && sha256(payload) == request.expected_hash =>
                    {
                        PeerResponse::Chunk {
                            request_id: request.request_id,
                            tensor_hash,
                            chunk_index: request.chunk_index,
                            payload: payload.clone(),
                            payload_hash: request.expected_hash,
                        }
                    }
                    Some(_) => PeerResponse::Error {
                        request_id: Some(request.request_id),
                        code: PeerErrorCode::Internal,
                    },
                    None => PeerResponse::Error {
                        request_id: Some(request.request_id),
                        code: PeerErrorCode::NotFound,
                    },
                }
            }
            PeerRequest::Have { hashes } => PeerResponse::Have {
                hashes: hashes
                    .into_iter()
                    .filter(|hash| {
                        self.chunks.keys().any(|(tensor, _)| tensor == hash)
                            || self.manifests.contains_key(hash)
                    })
                    .collect(),
            },
            PeerRequest::Cancel { request_id } => PeerResponse::Error {
                request_id: Some(request_id),
                code: PeerErrorCode::Busy,
            },
        }
    }
}

pub fn build_lan_swarm() -> anyhow::Result<LanSwarm> {
    build_lan_swarm_with_listeners(&["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/udp/0/quic-v1"])
}

pub fn build_lan_swarm_with_listeners(listeners: &[&str]) -> anyhow::Result<LanSwarm> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            || libp2p::yamux::Config::default(),
        )?
        .with_quic()
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();
            let mut kad =
                libp2p::kad::Behaviour::new(peer_id, libp2p::kad::store::MemoryStore::new(peer_id));
            kad.set_mode(Some(libp2p::kad::Mode::Server));
            Ok(LanBehaviour {
                identify: libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                    "/ts-p2p/1".into(),
                    key.public(),
                )),
                mdns: libp2p::mdns::tokio::Behaviour::new(Default::default(), peer_id)?,
                kad,
                request_response: libp2p::request_response::cbor::Behaviour::new(
                    [(
                        libp2p::swarm::StreamProtocol::new(CHUNK_PROTOCOL),
                        libp2p::request_response::ProtocolSupport::Full,
                    )],
                    libp2p::request_response::Config::default(),
                ),
            })
        })?
        .build();
    for listener in listeners {
        swarm.listen_on((*listener).parse()?)?;
    }
    Ok(swarm)
}

pub async fn run_lan_node(swarm: LanSwarm) -> anyhow::Result<()> {
    run_lan_node_with_provider(swarm, ChunkProvider::default()).await
}

pub async fn run_lan_node_with_provider(
    swarm: LanSwarm,
    provider: ChunkProvider,
) -> anyhow::Result<()> {
    run_lan_node_with_provider_and_notify(swarm, provider, None).await
}

pub async fn run_lan_node_with_provider_and_notify(
    mut swarm: LanSwarm,
    provider: ChunkProvider,
    mut listen_tx: Option<tokio::sync::oneshot::Sender<libp2p::Multiaddr>>,
) -> anyhow::Result<()> {
    use futures::StreamExt;
    let provider = provider;
    loop {
        match swarm.select_next_some().await {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                println!("listen: {address}");
                if let Some(sender) = listen_tx.take() {
                    let _ = sender.send(address);
                }
            }
            libp2p::swarm::SwarmEvent::Behaviour(LanBehaviourEvent::Mdns(event)) => match event {
                libp2p::mdns::Event::Discovered(peers) => {
                    for (peer_id, address) in peers {
                        swarm
                            .behaviour_mut()
                            .kad
                            .add_address(&peer_id, address.clone());
                        if let Err(error) = swarm.dial(address) {
                            eprintln!("dial {peer_id} failed: {error}");
                        }
                    }
                }
                libp2p::mdns::Event::Expired(peers) => {
                    for (peer_id, address) in peers {
                        swarm.behaviour_mut().kad.remove_address(&peer_id, &address);
                    }
                }
            },
            libp2p::swarm::SwarmEvent::Behaviour(LanBehaviourEvent::Identify(
                libp2p::identify::Event::Received { peer_id, info, .. },
            )) => {
                for address in info.listen_addrs {
                    swarm.behaviour_mut().kad.add_address(&peer_id, address);
                }
            }
            libp2p::swarm::SwarmEvent::Behaviour(LanBehaviourEvent::RequestResponse(event)) => {
                if let libp2p::request_response::Event::Message {
                    message:
                        libp2p::request_response::Message::Request {
                            request, channel, ..
                        },
                    ..
                } = event
                {
                    let response = provider.respond(request);
                    if let Err(error) = swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, response)
                    {
                        eprintln!("response failed: {error:?}");
                    }
                }
            }
            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("connected: {peer_id}");
            }
            libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("disconnected: {peer_id}");
            }
            _ => {}
        }
    }
}

pub const MANIFEST_PROTOCOL: &str = "/ts-p2p/manifest/1";
pub const CHUNK_PROTOCOL: &str = "/ts-p2p/chunk/1";
pub const MAX_CHUNKS_PER_REQUEST: usize = 64;
pub const MAX_CHUNK_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChunkRequest {
    pub request_id: u64,
    pub tensor_hash: Hash32,
    pub chunk_index: u32,
    pub expected_hash: Hash32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerRequest {
    GetManifest {
        manifest_hash: Hash32,
    },
    GetChunks {
        tensor_hash: Hash32,
        chunks: Vec<ChunkRequest>,
    },
    Have {
        hashes: Vec<Hash32>,
    },
    Cancel {
        request_id: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerResponse {
    Manifest {
        manifest_hash: Hash32,
        bytes: Vec<u8>,
    },
    Chunk {
        request_id: u64,
        tensor_hash: Hash32,
        chunk_index: u32,
        payload: Vec<u8>,
        payload_hash: Hash32,
    },
    Have {
        hashes: Vec<Hash32>,
    },
    Error {
        request_id: Option<u64>,
        code: PeerErrorCode,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerErrorCode {
    NotFound,
    InvalidRequest,
    TooLarge,
    Busy,
    Internal,
}

pub fn dht_key(kind: &str, id: &Hash32) -> Vec<u8> {
    let mut digest = Sha256::new();
    digest.update(b"TS-DHT\0");
    digest.update(kind.as_bytes());
    digest.update([0]);
    digest.update(id.0);
    digest.finalize().to_vec()
}

pub fn manifest_dht_key(hash: &Hash32) -> Vec<u8> {
    dht_key("manifest", hash)
}
pub fn tensor_dht_key(hash: &Hash32) -> Vec<u8> {
    dht_key("tensor", hash)
}

pub fn publish_manifest(
    swarm: &mut LanSwarm,
    hash: &Hash32,
) -> Result<libp2p::kad::QueryId, libp2p::kad::store::Error> {
    swarm
        .behaviour_mut()
        .kad
        .start_providing(libp2p::kad::RecordKey::new(&manifest_dht_key(hash)))
}

pub fn publish_tensor(
    swarm: &mut LanSwarm,
    hash: &Hash32,
) -> Result<libp2p::kad::QueryId, libp2p::kad::store::Error> {
    swarm
        .behaviour_mut()
        .kad
        .start_providing(libp2p::kad::RecordKey::new(&tensor_dht_key(hash)))
}

pub fn find_manifest_providers(swarm: &mut LanSwarm, hash: &Hash32) -> libp2p::kad::QueryId {
    swarm
        .behaviour_mut()
        .kad
        .get_providers(libp2p::kad::RecordKey::new(&manifest_dht_key(hash)))
}

pub fn find_tensor_providers(swarm: &mut LanSwarm, hash: &Hash32) -> libp2p::kad::QueryId {
    swarm
        .behaviour_mut()
        .kad
        .get_providers(libp2p::kad::RecordKey::new(&tensor_dht_key(hash)))
}

pub fn validate_chunk_request(
    request: &ChunkRequest,
    max_chunk_index: u32,
) -> Result<(), PeerErrorCode> {
    if request.chunk_index > max_chunk_index {
        return Err(PeerErrorCode::InvalidRequest);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FetchPriority {
    pub stage: u32,
    pub urgency: u8,
}

impl Ord for FetchPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .stage
            .cmp(&self.stage)
            .then_with(|| other.urgency.cmp(&self.urgency))
    }
}

impl PartialOrd for FetchPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FetchJob {
    pub priority: FetchPriority,
    pub request: ChunkRequest,
}

impl Ord for FetchJob {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for FetchJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Default)]
pub struct FetchScheduler {
    queue: BinaryHeap<FetchJob>,
    queued: HashSet<(Hash32, u32)>,
}

impl FetchScheduler {
    pub fn push(&mut self, job: FetchJob) {
        let key = (job.request.tensor_hash, job.request.chunk_index);
        if self.queued.insert(key) {
            self.queue.push(job);
        }
    }

    pub fn pop(&mut self) -> Option<FetchJob> {
        let job = self.queue.pop()?;
        self.queued
            .remove(&(job.request.tensor_hash, job.request.chunk_index));
        Some(job)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Availability {
    hashes: HashSet<Hash32>,
}

impl Availability {
    pub fn insert(&mut self, hash: Hash32) {
        self.hashes.insert(hash);
    }
    pub fn contains(&self, hash: &Hash32) -> bool {
        self.hashes.contains(hash)
    }
    pub fn retain_requested(&mut self, requested: impl IntoIterator<Item = Hash32>) {
        let wanted = requested.into_iter().collect::<HashSet<_>>();
        self.hashes.retain(|hash| wanted.contains(hash));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dht_keys_are_domain_separated() {
        let hash = Hash32([1; 32]);
        assert_ne!(manifest_dht_key(&hash), tensor_dht_key(&hash));
        assert_eq!(manifest_dht_key(&hash), manifest_dht_key(&hash));
    }

    #[test]
    fn invalid_chunk_index_is_rejected() {
        let request = ChunkRequest {
            request_id: 1,
            tensor_hash: Hash32::ZERO,
            chunk_index: 9,
            expected_hash: Hash32::ZERO,
        };
        assert_eq!(
            validate_chunk_request(&request, 8),
            Err(PeerErrorCode::InvalidRequest)
        );
    }

    #[test]
    fn provider_only_serves_hash_verified_chunks() {
        let mut provider = ChunkProvider::default();
        let tensor = Hash32([3; 32]);
        let payload = b"chunk".to_vec();
        let expected = ts_core::sha256(&payload);
        assert!(provider.insert_chunk(tensor, 0, expected, payload).is_ok());
        let response = provider.respond(PeerRequest::GetChunks {
            tensor_hash: tensor,
            chunks: vec![ChunkRequest {
                request_id: 7,
                tensor_hash: tensor,
                chunk_index: 0,
                expected_hash: expected,
            }],
        });
        assert!(matches!(
            response,
            PeerResponse::Chunk { request_id: 7, .. }
        ));
    }

    #[test]
    fn provider_rejects_unbounded_chunk_batches() {
        let provider = ChunkProvider::default();
        let response = provider.respond(PeerRequest::GetChunks {
            tensor_hash: Hash32::ZERO,
            chunks: (0..=MAX_CHUNKS_PER_REQUEST)
                .map(|index| ChunkRequest {
                    request_id: index as u64,
                    tensor_hash: Hash32::ZERO,
                    chunk_index: index as u32,
                    expected_hash: Hash32::ZERO,
                })
                .collect(),
        });
        assert!(matches!(
            response,
            PeerResponse::Error {
                code: PeerErrorCode::TooLarge,
                ..
            }
        ));
    }

    #[test]
    fn provider_inventory_loads_verified_manifest_chunks_from_store() {
        let root = tempfile::tempdir().unwrap();
        let store = ts_store::ObjectStore::open(root.path()).unwrap();
        let payload = b"chunk";
        let chunk_hash = sha256(payload);
        store.put_verified(chunk_hash, payload).unwrap();
        let tensor_hash = sha256(b"tensor-identity");
        let manifest = Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            5,
            vec![ts_core::TensorNode {
                descriptor: ts_core::TensorDescriptor {
                    name: "weight".into(),
                    shape: vec![5],
                    dtype: "U8".into(),
                    byte_len: 5,
                },
                tensor_hash,
                chunks: vec![ts_core::ChunkRef {
                    index: 0,
                    offset: 0,
                    length: 5,
                    hash: chunk_hash,
                }],
            }],
            vec![],
        );
        let provider = ChunkProvider::from_manifest(&store, &manifest).unwrap();
        assert!(matches!(
            provider.respond(PeerRequest::GetManifest {
                manifest_hash: manifest.root
            }),
            PeerResponse::Manifest { .. }
        ));
        assert!(matches!(
            provider.respond(PeerRequest::GetChunks {
                tensor_hash,
                chunks: vec![ChunkRequest {
                    request_id: 1,
                    tensor_hash,
                    chunk_index: 0,
                    expected_hash: chunk_hash
                }]
            }),
            PeerResponse::Chunk { .. }
        ));
    }

    #[test]
    fn scheduler_deduplicates_and_prioritizes_earlier_layers() {
        let mut scheduler = FetchScheduler::default();
        let make = |stage, index| FetchJob {
            priority: FetchPriority { stage, urgency: 1 },
            request: ChunkRequest {
                request_id: index as u64,
                tensor_hash: Hash32([stage as u8; 32]),
                chunk_index: index,
                expected_hash: Hash32::ZERO,
            },
        };
        scheduler.push(make(2, 0));
        scheduler.push(make(0, 0));
        scheduler.push(make(0, 0));
        assert_eq!(scheduler.len(), 2);
        assert_eq!(scheduler.pop().unwrap().priority.stage, 0);
    }

    #[tokio::test]
    async fn two_local_nodes_transfer_a_verified_chunk() {
        use futures::StreamExt;
        use libp2p::request_response::{Event, Message};
        use libp2p::swarm::SwarmEvent;

        let mut provider = ChunkProvider::default();
        let tensor = Hash32([8; 32]);
        let payload = b"layer-zero".to_vec();
        let expected = sha256(&payload);
        provider
            .insert_chunk(tensor, 0, expected, payload.clone())
            .unwrap();

        let server = build_lan_swarm_with_listeners(&["/ip4/127.0.0.1/tcp/0"]).unwrap();
        let server_id = *server.local_peer_id();
        let (address_tx, address_rx) = tokio::sync::oneshot::channel();
        let server_task = tokio::spawn(run_lan_node_with_provider_and_notify(
            server,
            provider,
            Some(address_tx),
        ));
        let address = address_rx
            .await
            .unwrap()
            .with(libp2p::multiaddr::Protocol::P2p(server_id.into()));

        let mut client = build_lan_swarm_with_listeners(&[]).unwrap();
        client.dial(address).unwrap();
        let request = PeerRequest::GetChunks {
            tensor_hash: tensor,
            chunks: vec![ChunkRequest {
                request_id: 42,
                tensor_hash: tensor,
                chunk_index: 0,
                expected_hash: expected,
            }],
        };
        let response = loop {
            match client.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    client
                        .behaviour_mut()
                        .request_response
                        .send_request(&peer_id, request.clone());
                }
                SwarmEvent::Behaviour(LanBehaviourEvent::RequestResponse(Event::Message {
                    message: Message::Response { response, .. },
                    ..
                })) => break response,
                _ => {}
            }
        };
        server_task.abort();
        assert!(
            matches!(response, PeerResponse::Chunk { request_id: 42, payload, .. } if payload == b"layer-zero")
        );
    }
}

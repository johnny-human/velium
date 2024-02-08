extern crate lazy_static;

use lazy_static::lazy_static;
use md5;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer};
use std::{collections::HashMap, sync::Mutex};
use veilid_core::{PeerTableData, Timestamp};

#[derive(Debug)]
pub struct Md5Digest(md5::Digest);

impl Serialize for Md5Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:x}", self.0))
    }
}

impl PartialEq for Md5Digest {
    fn eq(&self, other: &Self) -> bool {
        self.0 .0 == other.0 .0
    }
}

impl Clone for Md5Digest {
    fn clone(&self) -> Self {
        Md5Digest(self.0.clone())
    }
}

#[derive(Debug, Serialize)]
pub struct PeerState {
    pub peers: HashMap<String, Peer>,
    pub hashes: HashMap<String, Md5Digest>,
}

lazy_static! {
    pub static ref PEER_STATE: Mutex<PeerState> = Mutex::new(PeerState {
        peers: HashMap::new(),
        hashes: HashMap::new(),
    });
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Peer {
    node_id: String,
    peer_address: String,
    time_added: Timestamp,
    messages_sent: u32,
}

#[derive(Serialize)]
pub struct Changes(HashMap<String, Peer>);

impl Peer {
    fn hash(&self) -> md5::Digest {
        let content = format!(
            "{}{}{}{}",
            self.node_id, self.peer_address, self.time_added, self.messages_sent
        );
        md5::compute(content.as_bytes())
    }
}

#[derive(Debug)]
pub enum UpsertError {
    InvalidData(String),
    LockFailed(String),
}

impl PeerState {
    pub fn new(peers: HashMap<String, Peer>) -> Self {
        Self {
            peers,
            hashes: HashMap::new(),
        }
    }

    pub fn upsert(network_peers: Vec<PeerTableData>) -> Result<Changes, UpsertError> {
        let mut changes: HashMap<String, Peer> = HashMap::new();

        let mut data = PEER_STATE
            .lock()
            .map_err(|_| UpsertError::LockFailed("Lock failed".to_string()))?;

        for peer in network_peers {
            if let Some(first_node_id) = peer.node_ids.first() {
                let node_id = first_node_id.to_string();
                let peer = Peer {
                    node_id: node_id.clone(),
                    peer_address: peer.peer_address,
                    time_added: peer.peer_stats.time_added,
                    messages_sent: peer.peer_stats.rpc_stats.messages_sent,
                };

                if !data.hashes.get(&node_id).map_or(false, |existing_hash| {
                    *existing_hash == Md5Digest(peer.hash())
                }) {
                    let hash = Md5Digest(peer.hash());
                    data.peers.insert(node_id.clone(), peer.clone());
                    data.hashes.insert(node_id.clone(), hash);
                    changes.insert(node_id, peer);
                }
            } else {
                return Err(UpsertError::InvalidData(
                    "PeerTableData has empty node_ids".into(),
                ));
            }
        }

        Ok(Changes(changes))
    }

    // pub fn upsert_peer(&mut self, peer: Peer) {
    //     let hash = peer.hash();
    //     match self.hashes.get(&peer.node_id) {
    //         Some(&existing_hash) if existing_hash == hash => {
    //             // If the hash hasn't changed, there's nothing to update
    //             return;
    //         }
    //         _ => {
    //             // Update or insert the peer and its hash
    //             self.peers.insert(peer.node_id.clone(), peer.clone());
    //             self.hashes.insert(peer.node_id, hash);
    //         }
    //     }
    // }
}

use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use sha3::{Digest, Sha3_256};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::path::PathBuf;

use crate::error::Error;

/// P2P default binding addr.
pub const P2P_ADDR: &str = "0.0.0.0:7364";

/// P2P default transport.
pub const P2P_TRANSPORT: &str = "tcp";

/// Layer default binding addr.
pub const LAYER_ADDR: &str = "0.0.0.0:7000";

/// Layer default lower on-off (whether public).
pub const LAYER_PUBLIC_DEFAULT: bool = true;

/// RPC default binding addr.
pub const RPC_ADDR: &str = "127.0.0.1:8000";

/// message channel.
pub const MAX_MESSAGE_CAPACITY: usize = 1024;

/// Configure file name
pub const CONFIG_FILE_NAME: &str = "config.toml";

pub const DEFAULT_STORAGE_DIR_NAME: &str = ".tdn";

lazy_static! {
    pub static ref DEFAULT_STORAGE_DIR: PathBuf = {
        #[cfg(feature = "dev")]
        let mut path = PathBuf::from("./");

        #[cfg(not(feature = "dev"))]
        let mut path = if dirs::home_dir().is_some() {
            dirs::home_dir().unwrap()
        } else {
            PathBuf::from("./")
        };

        path.push(DEFAULT_STORAGE_DIR_NAME);
        let _ = std::fs::create_dir_all(&path)
            .expect(&format!("Cannot Build Storage Path: {:?}", path));
        path
    };
}

// Type: RPC Param
pub type RpcParam = Value;
pub use serde_json::json;

/// Type: PeerAddr
pub type PeerAddr = chamomile::prelude::PeerId;
/// Type: P2P common Broadcast
pub type Broadcast = chamomile::prelude::Broadcast;

/// Type: GroupId
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct GroupId(pub [u8; 32]);

impl GroupId {
    pub fn short_show(&self) -> String {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        let mut new_hex = String::new();
        new_hex.push_str("0x");
        new_hex.push_str(&hex[0..4]);
        new_hex.push_str("...");
        new_hex.push_str(&hex[hex.len() - 5..]);
        new_hex
    }

    pub fn from_symbol(s: impl ToString) -> GroupId {
        let s = s.to_string();
        let mut sha = Sha3_256::new();
        sha.input(&s);
        let mut peer_bytes = [0u8; 32];
        peer_bytes.copy_from_slice(&sha.result()[..]);
        GroupId(peer_bytes)
    }

    pub fn from_hex(s: impl ToString) -> Result<GroupId, Error> {
        let s = s.to_string();
        if s.len() != 64 {
            return Err(Error::Hex);
        }

        let mut value = [0u8; 32];

        for i in 0..(s.len() / 2) {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|_e| Error::Hex)?;
            value[i] = res;
        }

        Ok(GroupId(value))
    }

    pub fn to_hex(&self) -> String {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        hex
    }
}

impl Debug for GroupId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

/// Type: EventId
pub struct EventId(pub [u8; 64]);

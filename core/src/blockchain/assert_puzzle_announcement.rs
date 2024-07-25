use crate::blockchain::sized_bytes::Bytes32;
use dg_xch_macros::ChiaSerial;
use serde::{Deserialize, Serialize};

use super::unsized_bytes::UnsizedBytes;
use crate::blockchain::sized_bytes::SizedBytes;

#[derive(ChiaSerial, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct AssertPuzzleAnnouncement {
    pub message: UnsizedBytes,
    pub morph_bytes: Option<UnsizedBytes>,
    pub puzzle_hash: Bytes32,
}

impl AssertPuzzleAnnouncement {
    pub fn new(puzzle_hash: Bytes32, message: Vec<u8>, morph_bytes: Option<Vec<u8>>) -> Self {
        let message = UnsizedBytes::new(&message);
        let morph_bytes = match morph_bytes {
            Some(m) => Some(UnsizedBytes::new(&m)),
            None => None,
        };
        Self {
            message: message,
            morph_bytes: morph_bytes,
            puzzle_hash: puzzle_hash,
        }
    }
}

use crate::blockchain::sized_bytes::Bytes32;
use crate::blockchain::sized_bytes::SizedBytes;
use dg_xch_macros::ChiaSerial;
use serde::{Deserialize, Serialize};

use super::unsized_bytes::UnsizedBytes;
#[derive(ChiaSerial, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct AssertCoinAnnouncement {
    pub message: UnsizedBytes,
    pub morph_bytes: Option<UnsizedBytes>,
    pub coin_id: Bytes32,
}

impl AssertCoinAnnouncement {
    pub fn new(coin_id: Bytes32, message: Vec<u8>, morph_bytes: Option<Vec<u8>>) -> Self {
        let message = UnsizedBytes::new(&message);
        let morph_bytes = match morph_bytes {
            Some(m) => Some(UnsizedBytes::new(&m)),
            None => None,
        };
        Self {
            message,
            morph_bytes,

            coin_id: coin_id,
        }
    }
}

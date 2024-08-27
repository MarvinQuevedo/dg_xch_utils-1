use crate::blockchain::unsized_bytes::UnsizedBytes;
use dg_xch_macros::ChiaSerial;
use serde::{Deserialize, Serialize};

#[derive(ChiaSerial, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct VdfOutput {
    pub data: UnsizedBytes,
}

impl Default for VdfOutput {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

use super::unsized_bytes::UnsizedBytes;
use crate::blockchain::sized_bytes::{prep_hex_str, SizedBytes};
use crate::{
    blockchain::sized_bytes::Bytes32,
    clvm::{program::Program, utils::encode_bigint},
};
use dg_xch_macros::ChiaSerial;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::vec::Vec;
#[derive(ChiaSerial, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Payment {
    pub puzzle_hash: Bytes32,
    pub amount: u64,
    pub memos: Vec<UnsizedBytes>,
}

impl Payment {
    pub fn as_condition_args(&self) -> Vec<Program> {
        let mut args = Vec::new();
        args.push(Program::try_from(self.puzzle_hash.bytes.to_vec()).unwrap());
        args.push(Program::try_from(encode_bigint(BigInt::from(self.amount)).unwrap()).unwrap());
        let mut memos_program_list = Vec::new();
        for memo in &self.memos {
            memos_program_list.push(Program::try_from(memo.bytes.to_vec()).unwrap());
        }
        args.push(Program::to(memos_program_list));

        args
    }

    pub fn as_condition(&self) -> Program {
        let program_list = vec![
            Program::try_from(encode_bigint(BigInt::from(51)).unwrap()).unwrap(),
            Program::to(self.as_condition_args()),
        ];
        Program::to(program_list)
    }

    pub fn name(&self) -> Bytes32 {
        self.as_condition().tree_hash()
    }

    pub fn from_condition(condition: &Program) -> Payment {
        let python_condition = condition.as_list();
        let puzzle_hash = Bytes32::from(python_condition[1].as_atom().unwrap().serialized);
        let amount = python_condition[2].as_int().unwrap().try_into().unwrap();
        let memos = if python_condition.len() > 3 {
            python_condition[3..]
                .iter()
                .map(|m| UnsizedBytes::new(&m.as_atom().unwrap().serialized.to_vec()))
                .collect()
        } else {
            Vec::new()
        };

        Payment {
            puzzle_hash,
            amount,
            memos,
        }
    }
}

use crate::blockchain::coin::Coin;
use crate::blockchain::condition_opcode::{ConditionCost, ConditionOpcode};
use crate::blockchain::sized_bytes::{Bytes32, SizedBytes};
use crate::blockchain::utils::{additions_for_solution, fee_for_solution};
use crate::clvm::program::{Program, SerializedProgram};
use crate::clvm::utils::INFINITE_COST;
use dg_xch_macros::ChiaSerial;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};
use std::vec;

use super::unsized_bytes::UnsizedBytes;

#[derive(ChiaSerial, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct CoinSpend {
    pub coin: Coin,
    pub puzzle_reveal: SerializedProgram,
    pub solution: SerializedProgram,
}

impl CoinSpend {
    pub fn additions(&self) -> Result<Vec<Coin>, Error> {
        additions_for_solution(
            self.coin.name(),
            &self.puzzle_reveal,
            &self.solution,
            INFINITE_COST,
        )
    }
    pub fn memos(&self) -> Vec<(Coin, Option<Vec<UnsizedBytes>>)> {
        let result = compute_additions_with_cost_for_memos(self, INFINITE_COST);
        match result {
            Ok((additions, _)) => additions,
            Err(_) => vec![],
        }
    }
    pub fn reserved_fee(self) -> BigInt {
        fee_for_solution(&self.puzzle_reveal, &self.solution, INFINITE_COST)
    }
}

pub fn compute_additions_with_cost(
    cs: &CoinSpend,
    max_cost: u64,
) -> Result<(Vec<Coin>, u64), Error> {
    let parent_coin_info = cs.coin.name();
    let mut ret: Vec<Coin> = vec![];
    let (mut cost, r) = cs
        .puzzle_reveal
        .run_with_cost(max_cost, &cs.solution.to_program())?;
    for cond in Program::to(r).as_list() {
        if cost > max_cost {
            return Err(Error::new(
                ErrorKind::Other,
                "BLOCK_COST_EXCEEDS_MAX compute_additions() for CoinSpend",
            ));
        } else {
            let atoms = cond.as_list();
            if atoms.is_empty() {
                return Err(Error::new(ErrorKind::Other, "Atoms List is Empty"));
            }
            let op = &atoms[0];
            if [ConditionOpcode::AggSigMe, ConditionOpcode::AggSigUnsafe].contains(&op.into()) {
                cost += ConditionCost::AggSig as u64;
                continue;
            }
            if ConditionOpcode::from(op) != ConditionOpcode::CreateCoin {
                continue;
            }
            cost += ConditionCost::CreateCoin as u64;
            if atoms.len() < 3 {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Invalid Number ot Atoms in Program",
                ));
            }
            let puzzle_hash = Bytes32::new(&atoms[1].as_vec().unwrap_or_default());
            let amount = atoms[2].as_int()?;
            ret.push(Coin {
                parent_coin_info,
                puzzle_hash,
                amount: amount
                    .to_u64()
                    .expect("Expected a positive amount when computing additions"),
            });
        }
    }
    Ok((ret, cost))
}

pub fn compute_additions_with_cost_for_memos(
    cs: &CoinSpend,
    max_cost: u64,
) -> Result<(Vec<(Coin, Option<Vec<UnsizedBytes>>)>, u64), Error> {
    let parent_coin_info = cs.coin.name();
    let mut ret: Vec<(Coin, Option<Vec<UnsizedBytes>>)> = Vec::new();
    let (mut cost, r) = cs
        .puzzle_reveal
        .run_with_cost(max_cost, &cs.solution.to_program())?;
    for cond in Program::to(r).as_list() {
        if cost > max_cost {
            return Err(Error::new(
                ErrorKind::Other,
                "BLOCK_COST_EXCEEDS_MAX compute_additions() for CoinSpend",
            ));
        } else {
            let atoms = cond.as_list();
            if atoms.is_empty() {
                return Err(Error::new(ErrorKind::Other, "Atoms List is Empty"));
            }
            let op = &atoms[0];
            if [ConditionOpcode::AggSigMe, ConditionOpcode::AggSigUnsafe].contains(&op.into()) {
                cost += ConditionCost::AggSig as u64;
                continue;
            }
            if ConditionOpcode::from(op) != ConditionOpcode::CreateCoin {
                continue;
            }
            cost += ConditionCost::CreateCoin as u64;
            if atoms.len() < 3 {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Invalid Number ot Atoms in Program",
                ));
            }
            let puzzle_hash = Bytes32::new(&atoms[1].as_vec().unwrap_or_default());
            let amount = atoms[2].as_int()?;
            let coin = Coin {
                parent_coin_info,
                puzzle_hash,
                amount: amount
                    .to_u64()
                    .expect("Expected a positive amount when computing additions"),
            };
            let mut memos: Option<Vec<UnsizedBytes>> = None;
            if atoms.len() > 3 {
                let memos_args = atoms[3].as_list();
                if !memos_args.is_empty() {
                    let mut memos_vec = Vec::new();
                    for memo in memos_args {
                        memos_vec.push(UnsizedBytes::new(&memo.as_vec().unwrap()));
                    }
                    memos = Some(memos_vec);
                }
            }
            ret.push((coin, memos));
        }
    }
    Ok((ret, cost))
}

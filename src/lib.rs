mod blockstore;
mod state;
mod types;
mod uint256;
use crate::blockstore::Blockstore;
use cid::multihash::Code;
use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_encoding::{to_vec, CborStore, RawBytes, DAG_CBOR};
use fvm_sdk as sdk;
use fvm_sdk::message::NO_DATA_BLOCK_ID;
use fvm_shared::bigint::BigUint;
use fvm_shared::ActorID;
use state::{State, Token};
use std::collections::HashMap;
use types::{Allowance, Approve, Mint, Transfer, TransferFrom};
use uint256::Uint256;
#[macro_use]
mod abort;

#[no_mangle]
pub fn invoke(params: Vec<u8>) -> u32 {
    let ret: Option<RawBytes> = match sdk::message::method_number() {
        1 => {
            // create token(symbol,decimal,total_supply)
            let mock_total_supply =
                BigUint::parse_bytes(b"100000000000000000000000000000000", 10).unwrap();
            let mock_total_supply = Uint256 {
                big_uint: mock_total_supply,
            };
            let mock_token = Token {
                symbol: "wfil".to_string(),
                decimal: 18,
                total_supply: mock_total_supply,
            };

            let params = serde_json::to_vec(&mock_token).unwrap();
            let mut state = State::default();
            let token: Token = serde_json::from_slice(&params).unwrap();
            let result = state.constructor(token);
            state.save();
            result
        }
        2 => {
            // mint
            let mock_value = BigUint::parse_bytes(b"4546347290348029834222344344", 10).unwrap();
            let mock_amount = Uint256 {
                big_uint: mock_value,
            };
            unsafe {
                let mock_actor = sdk::sys::message::caller().unwrap();
                let mock_mint = Mint {
                    actor: mock_actor,
                    amount: mock_amount,
                };

                let params = serde_json::to_vec(&mock_mint).unwrap();
                let mint = Mint::from_slice(&params).unwrap();
                let mut state = State::load();
                let res = state.mint(mint.actor, mint.amount);
                state.save();
                Some(RawBytes::new(res.to_bytes()))
            }
        }
        3 => {
            //  actor balance
            unsafe {
                let mock_actor = sdk::sys::message::caller().unwrap();
                let state = State::load();
                let balance = state.balance_of(mock_actor);
                Some(RawBytes::new(balance.to_string().as_bytes().to_vec()))
            }
        }
        4 => {
            unsafe {
                //  allowance
                let mock_actor = sdk::sys::message::caller().unwrap();
                let mock_approve = Allowance {
                    from: mock_actor,
                    to: 2u64,
                };
                let state = State::load();
                let amount = state.allowance(&mock_approve.from, &mock_approve.to);
                Some(RawBytes::new(amount.to_string().as_bytes().to_vec()))
            }
        }
        5 => {
            unsafe {
                // transfer_from
                let mock_value = BigUint::parse_bytes(b"45463475445", 10).unwrap();
                let mock_amount = Uint256 {
                    big_uint: mock_value,
                };
                let mock_actor = sdk::sys::message::caller().unwrap();
                let mock_transfer_from = TransferFrom {
                    from: mock_actor,
                    to: 2u64,
                    amount: mock_amount,
                };

                let mut state = State::load();
                let res = state.transfer_from(
                    mock_transfer_from.from,
                    mock_transfer_from.to,
                    mock_transfer_from.amount,
                );
                state.save();
                Some(RawBytes::new(res.to_bytes()))
            }
        }
        6 => {
            // transfer

            //let transfer = Transfer::from_slice(&params).unwrap();
            let mock_value = BigUint::parse_bytes(b"454634729034802983", 10).unwrap();
            let mock_amount = Uint256 {
                big_uint: mock_value,
            };
            let mock_transfer = Transfer {
                to: 2u64,
                amount: mock_amount,
            };
            let mut state = State::load();
            let res = state.transfer(mock_transfer.to, mock_transfer.amount);
            state.save();
            Some(RawBytes::new(res.to_bytes()))
        }
        7 => {
            let mock_value = BigUint::parse_bytes(b"454634729034802983", 10).unwrap();
            let mock_amount = Uint256 {
                big_uint: mock_value,
            };
            let mock_approve = Approve {
                actor: 2u64,
                amount: mock_amount,
            };
            let mut state = State::load();
            let res = state.approve(mock_approve.actor, mock_approve.amount);
            state.save();
            res
        }
        8 => {
            // token symbol
            let state = State::load();
            Some(RawBytes::new(state.symbol().into_bytes()))
        }
        9 => {
            //  token decimal
            let state = State::load();
            Some(RawBytes::new(state.decimal().to_be_bytes().to_vec()))
        }
        10 => {
            //  token total_sypply
            let state = State::load();
            Some(RawBytes::new(state.total_supply().to_bytes_be()))
        }
        _ => abort!(USR_UNHANDLED_MESSAGE, "unrecognized method"),
    };

    match ret {
        None => NO_DATA_BLOCK_ID,
        Some(v) => match sdk::ipld::put_block(DAG_CBOR, v.bytes()) {
            Ok(id) => id,
            Err(err) => abort!(USR_SERIALIZATION, "failed to store return value: {}", err),
        },
    }
}

#[cfg(test)]
mod uint256_test {

    use super::*;
    #[test]
    fn test_state() {
        unsafe {
            let actor: ActorID = 1u64;
            let value = BigUint::parse_bytes(b"4546347290348029834222344344", 10).unwrap();
            let amount = Uint256 {
                big_uint: value.clone(),
            };

            let total_supply =
                BigUint::parse_bytes(b"100000000000000000000000000000000", 10).unwrap();
            let total_supply = Uint256 {
                big_uint: total_supply.clone(),
            };
            let mut state = State::default();
            let token = Token {
                symbol: "wfil".to_string(),
                decimal: 18u64,
                total_supply: total_supply,
            };
            state.constructor(token);
            state.mint(actor, amount.clone());
            let balance = state.balance_of(actor);
            assert_eq!(balance, amount);

            let to_actor: ActorID = 2u64;

            let value = BigUint::parse_bytes(b"4546347290348029834222", 10).unwrap();
            let amount = Uint256 {
                big_uint: value.clone(),
            };

            let from_old_balance = state.balance_of(actor);
            let to_old_balance = state.balance_of(to_actor);
            state.transfer(to_actor, amount.clone());
            let from_balance = state.balance_of(actor);
            let to_balance = state.balance_of(to_actor);
            assert_eq!(from_old_balance, from_balance + amount.clone());
            assert_eq!(to_balance, to_old_balance + amount.clone());

            let value = BigUint::parse_bytes(b"4546347290348029", 10).unwrap();
            let amount = Uint256 {
                big_uint: value.clone(),
            };
            state.approve(to_actor, amount.clone());

            let allowance_balance = state.allowance(&actor, &to_actor);
            assert_eq!(allowance_balance, amount);

            let value = BigUint::parse_bytes(b"454634729034", 10).unwrap();
            let amount = Uint256 {
                big_uint: value.clone(),
            };
            let from_old_balance = state.balance_of(actor);
            let to_old_balance = state.balance_of(to_actor);
            let allowance_old = state.allowance(&actor, &to_actor);
            state.transfer_from(actor, to_actor, amount.clone());
            let from_balance = state.balance_of(actor);
            let to_balance = state.balance_of(to_actor);
            let allowance = state.allowance(&actor, &to_actor);
            assert_eq!(from_old_balance, from_balance + amount.clone());
            assert_eq!(to_balance, to_old_balance + amount.clone());
            assert_eq!(allowance_old, allowance + amount.clone());
        }
    }

    #[test]
    fn uint256_serde_test() {
        let value = BigUint::parse_bytes(b"10000000", 10).unwrap();

        let u1 = Uint256 {
            big_uint: value.clone(),
        };

        let serde_value = serde_json::to_vec(&u1).unwrap();
        let uint256: Uint256 = serde_json::from_slice(&serde_value).unwrap();

        assert_eq!(uint256.big_uint, value);

        let value = BigUint::parse_bytes(b"1289472934337823047092830498", 10).unwrap();

        let u1 = Uint256 {
            big_uint: value.clone(),
        };

        let serde_value = serde_json::to_vec(&u1).unwrap();
        let uint256: Uint256 = serde_json::from_slice(&serde_value).unwrap();

        assert_eq!(uint256.big_uint, value);
    }

    #[test]
    fn uint256_ops_test() {
        let a = BigUint::parse_bytes(b"2347290348029834222344344", 10).unwrap();
        let u1 = Uint256 {
            big_uint: a.clone(),
        };
        let b = BigUint::parse_bytes(b"4546347290348029834222344344", 10).unwrap();
        let u2 = Uint256 {
            big_uint: b.clone(),
        };
        let u3 = u1.clone() + u2.clone();
        let c = a.clone() + b.clone();
        assert_eq!(u3.big_uint, c);

        let u3 = u2.clone() - u1.clone();
        let c = b.clone() - a.clone();
        assert_eq!(u3.big_uint, c);

        assert_eq!(u3 < u2, true);
        assert_eq!(u3 > u2, false);
    }
}

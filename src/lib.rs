mod blockstore;
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
use std::collections::HashMap;
use uint256::Uint256;
/// A macro to abort concisely.
/// This should be part of the SDK as it's very handy.
macro_rules! abort {
    ($code:ident, $msg:literal $(, $ex:expr)*) => {
        fvm_sdk::vm::abort(
            fvm_shared::error::ExitCode::$code.value(),
            Some(format!($msg, $($ex,)*).as_str()),
        )
    };
}
#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, Default)]
pub struct Token {
    pub symbol: String,
    pub decimal: u64,
    pub total_supply: Uint256,
}

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, Default)]
pub struct State {
    pub token: Token,
    pub balance_of: HashMap<ActorID, Uint256>,
    pub allowance: HashMap<ActorID, HashMap<ActorID, Uint256>>,
}

/// We should probably have a derive macro to mark an object as a state object,
/// and have load and save methods automatically generated for them as part of a
/// StateObject trait (i.e. impl StateObject for State).
impl State {
    pub fn load() -> Self {
        // First, load the current state root.
        let root = match sdk::sself::root() {
            Ok(root) => root,
            Err(err) => abort!(USR_ILLEGAL_STATE, "failed to get root: {:?}", err),
        };

        // Load the actor state from the state tree.
        match Blockstore.get_cbor::<Self>(&root) {
            Ok(Some(state)) => state,
            Ok(None) => abort!(USR_ILLEGAL_STATE, "state does not exist"),
            Err(err) => abort!(USR_ILLEGAL_STATE, "failed to get state: {}", err),
        }
    }

    pub fn save(&self) -> Cid {
        let serialized = match to_vec(self) {
            Ok(s) => s,
            Err(err) => abort!(USR_SERIALIZATION, "failed to serialize state: {:?}", err),
        };
        let serialized = serialized.as_slice();
        let cid = match sdk::ipld::put(Code::Blake2b256.into(), 32, DAG_CBOR, &serialized) {
            Ok(cid) => cid,
            Err(err) => abort!(USR_SERIALIZATION, "failed to store initial state: {:}", err),
        };
        if let Err(err) = sdk::sself::set_root(&cid) {
            abort!(USR_ILLEGAL_STATE, "failed to set root ciid: {:}", err);
        }
        // abort!(USR_ILLEGAL_STATE, "failed to set root ciid:");
        cid
    }
}

#[no_mangle]
pub fn invoke(params: Vec<u8>) -> u32 {
    let ret: Option<RawBytes> = match sdk::message::method_number() {
        1 => {
            let mut state = State::load();
            let token: Token = serde_json::from_slice(&params).unwrap();
            let result = constructor(&mut state, token);
            state.save();
            result
        }
        //2 => balance_of(),
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

pub fn constructor(state: &mut State, token: Token) -> Option<RawBytes> {
    state.token = token;
    state.balance_of = HashMap::new();
    state.allowance = HashMap::new();
    None
}

pub fn symbol(state: &State) -> String {
    state.token.symbol.clone()
}

pub fn decimal(state: &State) -> u64 {
    state.token.decimal
}

pub fn total_supply(state: &State) -> Uint256 {
    state.token.total_supply.clone()
}

pub fn mint(state: &mut State, actor: ActorID, amount: Uint256) -> Option<RawBytes> {
    let mut default_balance: Uint256 = Default::default();
    match state.balance_of.get_mut(&actor) {
        None => {
            state.balance_of.insert(actor, amount);
        }
        Some(balance) => {
            *balance = balance.clone() + amount;
        }
    };
    None
}

pub fn balance_of(state: &State, actor: ActorID) -> Uint256 {
    let default_balance = Uint256::default();

    let balacnce = state.balance_of.get(&actor).unwrap_or(&default_balance);
    balacnce.clone()
}

pub unsafe fn transfer(state: &mut State, to: ActorID, amount: Uint256) -> Option<RawBytes> {
    //let from = sdk::sys::message::caller().unwrap();
    let from: ActorID = 1u64;
    let mut from_default_balance = Uint256::default();

    let from_balance = state.balance_of.get_mut(&from);
    match from_balance {
        None => {
            abort!(SYS_ASSERTION_FAILED, "Insufficient Balance");
        }
        Some(from_balance) => {
            if *from_balance < amount {
                abort!(SYS_ASSERTION_FAILED, "Insufficient Balance")
            }
            *from_balance = from_balance.clone() - amount.clone();
        }
    }

    match state.balance_of.get_mut(&to) {
        None => {
            state.balance_of.insert(to, amount);
        }
        Some(to_balance) => {
            *to_balance = to_balance.clone() + amount.clone();
        }
    }

    None
}

pub unsafe fn approve(state: &mut State, to: ActorID, amount: Uint256) -> Option<RawBytes> {
    //let from = sdk::sys::message::caller().unwrap();
    let from: ActorID = 1u64;
    let default_balance = Uint256::default();
    let from_balance = state
        .balance_of
        .get(&from)
        .unwrap_or(&default_balance)
        .clone();
    if from_balance < amount {
        abort!(SYS_ASSERTION_FAILED, "Insufficient Balance")
    }
    if let None = state.allowance.get_mut(&from) {
        state.allowance.insert(from, HashMap::new());
    }
    state.allowance.get_mut(&from).unwrap().insert(to, amount);
    None
}

pub fn allowance(state: &State, from: &ActorID, to: &ActorID) -> Uint256 {
    //let from = sdk::sys::message::caller().unwrap();

    match state.allowance.get(from) {
        None => return Uint256::default(),
        Some(allow) => match allow.get(to) {
            None => return Uint256::default(),
            Some(balance) => return balance.clone(),
        },
    }
}

pub unsafe fn transfer_from(
    state: &mut State,
    from: ActorID,
    to: ActorID,
    amount: Uint256,
) -> Option<RawBytes> {
    match state.allowance.get_mut(&from) {
        None => {
            abort!(SYS_ASSERTION_FAILED, "Insufficient Balance");
        }
        Some(allowance) => {
            let mut allowance_default_balance: Uint256 = Default::default();
            match allowance.get_mut(&to) {
                None => {
                    abort!(SYS_ASSERTION_FAILED, "Insufficient Balance");
                }
                Some(value) => {
                    if *value < amount {
                        abort!(SYS_ASSERTION_FAILED, "Insufficient Balance")
                    }
                    let from_balance = state.balance_of.get_mut(&from).unwrap();
                    *from_balance = from_balance.clone() - amount.clone();

                    match state.balance_of.get_mut(&to) {
                        None => {
                            state.balance_of.insert(to, amount);
                        }
                        Some(to_balance) => {
                            *to_balance = to_balance.clone() + amount.clone();
                            *value = value.clone() - amount;
                        }
                    }
                }
            }
        }
    };

    None
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
            constructor(&mut state, token);
            mint(&mut state, actor, amount.clone());
            let balance = balance_of(&state, actor);
            assert_eq!(balance, amount);

            let to_actor: ActorID = 2u64;

            let value = BigUint::parse_bytes(b"4546347290348029834222", 10).unwrap();
            let amount = Uint256 {
                big_uint: value.clone(),
            };

            let from_old_balance = balance_of(&state, actor);
            let to_old_balance = balance_of(&state, to_actor);
            transfer(&mut state, to_actor, amount.clone());
            let from_balance = balance_of(&state, actor);
            let to_balance = balance_of(&state, to_actor);
            assert_eq!(from_old_balance, from_balance + amount.clone());
            assert_eq!(to_balance, to_old_balance + amount.clone());

            let value = BigUint::parse_bytes(b"4546347290348029", 10).unwrap();
            let amount = Uint256 {
                big_uint: value.clone(),
            };
            approve(&mut state, to_actor, amount.clone());

            let allowance_balance = allowance(&state, &actor, &to_actor);
            assert_eq!(allowance_balance, amount);

            let value = BigUint::parse_bytes(b"454634729034", 10).unwrap();
            let amount = Uint256 {
                big_uint: value.clone(),
            };
            let from_old_balance = balance_of(&state, actor);
            let to_old_balance = balance_of(&state, to_actor);
            let allowance_old = allowance(&state, &actor, &to_actor);
            transfer_from(&mut state, actor, to_actor, amount.clone());
            let from_balance = balance_of(&state, actor);
            let to_balance = balance_of(&state, to_actor);
            let allowance = allowance(&state, &actor, &to_actor);
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

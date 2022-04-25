use crate::blockstore::Blockstore;
use crate::types::{MintLog, TransferLog};
use crate::uint256::Uint256;
use cid::multihash::Code;
use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_encoding::{to_vec, CborStore, RawBytes, DAG_CBOR};
use fvm_sdk as sdk;
use fvm_shared::ActorID;
use std::collections::HashMap;
#[macro_use]
use crate::abort;
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

    pub fn constructor(&mut self, token: Token) -> Option<RawBytes> {
        self.token = token;
        self.balance_of = HashMap::new();
        self.allowance = HashMap::new();
        None
    }

    pub fn symbol(&self) -> String {
        self.token.symbol.clone()
    }

    pub fn decimal(&self) -> u64 {
        self.token.decimal
    }

    pub fn total_supply(&self) -> Uint256 {
        self.token.total_supply.clone()
    }

    pub fn mint(&mut self, actor: ActorID, amount: Uint256) -> MintLog {
        let mut default_balance: Uint256 = Default::default();
        match self.balance_of.get_mut(&actor) {
            None => {
                self.balance_of.insert(actor, amount.clone());
            }
            Some(balance) => {
                *balance = balance.clone() + amount.clone();
            }
        };

        MintLog::new(actor, amount)
    }

    pub fn balance_of(&self, actor: ActorID) -> Uint256 {
        let default_balance = Uint256::default();
        let balacnce = self.balance_of.get(&actor).unwrap_or(&default_balance);
        balacnce.clone()
    }

    pub fn transfer(&mut self, to: ActorID, amount: Uint256) -> TransferLog {
        unsafe {
            let from = sdk::sys::message::caller().unwrap();

            let mut from_default_balance = Uint256::default();

            let from_balance = self.balance_of.get_mut(&from);
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

            match self.balance_of.get_mut(&to) {
                None => {
                    self.balance_of.insert(to, amount.clone());
                }
                Some(to_balance) => {
                    *to_balance = to_balance.clone() + amount.clone();
                }
            }

            TransferLog::new(from, to, amount)
        }
    }

    pub fn approve(&mut self, to: ActorID, amount: Uint256) -> Option<RawBytes> {
        unsafe {
            let from = sdk::sys::message::caller().unwrap();

            let default_balance = Uint256::default();
            let from_balance = self
                .balance_of
                .get(&from)
                .unwrap_or(&default_balance)
                .clone();
            if from_balance < amount {
                abort!(SYS_ASSERTION_FAILED, "Insufficient Balance")
            }
            if let None = self.allowance.get_mut(&from) {
                self.allowance.insert(from, HashMap::new());
            }
            self.allowance.get_mut(&from).unwrap().insert(to, amount);
            None
        }
    }

    pub fn allowance(&self, from: &ActorID, to: &ActorID) -> Uint256 {
        match self.allowance.get(from) {
            None => return Uint256::default(),
            Some(allow) => match allow.get(to) {
                None => return Uint256::default(),
                Some(balance) => return balance.clone(),
            },
        }
    }

    pub fn transfer_from(&mut self, from: ActorID, to: ActorID, amount: Uint256) -> TransferLog {
        unsafe {
            match self.allowance.get_mut(&from) {
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
                            let from_balance = self.balance_of.get_mut(&from).unwrap();
                            *from_balance = from_balance.clone() - amount.clone();

                            match self.balance_of.get_mut(&to) {
                                None => {
                                    self.balance_of.insert(to, amount.clone());
                                }
                                Some(to_balance) => {
                                    *to_balance = to_balance.clone() + amount.clone();
                                    *value = value.clone() - amount.clone();
                                }
                            }
                        }
                    }
                }
            };

            TransferLog::new(from, to, amount)
        }
    }
}

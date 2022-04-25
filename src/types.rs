use crate::uint256::Uint256;
use fvm_shared::ActorID;
use serde::{Deserialize, Serialize};
use serde_json::Error;
#[derive(Serialize, Deserialize, Debug)]
pub struct Mint {
    pub actor: ActorID,
    pub amount: Uint256,
}

impl Mint {
    pub fn from_slice(data: &[u8]) -> Result<Self, Error> {
        serde_json::from_slice::<Self>(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transfer {
    pub to: ActorID,
    pub amount: Uint256,
}

impl Transfer {
    pub fn from_slice(data: &[u8]) -> Result<Self, Error> {
        serde_json::from_slice::<Self>(data)
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Approve {
    pub actor: ActorID,
    pub amount: Uint256,
}

impl Approve {
    pub fn from_slice(data: &[u8]) -> Result<Self, Error> {
        serde_json::from_slice::<Self>(data)
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Allowance {
    pub from: ActorID,
    pub to: ActorID,
}

impl Allowance {
    pub fn from_slice(data: &[u8]) -> Result<Self, Error> {
        serde_json::from_slice::<Self>(data)
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TransferFrom {
    pub from: ActorID,
    pub to: ActorID,
    pub amount: Uint256,
}

impl TransferFrom {
    pub fn from_slice(data: &[u8]) -> Result<Self, Error> {
        serde_json::from_slice::<Self>(data)
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MintLog {
    actor: ActorID,
    amount: Uint256,
}

impl MintLog {
    pub fn new(actor: ActorID, amount: Uint256) -> Self {
        Self {
            actor: actor,
            amount: amount,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferLog {
    from: ActorID,
    to: ActorID,
    amount: Uint256,
}

impl TransferLog {
    pub fn new(from: ActorID, to: ActorID, amount: Uint256) -> Self {
        Self {
            from: from,
            to: to,
            amount: amount,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

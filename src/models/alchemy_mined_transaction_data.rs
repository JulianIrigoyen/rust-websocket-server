use serde::{Deserialize, Serialize};
use timely::dataflow::operators::{Inspect, Map, ToStream};
use timely::dataflow::scopes::Scope;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlchemyMinedTransactionData {
    pub removed: bool,
    pub transaction: Option<AlchemyTransaction>,
    pub hash: String,
    pub subscription: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlchemyTransaction {
    pub blockHash: Option<String>,
    pub blockNumber: Option<String>,
    pub from: Option<String>,
    pub gas: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>,
}

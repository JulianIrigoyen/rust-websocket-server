use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub enum EthTransactionTypes {
    Erc20Transfer(ERC20TransferInput),
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ERC20TransferInput {
    pub to: String,
    pub value: u64,
}


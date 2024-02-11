use crate::models::alchemy::alchemy_mined_transaction_data::AlchemyMinedTransactionData;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AlchemyEventTypes {
    AlchemyMinedTransactions(AlchemyMinedTransactionData),
}

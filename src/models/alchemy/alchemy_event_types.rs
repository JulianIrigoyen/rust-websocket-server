use crate::models::alchemy::alchemy_mined_transaction_data::AlchemyMinedTransactionData;

#[derive(Debug, Clone)]
pub enum AlchemyEventTypes {
    AlchemyMinedTransactions(AlchemyMinedTransactionData),
}

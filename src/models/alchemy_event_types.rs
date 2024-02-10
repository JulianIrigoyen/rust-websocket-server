use crate::models::alchemy_mined_transaction_data::AlchemyMinedTransactionData;

#[derive(Debug, Clone)]
pub enum AlchemyEventTypes {
    AlchemyMinedTransactions(AlchemyMinedTransactionData),
}

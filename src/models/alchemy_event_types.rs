use crate::models::alchemy_mined_transaction_data::AlchemyMinedTransactionData;
use crate::models::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
use crate::models::polygon_crypto_level2_book_data::PolygonCryptoLevel2BookData;
use crate::models::polygon_crypto_quote_data::PolygonCryptoQuoteData;
use crate::models::polygon_crypto_trade_data::PolygonCryptoTradeData;

#[derive(Debug, Clone)]
pub enum AlchemyEventTypes {
    AlchemyMinedTransactions(AlchemyMinedTransactionData),
}

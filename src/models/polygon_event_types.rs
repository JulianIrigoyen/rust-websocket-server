use crate::models::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
use crate::models::polygon_crypto_level2_book_data::PolygonCryptoLevel2BookData;
use crate::models::polygon_crypto_quote_data::PolygonCryptoQuoteData;
use crate::models::polygon_crypto_trade_data::PolygonCryptoTradeData;

#[derive(Debug, Clone)]
pub enum PolygonEventTypes {
    XaAggregateMinute(PolygonCryptoAggregateData),
    XasAggregateSecond(PolygonCryptoAggregateData),
    XtTrade(PolygonCryptoTradeData),
    XqQuote(PolygonCryptoQuoteData),
    Xl2Level2book(PolygonCryptoLevel2BookData),
}

// Import necessary Diesel macros and types
use diesel::prelude::*;

// Define Diesel models for your structs
table! {
    polygon_crypto_level2_book_data {
        id -> Integer,
        event_type -> Varchar,
        pair -> Varchar,
        timestamp -> Bigint,
        received_timestamp -> Bigint,
        exchange_id -> Bigint,
        bid_prices -> Jsonb,
        ask_prices -> Jsonb,
    }
}

table! {
    polygon_crypto_aggregate_data {
        id -> Integer,
        event_type -> Varchar,
        pair -> Varchar,
        open -> Double,
        close -> Double,
        high -> Double,
        low -> Double,
        volume -> Double,
        timestamp -> Bigint,
        end_time -> Bigint,
        vw -> Double,
        avg_trade_size -> Bigint,
    }
}

table! {
    polygon_crypto_quote_data {
        id -> Integer,
        event_type -> Varchar,
        pair -> Varchar,
        bid_price -> Double,
        bid_size -> Double,
        ask_price -> Double,
        ask_size -> Double,
        timestamp -> Bigint,
        exchange_id -> Bigint,
        received_timestamp -> Bigint,
    }
}

table! {
    polygon_crypto_trade_data {
        id -> Integer,
        event_type -> Varchar,
        pair -> Varchar,
        price -> Double,
        timestamp -> Bigint,
        size -> Double,
        conditions -> Jsonb,
        trade_id -> Nullable<Varchar>,
        exchange_id -> Bigint,
        received_timestamp -> Bigint,
    }
}

// Define ToDiesel trait for converting your structs to corresponding Diesel models
pub trait ToDiesel {
    type DieselModel;

    fn to_diesel(self) -> Self::DieselModel;
}

impl ToDiesel for PolygonCryptoLevel2BookData {
    type DieselModel = polygon_crypto_level2_book_data::New;

    fn to_diesel(self) -> Self::DieselModel {
        polygon_crypto_level2_book_data::New {
            event_type: self.event_type,
            pair: self.pair,
            timestamp: self.timestamp,
            received_timestamp: self.received_timestamp,
            exchange_id: self.exchange_id,
            bid_prices: serde_json::to_value(self.bid_prices).unwrap(),
            ask_prices: serde_json::to_value(self.ask_prices).unwrap(),
        }
    }
}



// Similarly, implement ToDiesel for other structs

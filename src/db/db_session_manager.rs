use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::Error;
use r2d2::{Pool, PooledConnection};

use crate::models::*;
use crate::models::polygon_event_types::PolygonEventTypes;
use crate::ToDiesel;

impl DbSessionManager {
    pub fn new() -> Self {
        // Load the database connection URL from the environment using dotenv_codegen
        let database_url = dotenv!("DATABASE_URL");

        // Create a connection manager for the PostgreSQL database
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        // Create a connection pool using the connection manager
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create database connection pool");

        DbSessionManager { pool }
    }

    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, diesel::r2d2::PoolError> {
        self.pool.get()
    }
    pub fn persist_event(&self, event: &PolygonEventTypes) -> Result<(), Error> {
        let connection = self.get_connection()?;

        // Match on the event type to determine which table to insert into
        match event {
            PolygonEventTypes::Level2BookData(level2_book_data) => {
                diesel::insert_into(polygon_crypto_level2_book_data::table)
                    .values(level2_book_data.to_diesel())
                    .execute(&mut connection)?;
            }
            PolygonEventTypes::AggregateData(aggregate_data) => {
                diesel::insert_into(polygon_crypto_aggregate_data::table)
                    .values(aggregate_data.to_diesel())
                    .execute(&mut connection)?;
            }
            PolygonEventTypes::QuoteData(quote_data) => {
                diesel::insert_into(polygon_crypto_quote_data::table)
                    .values(quote_data.to_diesel())
                    .execute(&mut connection)?;
            }
            PolygonEventTypes::TradeData(trade_data) => {
                diesel::insert_into(polygon_crypto_trade_data::table)
                    .values(trade_data.to_diesel())
                    .execute(&mut connection)?;
            }
        }

        Ok(())
    }
}

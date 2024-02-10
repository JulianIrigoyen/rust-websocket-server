use std::sync::Arc;

use ethers::types::U256;

use crate::DbSessionManager;
use crate::models::alchemy_event_types::AlchemyEventTypes;
use crate::models::alchemy_mined_transaction_data::AlchemyMinedTransactionData;

pub struct AlchemyWhaleTracker {
    db_session_manager: Arc<DbSessionManager>,
    whale_threshold: U256, //in Eth
}

impl AlchemyWhaleTracker {
    pub fn new(db_session_manager: Arc<DbSessionManager>, whale_threshold_eth: f64) -> Self {
        let whale_threshold = U256::from(10u64.pow(18)) * U256::from(whale_threshold_eth as u64);

        AlchemyWhaleTracker {
            db_session_manager,
            whale_threshold,
        }
    }

    pub fn process(&self, data: &AlchemyMinedTransactionData) {
        // If the transaction meets the criteria, store it using the session manager
        if self.meets_whale_criteria(data) {
            println!("WHALE ALERT !");
            // self.db_session_manager.persist_event(); // Assuming this method exists
        }

        data.params.result.transaction.print_transaction_details();
    }

    pub fn meets_whale_criteria(&self, data: &AlchemyMinedTransactionData) -> bool {
        // Directly access the transaction since we're not matching against a pattern
        let transaction = &data.params.result.transaction;

        // Then, you can directly access `value` within `transaction`
        if let Some(value_str) = &transaction.value {
            let value = U256::from_str_radix(value_str.trim_start_matches("0x"), 16)
                .unwrap_or_else(|_| U256::zero());
            return value > self.whale_threshold;
        }

        false
    }
}

impl AlchemyWhaleTracker {
    pub(crate) fn apply(&self, event: &AlchemyEventTypes) {
        match event {
            AlchemyEventTypes::AlchemyMinedTransactions(data) => self.process(data),
        }
    }
}
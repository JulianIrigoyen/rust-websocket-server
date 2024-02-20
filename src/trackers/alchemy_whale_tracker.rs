use std::sync::Arc;

use ethers::types::U256;

use crate::DbSessionManager;
use crate::http::moralis_http_client::MoralisHttpClient;
use crate::models::alchemy::alchemy_event_types::AlchemyEventTypes;
use crate::models::alchemy::alchemy_mined_transaction_data::AlchemyMinedTransactionData;
use crate::util::eth_tx_decoder::EthTxDecoder;
use crate::util::eth_tx_decoder::TokenInfoWithPrice;

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

    pub(crate) fn apply(&self, event: &AlchemyEventTypes) {
        match event {
            AlchemyEventTypes::AlchemyMinedTransactions(data) => self.process_working(data),
        }
    }

    pub fn process_working(&self, data: &AlchemyMinedTransactionData) {
        let decoder = EthTxDecoder::new();
        // Check for ERC-20 transfer transactions where the value is 0x0
        if let Some(value_str) = &data.params.result.transaction.value {
            let value = U256::from_str_radix(value_str.trim_start_matches("0x"), 16).unwrap_or_else(|_| U256::zero());

            if self.meets_whale_criteria(data) {
                data.params.result.transaction.print_transaction_details();
                // self.db_session_manager.persist_event(); // Assuming this method exists
            }

            // Process only if transaction value is 0 (potential ERC-20 transfer)
            if value.is_zero() {
                if let Some(input) = &data.params.result.transaction.input {
                    if let Some((recipient, amount)) = decoder.decode_tx_input(input) {
                        // Check if the amount exceeds the whale threshold
                        if amount > self.whale_threshold {
                            println!("ERC-20 Transfer Detected");
                            println!("From: {}, To: {}, Amount: {}", data.params.result.transaction.from.as_ref().unwrap_or(&"Unknown".to_string()), recipient, amount);
                        }
                    }
                }
            }
        }
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
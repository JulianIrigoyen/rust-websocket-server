use std::sync::Arc;

use ethers::types::U256;

use crate::DbSessionManager;
use crate::http::moralis_http_client::MoralisHttpClient;
use crate::models::alchemy_event_types::AlchemyEventTypes;
use crate::models::alchemy_mined_transaction_data::AlchemyMinedTransactionData;
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

    pub fn process_deprecated(&self, data: &AlchemyMinedTransactionData) {
        // If the transaction meets the criteria, store it using the session manager
        if self.meets_whale_criteria(data) {
            data.params.result.transaction.print_transaction_details();
            // self.db_session_manager.persist_event(); // Assuming this method exists
        }
    }

    pub async fn process_async(&self, data: &AlchemyMinedTransactionData) {
        let decoder = EthTxDecoder::new();
        if let Some(input) = &data.params.result.transaction.input {
            // Assuming `chain` is known and static, e.g., "eth".
            if let Some(token_info_with_price) = decoder.decode_tx_input(input) {

                println!("ERC-20 Transfer Detected with Price Info {:?}", token_info_with_price);
                // println!("Token Address: {}, Recipient: {}, Amount: {}, USD Price: {}", token_info_with_price.token_address, token_info_with_price.recipient, token_info_with_price.amount, token_info_with_price.usd_price);
            }
        }
    }

    pub async fn process_new(&self, data: &AlchemyMinedTransactionData) {
        let decoder = EthTxDecoder::new();

        // Check for ERC-20 transfer transactions where the value is 0x0
        if let Some(value_str) = &data.params.result.transaction.value {
            let value = U256::from_str_radix(value_str.trim_start_matches("0x"), 16).unwrap_or_else(|_| U256::zero());

            // Process only if transaction value is 0 (potential ERC-20 transfer)
            if value.is_zero() {
                if let Some(input) = &data.params.result.transaction.input {
                    if let Some(token_info_with_price) = decoder.decode_tx_input_and_fetch_price(input, "eth").await {
                        println!("ERC-20 Transfer Detected with Price Info");
                        println!("Token Address: {}, Recipient: {}, Amount: {}, USD Price: {}", token_info_with_price.token_address, token_info_with_price.recipient, token_info_with_price.amount, token_info_with_price.usd_price);
                    }
                }
            } else {
                // For non-ERC-20 transfers (ETH transfers), check if the value exceeds the whale threshold
                if value > self.whale_threshold {
                    println!("WHALE ALERT! ETH Transfer Detected");
                    println!("From: {}, To: {}, Value: {}", data.params.result.transaction.from.as_ref().unwrap_or(&"Unknown".to_string()), data.params.result.transaction.to.as_ref().unwrap_or(&"Unknown".to_string()), value);
                }
            }
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

impl AlchemyWhaleTracker {
    pub(crate) fn apply(&self, event: &AlchemyEventTypes) {
        match event {
            AlchemyEventTypes::AlchemyMinedTransactions(data) => self.process_working(data),
        }
    }
}
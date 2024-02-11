use ethers::types::U256;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlchemyTransaction {
    pub block_hash: Option<String>,
    pub block_number: Option<String>,
    pub from: Option<String>,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
    pub hash: String,
    pub input: Option<String>,
    pub nonce: Option<String>,
    pub to: Option<String>,
    pub transaction_index: Option<String>,
    pub value: Option<String>,
    pub v: Option<String>,
    pub r: Option<String>,
    pub s: Option<String>,
    // Consider adding type, maxFeePerGas, maxPriorityFeePerGas for EIP-1559 transactions
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlchemySubscriptionResult {
    pub removed: bool,
    pub transaction: AlchemyTransaction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlchemySubscriptionParams {
    pub result: AlchemySubscriptionResult,
    pub subscription: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlchemyMinedTransactionData {
    pub jsonrpc: String,
    pub method: String,
    pub params: AlchemySubscriptionParams,
}

impl AlchemyTransaction {
    // This method prints out the detailed transaction information with value in ETH
    pub fn print_transaction_details(&self) {
        // Parse the hexadecimal value to U256
        let value_wei = U256::from_str_radix(self.value.as_deref().unwrap_or("0x0").trim_start_matches("0x"), 16)
            .unwrap_or_else(|_| U256::zero());


        // Convert Wei to ETH by dividing by 10^18
        let value_eth = if value_wei > U256::zero() {
            let eth_float: f64 = value_wei.as_u128() as f64 / 1e18;
            format!("{:.18}", eth_float).trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            "0".to_string() // Directly handle the zero case to avoid displaying "0.000..."
        };

        println!("Transaction Details:");
        println!("From Address: {}", self.from.as_ref().unwrap());
        println!("To Address: {}", self.to.as_ref().unwrap());
        println!("Value: {} ETH", value_eth);
    }
}


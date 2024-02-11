use std::collections::HashMap;
use std::str::FromStr;

use ethers::types::{Address, U256};

use crate::http::moralis_http_client::MoralisHttpClient;

pub struct EthTxDecoder {
    known_token_addresses: std::collections::HashMap<Address, String>,
}

impl EthTxDecoder {

    const transfer_keccak_signature: &str = "0xa9059cbb";

    pub fn new() -> Self {
        let mut known_token_addresses = HashMap::new();
        known_token_addresses.insert(Address::from_str("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE").unwrap(), "ETH".to_string());
        known_token_addresses.insert(Address::from_str("0x514910771AF9Ca656af840dff83E8264EcF986CA").unwrap(), "LINK".to_string());
        known_token_addresses.insert(Address::from_str("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984").unwrap(), "UNI".to_string());
        known_token_addresses.insert(Address::from_str("0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9").unwrap(), "AAVE".to_string());
        known_token_addresses.insert(Address::from_str("0xc00e94Cb662C3520282E6f5717214004A7f26888").unwrap(), "COMP".to_string());
        known_token_addresses.insert(Address::from_str("0x6B3595068778DD592e39A122f4f5a5cF09C90fE2").unwrap(), "SUSHI".to_string());
        known_token_addresses.insert(Address::from_str("0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2").unwrap(), "MKR".to_string());
        known_token_addresses.insert(Address::from_str("0xC011a73ee8576Fb46F5E1c5751cA3B9Fe0af2a6F").unwrap(), "SNX".to_string());
        known_token_addresses.insert(Address::from_str("0x0bc529c00C6401aEF6D220BE8C6Ea1667F6Ad93e").unwrap(), "YFI".to_string());
        known_token_addresses.insert(Address::from_str("0x5d3a536E4D6DbD6114cc1Ead35777bAB948E3643").unwrap(), "cDAI".to_string());

        EthTxDecoder { known_token_addresses }
    }


    pub fn decode_tx_input(&self, input: &str) -> Option<(Address, U256)> {
        match input.get(..10) {
            Some(transfer_keccak_signature) => self.decode_erc20_transfer(input),
            _ => None, // Extend this match arm to handle other types
        }
    }

    /**
    ERC-20 transfer input data format is 0xa9059cbb followed by two 32-byte arguments:

        Recipient Address: Padded to 32 bytes, with the actual address occupying the last 20 bytes.
        Amount: The transfer amount, encoded as a 32-byte unsigned integer.

    Recipient Address: The decoder skips the first 10 characters (representing the method selector) and reads the next 64 characters for the recipient address. Since Ethereum addresses are 20 bytes (40 characters) and are right-padded in the 32-byte argument, it further extracts the last 40 characters of this segment.
    Amount: The decoder reads the following 64 characters for the amount, which represents the transfer amount in hexadecimal. It converts this hexadecimal string to a U256 value.
     */

    fn decode_erc20_transfer(&self, input: &str) -> Option<(Address, U256)> {
        println!("Decoding ERC20 Transfer: {:?}", input);

        if input.len() < 138 {
            println!("Invalid input length for ERC-20 transfer.");
            return None;
        }


        let recipient_hex = &input[10..74]; // Correct start position for recipient
        let amount_hex = &input[74..138]; // Correct start position for amount

        let recipient = Address::from_str(&format!("0x{}", &recipient_hex[24..])).ok()?;
        let amount = U256::from_str_radix(&amount_hex, 16).ok()?;

        println!("{}", format!("decoded: {} received {} ", recipient, amount));

        Some((recipient, amount))
    }

    pub async fn decode_tx_input_and_fetch_price(&self, input: &str, chain: &str) -> Option<TokenInfoWithPrice> {
        if let Some((token_address, amount)) = self.decode_tx_input(input) {
            // Check if the token address is known and skip the API call if so
            if let Some(symbol) = self.known_token_addresses.get(&token_address) {
                println!("Token symbol found in known addresses: {}", symbol);
                // Assume a default or cached price if not fetching
                return Some(TokenInfoWithPrice {
                    token_address,
                    recipient: token_address,
                    amount,
                    usd_price: 0.0, // Placeholder for the price
                });
            } else {
                // Fetch the price if the token address is not known
                if let Ok(price_response) = MoralisHttpClient::new("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJub25jZSI6ImEyNTg2MGQ3LTlmZmEtNDgzZS1iYTc4LTE1NjAxNGFmMjU4ZCIsIm9yZ0lkIjoiMzczMDA5IiwidXNlcklkIjoiMzgzMzM5IiwidHlwZUlkIjoiZGZlOTE4NWMtMzRiYi00ZjY4LWJjODgtMGMwNmE2ZDQyMmM1IiwidHlwZSI6IlBST0pFQ1QiLCJpYXQiOjE3MDU1ODIwOTgsImV4cCI6NDg2MTM0MjA5OH0.hnF-gsJfACVNfbAWUWrVvfhRnbpmfcIt_oF_wlyoDRo").get_token_price(&token_address.to_string(), chain, true).await {
                    return Some(TokenInfoWithPrice {
                        token_address,
                        recipient: token_address,
                        amount,
                        usd_price: price_response.usd_price,
                    });
                }
            }
        }
        None
    }
}

pub struct TokenInfoWithPrice {
    pub token_address: Address,
    pub recipient: Address,
    pub amount: U256,
    pub usd_price: f64,
}

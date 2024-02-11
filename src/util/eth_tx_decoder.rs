use ethers::types::{Address, U256};
use std::str::FromStr;

pub struct EthTxDecoder;

impl EthTxDecoder {
    pub fn new() -> Self {
        EthTxDecoder {}
    }

    pub fn decode_tx_input(&self, input: &str) -> Option<(Address, U256)> {
        match input.get(..10) {
            Some("0xa9059cbb") => self.decode_erc20_transfer(input),
            _ => None, // Extend this match arm to handle other types
        }
    }

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
}

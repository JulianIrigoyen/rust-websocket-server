use std::collections::HashMap;
use std::str::FromStr;

use ethers::types::{Address, U256};

use crate::http::moralis_http_client::MoralisHttpClient;

/**

ERC-20 token transfers are identified by the input data starting with specific method signatures (also known as function selectors).
    These signatures are derived from the first four bytes of the hash of the function's signature.
    For ERC-20 tokens, the most crucial signatures to be aware of are for the transfer and transferFrom functions, as these are standard methods defined in the ERC-20 token standard for transferring tokens.

Key ERC-20 Method Signatures:

    Method Signature: transfer(address _to, uint256 _value)
    Hash (Keccak-256): a9059cbb
    Description: This method is used to transfer _value amount of tokens to the address _to. The input data for a transaction calling this method will start with 0xa9059cbb.
    transferFrom(address,address,uint256):

    Method Signature: transferFrom(address _from, address _to, uint256 _value)
    Hash (Keccak-256): 23b872dd
    Description: This method is used to transfer _value amount of tokens from the address _from to the address _to, typically requiring prior approval from _from. The input data for a transaction calling this method will start with 0x23b872dd.
    Other Important ERC-20 Signatures:
    approve(address,uint256):

    Method Signature: approve(address _spender, uint256 _value)
    Hash (Keccak-256): 095ea7b3
    Description: This method is used to allow _spender to withdraw up to _value amount of tokens on your behalf. The input data for a transaction calling this method will start with 0x095ea7b3.
    allowance(address,address):

    Method Signature: allowance(address _owner, address _spender)
    Hash (Keccak-256): dd62ed3e
    Description: This method returns the remaining number of tokens that _spender is allowed to withdraw from _owner. This does not typically initiate transactions but is essential for understanding token allowances.
    balanceOf(address):

    Method Signature: balanceOf(address _owner)
    Hash (Keccak-256): 70a08231
    Description: This method returns the token balance of _owner. Like allowance, it is used for querying state and not for initiating transactions.
 */

enum Erc20MethodSignatures {
    Transfer,
    TransferFrom,
    Approve,
    Allowance,
    BalanceOf
}

use lazy_static::lazy_static;

lazy_static! {
    static ref SIGNATURES: HashMap<&'static str, Erc20MethodSignatures> = {
        let mut m = HashMap::new();
        m.insert("0xa9059cbb", Erc20MethodSignatures::Transfer);
        m.insert("0x23b872dd", Erc20MethodSignatures::TransferFrom);
        m.insert("0x095ea7b3", Erc20MethodSignatures::Approve);
        m.insert("0xdd62ed3e", Erc20MethodSignatures::Allowance);
        m.insert("0x70a08231", Erc20MethodSignatures::BalanceOf);
        m
    };
}

pub struct EthTxDecoder {
    known_token_addresses: std::collections::HashMap<Address, String>,
}

impl EthTxDecoder {

    const TRANSFER_KECCAK_SIGNATURE: &str = "0xa9059cbb";
    const TRANSFER_FROM_KECCAK_SIGNATURE: &str = "23b872dd";
    const APPROVE_KECCAK_SIGNATURE: &str = "095ea7b3";
    const ALLOWANCE_KECCAK_SIGNATURE: &str = "dd62ed3e";
    const BALANCE_OF_KECCAK_SIGNATURE: &str = "dd62ed3e";

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
        let method_selector = input.get(..10)?;
        if let Some(function) = SIGNATURES.get(method_selector) {
            match function {
                Erc20MethodSignatures::Transfer => self.decode_erc20_transfer_tx(input),
                Erc20MethodSignatures::TransferFrom => self.decode_erc20_transfer_from_tx(input),
                Erc20MethodSignatures::Approve => self.decode_erc20_approve_tx(input),
                Erc20MethodSignatures::Allowance => self.decode_erc20_allowance_tx(input),
                Erc20MethodSignatures::BalanceOf => self.decode_erc20_balance_of_tx(input),
            }
        } else {
            None
        }
    }

    /**
    ERC-20 transfer input data format is 0xa9059cbb followed by two 32-byte arguments:

        Recipient Address: Padded to 32 bytes, with the actual address occupying the last 20 bytes.
        Amount: The transfer amount, encoded as a 32-byte unsigned integer.

        Recipient Address: The decoder skips the first 10 characters (representing the method selector) and reads the next 64 characters for the recipient address. Since Ethereum addresses are 20 bytes (40 characters) and are right-padded in the 32-byte argument, it further extracts the last 40 characters of this segment.
        Amount: The decoder reads the following 64 characters for the amount, which represents the transfer amount in hexadecimal. It converts this hexadecimal string to a U256 value.
     */

    fn decode_erc20_transfer_tx(&self, input: &str) -> Option<(Address, U256)> {
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

    fn decode_erc20_transfer_from_tx(&self, input: &str) -> Option<(Address, U256)> {
        println!("Decoding ERC20 Transfer From transaction: {:?}", input);
        None
    }

    fn decode_erc20_approve_tx(&self, input: &str) -> Option<(Address, U256)> {
        println!("Decoding ERC20 Approve transaction: {:?}", input);
        None
    }

    fn decode_erc20_allowance_tx(&self, input: &str) -> Option<(Address, U256)> {
        println!("Decoding ERC20 Allowance transaction: {:?}", input);
        None
    }

    fn decode_erc20_balance_of_tx(&self, input: &str) -> Option<(Address, U256)> {
        println!("Decoding ERC20 Balance Of transaction: {:?}", input);
        None
    }
}

pub struct TokenInfoWithPrice {
    pub token_address: Address,
    pub recipient: Address,
    pub amount: U256,
    pub usd_price: f64,
}


//  TODO : figure out which tokens are involved...
// pub async fn decode_tx_input_and_fetch_price(&self, input: &str, chain: &str) -> Option<TokenInfoWithPrice> {
//     if let Some((token_address, amount)) = self.decode_tx_input(input) {
//         // Check if the token address is known and skip the API call if so
//         if let Some(symbol) = self.known_token_addresses.get(&token_address) {
//             println!("Token symbol found in known addresses: {}", symbol);
//             // Assume a default or cached price if not fetching
//             return Some(TokenInfoWithPrice {
//                 token_address,
//                 recipient: token_address,
//                 amount,
//                 usd_price: 0.0, // Placeholder for the price
//             });
//         } else {
//             // Fetch the price if the token address is not known
//             if let Ok(price_response) = MoralisHttpClient::new("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJub25jZSI6ImEyNTg2MGQ3LTlmZmEtNDgzZS1iYTc4LTE1NjAxNGFmMjU4ZCIsIm9yZ0lkIjoiMzczMDA5IiwidXNlcklkIjoiMzgzMzM5IiwidHlwZUlkIjoiZGZlOTE4NWMtMzRiYi00ZjY4LWJjODgtMGMwNmE2ZDQyMmM1IiwidHlwZSI6IlBST0pFQ1QiLCJpYXQiOjE3MDU1ODIwOTgsImV4cCI6NDg2MTM0MjA5OH0.hnF-gsJfACVNfbAWUWrVvfhRnbpmfcIt_oF_wlyoDRo").get_token_price(&token_address.to_string(), chain, true).await {
//                 return Some(TokenInfoWithPrice {
//                     token_address,
//                     recipient: token_address,
//                     amount,
//                     usd_price: price_response.usd_price,
//                 });
//             }
//         }
//     }
//     None
// }
use anyhow::Error;
use ethers::prelude::*;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider, Ws};
use ethers::signers::{LocalWallet, Signer};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;
use dotenv::dotenv;
use ethers::types::{Address, Filter};
use std::env;
use futures::StreamExt;

// Define the Transfer event using abigen!
abigen!(
    PABERC20,
    r#"
        [{"inputs":[{"internalType":"string","name":"name","type":"string"},{"internalType":"string","name":"symbol","type":"string"},{"internalType":"uint8","name":"decimals","type":"uint8"},{"internalType":"uint256","name":"totalSupply","type":"uint256"}],"payable":true,"stateMutability":"payable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"owner","type":"address"},{"indexed":true,"internalType":"address","name":"spender","type":"address"},{"indexed":false,"internalType":"uint256","name":"value","type":"uint256"}],"name":"Approval","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"from","type":"address"},{"indexed":true,"internalType":"address","name":"to","type":"address"},{"indexed":false,"internalType":"uint256","name":"value","type":"uint256"}],"name":"Transfer","type":"event"},{"constant":true,"inputs":[{"internalType":"address","name":"owner","type":"address"},{"internalType":"address","name":"spender","type":"address"}],"name":"allowance","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"spender","type":"address"},{"internalType":"uint256","name":"value","type":"uint256"}],"name":"approve","outputs":[{"internalType":"bool","name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"internalType":"address","name":"account","type":"address"}],"name":"balanceOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"uint256","name":"value","type":"uint256"}],"name":"burn","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"internalType":"uint8","name":"","type":"uint8"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"spender","type":"address"},{"internalType":"uint256","name":"subtractedValue","type":"uint256"}],"name":"decreaseAllowance","outputs":[{"internalType":"bool","name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"spender","type":"address"},{"internalType":"uint256","name":"addedValue","type":"uint256"}],"name":"increaseAllowance","outputs":[{"internalType":"bool","name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"name","outputs":[{"internalType":"string","name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"internalType":"string","name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"recipient","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"transfer","outputs":[{"internalType":"bool","name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"sender","type":"address"},{"internalType":"address","name":"recipient","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"transferFrom","outputs":[{"internalType":"bool","name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"}]    
    "#,
);

// Use abigen macro to generate contract bindings
abigen!(
    PabLedgerContract,
    r#"
        [{"inputs":[],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"account","type":"address"},{"indexed":false,"internalType":"uint256","name":"balance","type":"uint256"}],"name":"BalanceUpdated","type":"event"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"balances","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"account","type":"address"}],"name":"getBalance","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"owner","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"newOwner","type":"address"}],"name":"transferOwnership","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"account","type":"address"},{"internalType":"uint256","name":"balance","type":"uint256"}],"name":"updateBalance","outputs":[],"stateMutability":"nonpayable","type":"function"}]    
    "#
);
abigen!(
    PabKOLStakingContract,
    r#"
        [{"inputs":[],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"user","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"timestamp","type":"uint256"}],"name":"Staked","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"user","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"timestamp","type":"uint256"}],"name":"Withdrawn","type":"event"},{"inputs":[{"internalType":"address","name":"_user","type":"address"}],"name":"getInfoOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"},{"internalType":"uint256","name":"","type":"uint256"},{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"owner","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"_owner","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"stake","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"stakes","outputs":[{"internalType":"uint256","name":"amount","type":"uint256"},{"internalType":"uint256","name":"since","type":"uint256"},{"internalType":"bool","name":"staked","type":"bool"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"_user","type":"address"}],"name":"stakesOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"newOwner","type":"address"}],"name":"transferOwnership","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"_owner","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"withdraw","outputs":[],"stateMutability":"nonpayable","type":"function"},{"stateMutability":"payable","type":"receive"}]
    "#
);

const PAB_BALANCE_LEDGER_CONTRACT: &str = "0x5C98D79e6Ce7299a2Ea84B2898eAF064038AA1f3";
const PAB_STAKING_CONTRACT: &str = "0x4fc96644264Dba5630cdcc4b7696A3f7b20d4471";
const PAB_TOKEN_CONTRACT: &str = "0xD6311f9A6bd3a802263F4cd92e2729bC2C31Ed23";
const BSC_WSS_URL: &str = "wss://bsc-mainnet.infura.io/ws/v3/7dec7de5256648e0bc864fbe224addeb";
const BSC_HTTP_URL: &str = "https://bsc-mainnet.infura.io/v3/7dec7de5256648e0bc864fbe224addeb";
const PAB_TRANSFER_SIG: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

#[derive(Clone, Debug, Serialize, Deserialize, EthEvent)]
pub struct Transfer {
    #[ethevent(indexed)]
    pub from: Address,
    #[ethevent(indexed)]
    pub to: Address,
    pub tokens: U256,
}

pub async fn monitor_pab_transfer_event() -> Result<(), Error> {
    println!("listen for {} events", PAB_TOKEN_CONTRACT);

    let token_topics = [
        H256::from(PAB_TRANSFER_SIG.parse::<H160>().expect("wrong sig")),
        H256::from(PAB_STAKING_CONTRACT.parse::<H160>().expect("wrong address to")),
    ];
    
    let filter = Filter::new()
        .topic0(token_topics.to_vec())  // Monitor by the Transfer event signature
        .topic2(token_topics.to_vec());

    match Provider::<Ws>::connect(BSC_WSS_URL).await{
        Ok(provider) => {
            println!("Connected to BSC mainnet");
            let provider = Arc::new(provider);
            let event = Transfer::new::<_, Provider<Ws>>(filter, Arc::clone(&provider));
            let event_sub = event.subscribe().await;
            match event_sub{
                Ok(mut transfers) => {
                    println!("Subscribed to {:?}", event);
                    while let Some(log) = transfers.next().await {
                        println!("Transfer: {:?}", log);
                        // proxy_contract_call_kol_staking(log.from.to_string(), 10000).await?;
                    }
                }
                Err(e) => println!("Error subscribing to Transfer event: {:?}", e),
            }
        }
        Err(e) => println!("Error connecting to BSC mainnet: {:?}", e),
    }

    Ok(())
}

async fn proxy_contract_call_update_balance(account: String, update_amount: u64) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load the private key from the environment
    let private_key = env::var("BSC_PRIVATE_KEY")?;
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?;

    // Connect to the BSC mainnet using the RPC URL
    let provider = Provider::<Http>::try_from(BSC_HTTP_URL)?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    let contract_address = PAB_BALANCE_LEDGER_CONTRACT.parse::<Address>()?;
    let contract = PabLedgerContract::new(contract_address, client);

    let updated_account_address = H160::from_str(&account).unwrap_or_default();
    let amount = U256::from(update_amount);
    match contract.update_balance(updated_account_address, amount).send().await{
        Ok(result) => println!("Transaction sent: {:?}", result),
        Err(e) => println!("Error updating balance: {:?}", e),
    }

    Ok(())
}

pub async fn proxy_contract_call_query_kol_staking(account: String) -> Result<u64, Error> {
    dotenv().ok();

    let private_key = env::var("BSC_PRIVATE_KEY")?;
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?;

    let provider = Provider::<Http>::try_from(BSC_HTTP_URL)?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    let contract_address = PAB_STAKING_CONTRACT.parse::<Address>()?;
    let contract = PabKOLStakingContract::new(contract_address, client);

    let account_address = H160::from_str(&account).unwrap_or_default();
    let mut amount: u64 = 0;
    match contract.stakes_of(account_address).call().await{
        Ok(result) => {
            println!("Transaction sent: {:?}", result);
            amount = result.as_u64();
        }
        Err(e) => println!("Error updating balance: {:?}", e),
    }

    Ok(amount)
}

async fn proxy_contract_call_kol_staking(owner: String, staked_amount: u64) -> Result<(), Error> {
    dotenv().ok();

    let private_key = env::var("BSC_PRIVATE_KEY")?;
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?;

    let provider = Provider::<Http>::try_from(BSC_HTTP_URL)?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    let contract_address = PAB_STAKING_CONTRACT.parse::<Address>()?;
    let contract = PabKOLStakingContract::new(contract_address, client);

    let account_address = H160::from_str(&owner).unwrap_or_default();
    let amount = U256::from(staked_amount);
    match contract.stake(account_address, amount).send().await{
        Ok(result) => println!("Transaction sent: {:?}", result),
        Err(e) => println!("Error updating balance: {:?}", e),
    }

    Ok(())
}

use anyhow::Error;
use ethers::prelude::*;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider, Ws};
use ethers::signers::{LocalWallet, Signer};
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;
use dotenv::dotenv;
use ethers::types::{Address, Filter, Log};
use std::env;

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
        [{"inputs":[{"internalType":"uint256","name":"_annualInterestRate","type":"uint256"},{"internalType":"uint256","name":"_minimumStakingPeriod","type":"uint256"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"user","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"timestamp","type":"uint256"}],"name":"Staked","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"user","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"reward","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"timestamp","type":"uint256"}],"name":"Withdrawn","type":"event"},{"inputs":[],"name":"annualInterestRate","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"components":[{"internalType":"uint256","name":"amount","type":"uint256"},{"internalType":"uint256","name":"since","type":"uint256"},{"internalType":"bool","name":"staked","type":"bool"}],"internalType":"struct SimpleStaking.Stake","name":"stakeInfo","type":"tuple"}],"name":"calculateReward","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"_user","type":"address"}],"name":"getInfoOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"},{"internalType":"uint256","name":"","type":"uint256"},{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"minimumStakingPeriod","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"stake","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"stakes","outputs":[{"internalType":"uint256","name":"amount","type":"uint256"},{"internalType":"uint256","name":"since","type":"uint256"},{"internalType":"bool","name":"staked","type":"bool"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"_user","type":"address"}],"name":"stakesOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"withdraw","outputs":[],"stateMutability":"nonpayable","type":"function"},{"stateMutability":"payable","type":"receive"}]    
    "#
);

const PAB_BALANCE_LEDGER_CONTRACT: &str = "0x5C98D79e6Ce7299a2Ea84B2898eAF064038AA1f3";
const PAB_STAKING_CONTRACT: &str = "0x4fc96644264Dba5630cdcc4b7696A3f7b20d4471";
const PAB_STAKING_PERIOD: u32 = 365;
const PAB_STAKING_APR: u32 = 1;

pub async fn proxy_contract_call_update_balance(account: String) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load the private key from the environment
    let private_key = env::var("BSC_PRIVATE_KEY")?;
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?;

    // Connect to the BSC mainnet using the RPC URL
    let provider = Provider::<Http>::try_from(env::var("BSC_RPC_URL")?)?;
    let client = SignerMiddleware::new(provider, wallet);

    let client = Arc::new(client);

    // The address of the deployed smart contract on BSC
    let contract_address = PAB_BALANCE_LEDGER_CONTRACT.parse::<Address>()?;

    // Instantiate the contract
    let contract = PabLedgerContract::new(contract_address, client);

    // Call the smart contract function
    let account_address = H160::from_str(&account).unwrap_or_default();
    let amount = U256::from(100);
    match contract.update_balance(account_address, amount).send().await{
        Ok(result) => println!("Transaction sent: {:?}", result),
        Err(e) => println!("Error updating balance: {:?}", e),
    }

    Ok(())
}

pub async fn proxy_contract_call_query_kol_staking(account: String) -> Result<u64, Error> {
    dotenv().ok();

    // Load the private key from the environment
    let private_key = env::var("BSC_PRIVATE_KEY")?;
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?;

    // Connect to the BSC mainnet using the RPC URL
    let provider = Provider::<Http>::try_from(env::var("BSC_RPC_URL")?)?;
    let client = SignerMiddleware::new(provider, wallet);

    let client = Arc::new(client);

    // The address of the deployed smart contract on BSC
    let contract_address = PAB_STAKING_CONTRACT.parse::<Address>()?;

    // Instantiate the contract
    let contract = PabKOLStakingContract::new(contract_address, client);

    // Call the smart contract function
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

async fn monitor_pab_transfer_event() -> Result<(), Error> {
    // Define the BSC WebSocket URL. You can use a provider like Infura, Alchemy, or a local BSC node.
    let ws_url = "wss://bsc-ws-node.nariox.org:443";

    // Set up the WebSocket provider
    let provider = Provider::<Ws>::connect(ws_url).await?;

    // BEP-20 Transfer event signature (similar to ERC-20)
    let transfer_event_signature: Address = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".parse()?;

    // You can optionally filter by the address of the token contract if you're monitoring a specific token
    let pab_token_contract_address: Address = "0xD6311f9A6bd3a802263F4cd92e2729bC2C31Ed23".parse()?;

    // Set up the filter to monitor Transfer events
    let filter = Filter::new()
        .topic0(transfer_event_signature)  // Monitor by the Transfer event signature
        .topic1(pab_token_contract_address);

    // Subscribe to the filter on the blockchain
    let mut stream = provider.subscribe_logs(&filter).await?;

    // Handle incoming Transfer events
    while let Some(log) = stream.next().await {
        handle_event(log);
    }

    Ok(())
}

// Function to process incoming log event data
fn handle_event(log: Log) {
    // Decode the Transfer event
    println!("New Transfer event: {:?}", log);
}

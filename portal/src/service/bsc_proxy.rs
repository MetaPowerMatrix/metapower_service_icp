use ethers::prelude::*;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;
use dotenv::dotenv;
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
        [{"inputs":[{"internalType":"uint256","name":"_annualInterestRate","type":"uint256"},{"internalType":"uint256","name":"_minimumStakingPeriod","type":"uint256"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"user","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"timestamp","type":"uint256"}],"name":"Staked","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"user","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"reward","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"timestamp","type":"uint256"}],"name":"Withdrawn","type":"event"},{"inputs":[],"name":"annualInterestRate","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"components":[{"internalType":"uint256","name":"amount","type":"uint256"},{"internalType":"uint256","name":"since","type":"uint256"},{"internalType":"bool","name":"staked","type":"bool"}],"internalType":"struct SimpleStaking.Stake","name":"stakeInfo","type":"tuple"}],"name":"calculateReward","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"minimumStakingPeriod","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"stake","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"stakes","outputs":[{"internalType":"uint256","name":"amount","type":"uint256"},{"internalType":"uint256","name":"since","type":"uint256"},{"internalType":"bool","name":"staked","type":"bool"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"withdraw","outputs":[],"stateMutability":"nonpayable","type":"function"},{"stateMutability":"payable","type":"receive"}]
    "#
);

const PAB_BALANCE_LEDGER_CONTRACT: &str = "0x5C98D79e6Ce7299a2Ea84B2898eAF064038AA1f3";
const PAB_STAKING_CONTRACT: &str = "0xb83c19B725A7f684f6050857ef2581754E105551";
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

pub async fn proxy_contract_call_get_balance(account: String) -> Result<u64, Box<dyn std::error::Error>> {
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
    let mut amount: u64 = 0;
    match contract.get_balance(account_address).call().await{
        Ok(result) => {
            println!("Transaction sent: {:?}", result);
            amount = result.as_u64();
        }
        Err(e) => println!("Error updating balance: {:?}", e),
    }

    Ok(amount)
}

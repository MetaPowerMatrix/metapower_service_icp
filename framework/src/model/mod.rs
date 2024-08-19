use serde::{Deserialize, Serialize};

pub mod memcache;

#[derive(Default, Debug)]
pub struct BatteryConnection {
    pub id: String,
}

#[derive(Default, Debug)]
pub struct BatteryWallet {
    pub address: String,
    pub chain: String,
}

#[derive(Default, Debug)]
pub struct BatteryStatus {
    pub ip: String,
}

#[derive(Default, Debug)]
pub struct Battery {
    pub id: String,
    pub wallets: Vec<BatteryWallet>,
    pub connections: Vec<BatteryConnection>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct BatteryRole {
    pub name: String,
}

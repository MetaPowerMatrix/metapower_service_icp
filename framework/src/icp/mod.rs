use std::path::PathBuf;

use candid::{CandidType, Encode, Principal};
use ic_agent::{identity::BasicIdentity, Agent, AgentError, Identity};
use ring::signature::Ed25519KeyPair;
use serde::Deserialize;

const DEFAULT_IC_GATEWAY: &str = "https://ic0.app/";
pub const ENDPOINT_URL: &str = "http://localhost:8000/";
pub const PEM_FILE: &str = "identity.pem";
pub const AGENT_SMITH_CANISTER: &str = "eegr3-kiaaa-aaaai-acuaa-cai";
pub const NAIS_MATRIX_CANISTER: &str = "fvcqf-aqaaa-aaaak-ak5oa-cai";
pub const AGENT_BATTERY_CANISTER: &str = "edhxp-hqaaa-aaaai-acuaq-cai";

#[derive(Deserialize, CandidType)]
pub struct SnIdPaire {
    pub id: String,
    pub sn: String,
}

#[derive(Deserialize, CandidType)]
pub struct SnRequest {
    pub id: Vec<String>,
}

#[derive(Deserialize, CandidType, Default)]
pub struct SnResponse {
    pub pato_sn_id: Vec<SnIdPaire>,
}

#[derive(Deserialize, CandidType)]
pub struct KolRegistrationRequest {
    pub id: String,
    pub key: String,
}

#[derive(Deserialize, CandidType)]
pub struct FollowKolRequest {
    pub id: String,
    pub follower: String,
    pub key: String,
}

#[derive(Deserialize, CandidType)]
pub struct NamePros {
    pub id: String,
    pub name: String,
    pub pros: Vec<String>,
}

#[derive(Deserialize, CandidType, Default)]
pub struct NameResponse {
    pub name_pros: Vec<NamePros>,
}

#[derive(Deserialize, CandidType)]
pub struct NameRequest {
    pub id: Vec<String>,
}


pub async fn init_icp_agent() -> Result<Agent, AgentError>{
    let agent: Agent = Agent::builder()
        .with_url(DEFAULT_IC_GATEWAY)
        .with_identity(create_identity(None))
        .build()?;

    agent.fetch_root_key().await?;

    Ok(agent)
}
// static AGENT: Mutex<Agent> = Mutex::new(init_icp_agent().await.unwrap_or_default());

fn create_identity(maybe_pem: Option<PathBuf>) -> impl Identity {
    if let Some(pem_path) = maybe_pem {
        BasicIdentity::from_pem_file(pem_path).expect("Could not read the key pair.")
    } else {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .expect("Could not generate a key pair.")
            .as_ref()
            .to_vec();

        BasicIdentity::from_key_pair(
            Ed25519KeyPair::from_pkcs8(&pkcs8_bytes).expect("Could not generate the key pair."),
        )
    }
}

pub async fn call_update_method<T: CandidType>(canister_called: &str, method_name: &str, params: T) -> Result<Vec<u8>, AgentError>
{
    let agent = init_icp_agent().await?;
    let effective_canister_id = Principal::from_text(canister_called).unwrap();

    agent.update(&effective_canister_id, method_name)
      .with_effective_canister_id(effective_canister_id)
      .with_arg(Encode!(&params)?)
      .await
}

pub async fn call_query_method<T: CandidType>(canister_called: &str, method_name: &str, params: T) -> Result<Vec<u8>, AgentError>
{
    let agent = init_icp_agent().await?;
    let effective_canister_id = Principal::from_text(canister_called).unwrap();

    agent.query(&effective_canister_id, method_name)
      .with_effective_canister_id(effective_canister_id)
      .with_arg(Encode!(&params)?)
      .await
}

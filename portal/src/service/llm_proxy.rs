use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

use anyhow::anyhow;
use anyhow::Error;
use candid::CandidType;
use candid::Decode;
use candid::Encode;
use candid::Principal;
use metapower_framework::dao::crawler::download_image;
use metapower_framework::ensure_directory_exists;
use metapower_framework::icp::call_update_method;
use metapower_framework::icp::init_icp_agent;
use metapower_framework::icp::AGENT_BATTERY_CANISTER;
use metapower_framework::icp::AGENT_SMITH_CANISTER;
use metapower_framework::icp::NAIS_MATRIX_CANISTER;
use metapower_framework::XFILES_LOCAL_DIR;
use metapower_framework::XFILES_SERVER;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use crate::service::PatoInfoResponse;

pub const MAX_SAVE_BYTES: usize = 1024*1024*5;

#[derive(Clone, Serialize)]
struct ImageGenRequest {
    pub prompt: String,
}

#[derive(Clone, Serialize)]
struct FileGenRequest {
    pub content: String,
}

const LLM_HTTP_HOST: &str = "llm.metapowermatrix.ai";
const LLM_REQUEST_PROTOCOL: &str = "https://";

#[derive(Deserialize, CandidType, Serialize, Debug)]
pub struct CharacterGenRequest {
    pub tags: Vec<String>,
    pub name: String,
    pub gender: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageGenResponse {
    pub cover: String,
    pub avatar: String,
    pub character: String,
}

async fn get_pato_name(id: String) -> Result<String, Error>{
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_info", id).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), PatoInfoResponse).unwrap_or_default();

            Ok(response.name)
        }
        Err(e) => Err(anyhow!("request_pato_info error: {}", e)),
    }
}
async fn check_session_file(id: String, session_key: String, file_name: String) -> Result<bool, Error>{
    let agent = init_icp_agent().await?;
    let effective_canister_id = Principal::from_text(NAIS_MATRIX_CANISTER).unwrap();
    
    match agent.update(&effective_canister_id, "check_session_assets")
        .with_effective_canister_id(effective_canister_id)
        .with_arg(Encode!(&id, &session_key, &file_name)?)
        .await{
            Ok(result) => {
                Ok(Decode!(result.as_slice(), bool).unwrap_or_default())
            }
            Err(e) => {
                println!("check session file error: {}", e);
                Ok(false)
            }
    }
}
pub async fn read_session_file(id: String, session_key: String, file_name: String) -> Result<Vec<u8>, Error>{
    let agent = init_icp_agent().await?;
    let effective_canister_id = Principal::from_text(NAIS_MATRIX_CANISTER).unwrap();
    
    match agent.update(&effective_canister_id, "query_session_assets")
        .with_effective_canister_id(effective_canister_id)
        .with_arg(Encode!(&id, &session_key, &file_name)?)
        .await{
            Ok(result) => {
                Ok(Decode!(result.as_slice(), Vec<u8>).unwrap_or_default())
            }
            Err(e) => {
                Err(e.into())
            }
        }
}
async fn save_session_file(id: String, session_key: String, file_name: String, data: Vec<u8>) -> Result<(), Error>{
    let agent = init_icp_agent().await?;
    let effective_canister_id = Principal::from_text(NAIS_MATRIX_CANISTER).unwrap();
    
    match agent.update(&effective_canister_id, "upload_session_assets")
        .with_effective_canister_id(effective_canister_id)
        .with_arg(Encode!(&id, &session_key, &file_name, &data)?)
        .await{
            Ok(_) => Ok(()),
            Err(e) => {
                Err(e.into())
            }
        }
}

pub async fn upload_knowledge_save_in_canister(session_key: String, id: String, file_name: String, content: Vec<u8>) -> Result<String, Error> {
    let _ = ensure_directory_exists(&format!("{}/ai/{}", XFILES_LOCAL_DIR, id));
    let url_embedding = format!("{}{}/api/gen/embedding", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
    let url_summary = format!("{}{}/api/gen/summary", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);

    let local_name = file_name;
    let mut resp: String = "文件已经上传".to_string();
    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await?{
        let embedding_request = FileGenRequest{ content: String::from_utf8(content.clone()).unwrap_or_default() };

        save_session_file(id.clone(), session_key.clone(), local_name.clone(), content).await?;

        let client = reqwest::Client::new();
        let response = client
            .post(&url_embedding)
            .json(&json!(embedding_request))
            .send()
            .await?;

        let saved_bytes = response.bytes().await?;
        // let embedding: Vec<f64> = serde_json::from_slice(&saved_bytes)?;
        // println!("embedding: {:?}", embedding);
        let embedding_file = local_name.clone() + ".embed";
        save_session_file(id.clone(), session_key.clone(), embedding_file, saved_bytes.to_vec()).await?;

        let response = client
            .post(&url_summary)
            .json(&json!(embedding_request))
            .send()
            .await?;

        let summary: String = response.json().await?;
        println!("summary: {}", summary);
        let summary_file = local_name.clone() + ".sum";
        resp = summary.clone();
        save_session_file(id.clone(), session_key.clone(), summary_file, summary.as_bytes().to_vec()).await?;
    }

    Ok(resp)
}
pub async fn upload_image_save_in_canister(session_key: String, id: String, content: Vec<u8>) -> Result<String, Error> {
    let _ = ensure_directory_exists(&format!("{}/user/uploaded/{}", XFILES_LOCAL_DIR, id));
    let url = format!("{}{}/api/gen/image/description", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);

    let local_name = "upload.png".to_string();
    let resp = format!("{}/user/uploaded/{}/{}", XFILES_SERVER, id, local_name);
    let mut desc = String::default();

    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await?{
        save_session_file(id.clone(), session_key.clone(), local_name.clone(), content.clone()).await?;

        let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, local_name);
        match OpenOptions::new().write(true).create(true).truncate(true).open(&saved_local_file){
            Ok(mut file) => {
                file.write_all(&content)?;
            }
            Err(e) => {
                println!("write local file error: {}", e);
            }
        }

        let embedding_request = FileGenRequest{ content: resp.clone() };
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&json!(embedding_request))
            .send()
            .await?;

        desc = response.json().await?;
        println!("image description: {:?}", desc);
        let desc_file = local_name.clone() + ".desc";
        save_session_file(id.clone(), session_key.clone(), desc_file, desc.as_bytes().to_vec()).await?;
    }

    Ok(desc)
}
pub async fn set_pato_info(id: String, data: String, method: &str) -> Result<(), Error> {
    match call_update_method(AGENT_BATTERY_CANISTER, method, 
        (id, data)).await {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(e.into())
        }
    }
}
pub async fn submit_tags_with_proxy(tags: Vec<String>, session_key: String, id: String) -> Result<(), Error> {
    let _ = ensure_directory_exists(&format!("{}/ai/{}", XFILES_LOCAL_DIR, id));
    let mut character = String::default();

    set_pato_info(id.clone(), tags.join(","), "set_tags_of").await?;

    let local_name = "character.txt".to_string();

    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await?{
        let url = format!("{}{}/api/gen/character", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
        let tag_request = CharacterGenRequest {
            tags,
            name: get_pato_name(id.clone()).await.unwrap_or_default(),
            gender: "".to_string(),
        };
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&json!(tag_request))
            .send()
            .await?;
        character = response.text().await?;

        save_session_file(id.clone(), session_key.clone(), local_name.clone(), character.as_bytes().to_vec()).await?;
        set_pato_info(id.clone(), character.clone(), "set_character_of").await?;

        let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, local_name);
        match OpenOptions::new().write(true).create(true).truncate(true).open(&saved_local_file){
            Ok(mut file) => {
                file.write_all(character.as_bytes())?;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }                
        }
    }

    let url = format!("{}{}/api/gen/avatar", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
    let avatar_prompt = format!("Design an avatar that represents a fictional character or persona for storytelling or role-playing purposes. Provide details about the character's appearance, personality traits, and backstory to create a visually compelling and immersive avatar: {}", character);
    let avatar_request = ImageGenRequest {
        prompt: avatar_prompt,
    };
    let local_name = "avatar.png".to_string();

    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await?{
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&json!(avatar_request))
            .send()
            .await?;
        let file_url = response.text().await?;

        let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, local_name);
        download_image(&file_url, &saved_local_file).await?;

        let xfiles_path = format!("{}/ai/{}/{}", XFILES_SERVER, id, local_name);
        set_pato_info(id.clone(), xfiles_path, "set_avatar_of").await?;

        match OpenOptions::new().read(true).open(&saved_local_file){
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                save_session_file(id.clone(), session_key.clone(), local_name, content.as_bytes().to_vec()).await?;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }
    }

    let url = format!("{}{}/api/gen/cover", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
    let avatar_request = ImageGenRequest {
        prompt: character.clone(),
    };
    let local_name = "cover.png".to_string();

    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await?{
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&json!(avatar_request))
            .send()
            .await?;
        let file_url = response.text().await?;

        let xfiles_path = format!("{}/ai/{}/{}", XFILES_SERVER, id, local_name);
        set_pato_info(id.clone(), xfiles_path, "set_cover_of").await?;

        let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, local_name);
        download_image(&file_url, &saved_local_file).await?;
        match OpenOptions::new().read(true).open(&saved_local_file){
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                save_session_file(id, session_key, local_name, content.as_bytes().to_vec()).await?;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }
    }

    Ok(())
}

pub async fn gen_image_save_in_canister(prompt: String, session_key: String, id: String) -> Result<String, Error> {
    let mut resp = String::default();

    let url = format!("{}{}/api/gen/image", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
    let avatar_request = ImageGenRequest {
        prompt,
    };
    let local_name = "image.png".to_string();
    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await?{
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&json!(avatar_request))
            .send()
            .await?;
        let file_url = response.text().await?;
        download_image(&file_url, &local_name).await?;

        resp = format!("{}/ai/{}/{}/{}", XFILES_SERVER, id, session_key, local_name);

        let saved_local_file = format!("{}/ai/{}/{}/{}", XFILES_LOCAL_DIR, id, session_key, local_name);
        match OpenOptions::new().read(true).open(&saved_local_file){
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                save_session_file(id.clone(), session_key.clone(), local_name, content.as_bytes().to_vec()).await?;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }
    }

    Ok(resp)
}
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

use anyhow::anyhow;
use anyhow::Error;
use candid::CandidType;
use candid::Decode;
use metapower_framework::dao::crawler::download_image;
use metapower_framework::ensure_directory_exists;
use metapower_framework::icp::call_update_method;
use metapower_framework::icp::AGENT_SMITH_CANISTER;
use metapower_framework::icp::NAIS_MATRIX_CANISTER;
use metapower_framework::SubmitTagsResponse;
use metapower_framework::XFILES_LOCAL_DIR;
use metapower_framework::XFILES_SERVER;
use serde::{Deserialize, Serialize};
use serde_json::json;
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
async fn check_session_file(id: String, session_key: String, file_name: String) -> bool{
    match call_update_method(NAIS_MATRIX_CANISTER, "check_session_assets",
     (id, session_key, file_name)).await {
        Ok(result) => {
            Decode!(result.as_slice(), bool).unwrap_or_default()
        }
        Err(_) => {
            false
        }
    }
}
pub async fn read_session_file(id: String, session_key: String, file_name: String) -> Vec<u8>{
    match call_update_method(NAIS_MATRIX_CANISTER, "query_session_assets",
     (id, session_key, file_name)).await {
        Ok(result) => {
            Decode!(result.as_slice(), Vec<u8>).unwrap_or_default()
        }
        Err(_) => {
            vec![]
        }
    }
}
async fn save_session_file(id: String, session_key: String, file_name: String, data: Vec<u8>){
    match call_update_method(NAIS_MATRIX_CANISTER, "upload_session_assets", 
    (id, session_key, file_name, data)).await {
        Ok(_) => {}
        Err(e) => {
            println!("save_session_file error: {}", e);
        }
    }
}

pub async fn upload_knowledge_save_in_canister(session_key: String, id: String, file_name: String, content: Vec<u8>) -> Result<String, Error> {
    let mut resp = String::default();
    let _ = ensure_directory_exists(&format!("{}/ai/{}", XFILES_LOCAL_DIR, id));
    let url_embedding = format!("{}{}/api/gen/embedding", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
    let url_summary = format!("{}{}/api/gen/summary", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);

    let local_name = file_name;
    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await{
        resp = format!("{}/ai/{}/{}", XFILES_SERVER, id, local_name);

        let embedding_request = FileGenRequest{ content: String::from_utf8(content.clone()).unwrap_or_default() };

        let client = reqwest::Client::new();
        let response = client
            .post(&url_embedding)
            .json(&json!(embedding_request))
            .send()
            .await?;

        let embedding = response.text().await?;
        println!("embedding: {}", embedding);

        let response = client
            .post(&url_summary)
            .json(&json!(embedding_request))
            .send()
            .await?;

        let summary = response.text().await?;
        println!("summary: {}", summary);

        let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, local_name);
        match OpenOptions::new().write(true).create(true).truncate(true).open(&saved_local_file){
            Ok(mut file) => {
                file.write_all(&content)?;
                save_session_file(id.clone(), session_key.clone(), local_name.clone(), content).await;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }    
        let saved_local_embedding = format!("{}/ai/{}/{}.embedding", XFILES_LOCAL_DIR, id, local_name);
        match OpenOptions::new().write(true).create(true).truncate(true).open(&saved_local_embedding){
            Ok(mut file) => {
                file.write_all(embedding.as_bytes())?;
                save_session_file(id.clone(), session_key.clone(), local_name.clone(), embedding.as_bytes().to_vec()).await;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }    
        let saved_local_summary = format!("{}/ai/{}/{}.summary", XFILES_LOCAL_DIR, id, local_name);
        match OpenOptions::new().write(true).create(true).truncate(true).open(&saved_local_summary){
            Ok(mut file) => {
                file.write_all(summary.as_bytes())?;
                save_session_file(id.clone(), session_key.clone(), local_name.clone(), summary.as_bytes().to_vec()).await;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }    
    }

    Ok(resp)
}
pub async fn upload_image_save_in_canister(session_key: String, id: String, content: Vec<u8>) -> Result<String, Error> {
    let mut resp = String::default();

    let local_name = "upload.png".to_string();
    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await{
        resp = format!("{}/ai/{}/{}", XFILES_SERVER, id, local_name);

        let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, local_name);
        let _ = ensure_directory_exists(&format!("{}/ai/{}", XFILES_LOCAL_DIR, id));
        match OpenOptions::new().write(true).create(true).truncate(true).open(&saved_local_file){
            Ok(mut file) => {
                file.write_all(&content)?;
                save_session_file(id.clone(), session_key.clone(), local_name.clone(), content).await;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }
    }

    Ok(resp)
}
pub async fn submit_tags_with_proxy(tags: Vec<String>, session_key: String, id: String) -> Result<SubmitTagsResponse, Error> {
    let mut resp = SubmitTagsResponse::default();

    let url = format!("{}{}/api/gen/character", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
    let tag_request = CharacterGenRequest {
        tags,
        name: get_pato_name(id.clone()).await.unwrap_or_default(),
        gender: "".to_string(),
    };
    let _ = ensure_directory_exists(&format!("{}/ai/{}", XFILES_LOCAL_DIR, id));

    let local_name = "character.txt".to_string();
    resp.character = format!("{}/ai/{}/{}", XFILES_SERVER, id, local_name);

    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await{
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&json!(tag_request))
            .send()
            .await?;
        let character = response.text().await?;

        let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, local_name);
        match OpenOptions::new().write(true).create(true).truncate(true).open(&saved_local_file){
            Ok(mut file) => {
                file.write_all(character.as_bytes())?;
                save_session_file(id.clone(), session_key.clone(), local_name.clone(), character.as_bytes().to_vec()).await;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }                
        }
    }

    let url = format!("{}{}/api/gen/avatar", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
    let avatar_prompt = format!("Design an avatar that represents a fictional character or persona for storytelling or role-playing purposes. Provide details about the character's appearance, personality traits, and backstory to create a visually compelling and immersive avatar: {}", resp.character);
    let avatar_request = ImageGenRequest {
        prompt: avatar_prompt,
    };
    let local_name = "avatar.png".to_string();
    resp.avatar = format!("{}/ai/{}/{}", XFILES_SERVER, id, local_name);

    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await{
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&json!(avatar_request))
            .send()
            .await?;
        let file_url = response.text().await?;
        download_image(&file_url, &local_name).await?;

        let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, local_name);
        match OpenOptions::new().read(true).open(&saved_local_file){
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                save_session_file(id.clone(), session_key.clone(), local_name, content.as_bytes().to_vec()).await;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }
    }

    let url = format!("{}{}/api/gen/cover", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
    let avatar_request = ImageGenRequest {
        prompt: resp.character.clone(),
    };
    let local_name = "cover.png".to_string();
    resp.cover = format!("{}/ai/{}/{}", XFILES_SERVER, id, local_name);

    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await{
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&json!(avatar_request))
            .send()
            .await?;
        let file_url = response.text().await?;
        download_image(&file_url, &local_name).await?;

        let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, local_name);
        match OpenOptions::new().read(true).open(&saved_local_file){
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                save_session_file(id, session_key, local_name, content.as_bytes().to_vec()).await;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }
    }

    Ok(resp)
}

pub async fn gen_image_save_in_canister(prompt: String, session_key: String, id: String) -> Result<String, Error> {
    let mut resp = String::default();

    let url = format!("{}{}/api/gen/image", LLM_REQUEST_PROTOCOL, LLM_HTTP_HOST);
    let avatar_request = ImageGenRequest {
        prompt,
    };
    let local_name = "image.png".to_string();
    if !check_session_file(id.clone(), session_key.clone(), local_name.clone()).await{
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
                save_session_file(id.clone(), session_key.clone(), local_name, content.as_bytes().to_vec()).await;
            }
            Err(e) => {
                println!("open file error: {}", e);
            }
        }
    }

    Ok(resp)
}
use anyhow::{anyhow, Error};
use candid::{CandidType, Decode};
use metapower_framework::icp::{
    call_update_method, AGENT_BATTERY_CANISTER, AGENT_SMITH_CANISTER,
    NAIS_MATRIX_CANISTER,
};
use metapower_framework::{log, PatoInfoResp, SubmitTagsResponse};
use metapower_framework::{
    PatoInfo, XFILES_SERVER,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::str::from_utf8;
use std::time::SystemTime;
use std::io::Write;
use crate::service::{
    CreateResonse, HotTopicResponse, KolRelations, NameResponse, PatoInfoResponse, SharedKnowledgesResponse, SimpleResponse, TokenResponse
};
use crate::KolInfo;

use super::llm_proxy::{gen_image_save_in_canister, read_session_file, submit_tags_with_proxy, upload_knowledge_save_in_canister};
use super::{
    BecomeKolRequest, JoinKolRoomRequest, SubmitTagsRequest,
};

#[derive(Deserialize, Debug, Default, Serialize)]
struct PortalHotAi {
    id: String,
    name: String,
    talks: i32,
    pros: String,
}
#[derive(Deserialize, Debug, Default, Serialize)]
struct PortalPatoOfPro {
    id: String,
    name: String,
    subjects: Vec<String>,
}
#[derive(Deserialize, Debug, Default, Serialize)]
struct PortalKnowledge {
    sig: String,
    title: String,
    owner: String,
    summary: String,
}

#[derive(Deserialize, Debug, Default, Serialize, CandidType)]
struct BatterCallParams{
    id: String,
    token: String,
    sn: i64,
    method_name: String,
    arg: String,
}
fn prepare_battery_call_args<T: Serialize>(
    id: String,
    token: String,
    sn: i64,
    method_name: String,
    arg: T,
) -> String {
    serde_json::to_string(&BatterCallParams{
        id,
        token,
        sn,
        method_name,
        arg: serde_json::to_string(&arg).unwrap_or_default(),
    }).unwrap_or_default()
}

pub async fn town_login(id: String) -> Result<(), Error> {
    match call_update_method(NAIS_MATRIX_CANISTER, "request_pato_login", id).await {
        Ok(_) => {
            log!("login success");
        }
        Err(e) => {
            log!("connect matrix error: {}", e);
        }
    }

    Ok(())
}
pub async fn town_hots() -> String {
    match call_update_method(NAIS_MATRIX_CANISTER, "request_hot_ai", ()).await {
        Ok(response) => {
            // println!("town_hots response: {:?}", response);
            let result = Decode!(response.as_slice(), Vec<PatoInfoResp>).unwrap_or_default();
            let resp = result
                .iter()
                .map(|h| PortalHotAi {
                    id: h.id.clone(),
                    name: h.name.clone(),
                    talks: 0,
                    pros: "".to_string(),
                })
                .collect::<Vec<PortalHotAi>>();

            return serde_json::to_string(&resp).unwrap_or_default();
        }
        Err(e) => {
            log!("connect matrix error: {}", e);
        }
    }

    String::default()
}
pub async fn town_hot_topics() -> String {
    match call_update_method(NAIS_MATRIX_CANISTER, "request_hot_topics", ()).await {
        Ok(response) => {
            let result = Decode!(response.as_slice(), HotTopicResponse).unwrap_or_default();
            let topics = result.topics.clone();
            return serde_json::to_string(&topics).unwrap_or_default();
        }
        Err(e) => {
            log!("call matrix canister error: {}", e);
        }
    }

    String::default()
}
pub async fn shared_knowledges() -> String {
    match call_update_method(NAIS_MATRIX_CANISTER, "request_shared_knowledges", ()).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), SharedKnowledgesResponse).unwrap_or_default();
            let hots = response.books;
            let resp = hots
                .iter()
                .map(|h| PortalKnowledge {
                    sig: h.sig.clone(),
                    title: h.title.clone(),
                    owner: h.owner.clone(),
                    summary: h.summary.clone(),
                })
                .collect::<Vec<PortalKnowledge>>();

            return serde_json::to_string(&resp).unwrap_or_default();
        }
        Err(e) => {
            log!("connect matrix error: {}", e);
        }
    }

    String::default()
}

pub async fn town_register(name: String) -> Result<String, Error> {
    match call_update_method(NAIS_MATRIX_CANISTER, "request_create_pato", name).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), CreateResonse).unwrap_or_default();
            println!("request_create_pato response: {:?}", response);
            return Ok(response.id);
        }
        Err(e) => {
            log!("connect matrix error: {}", e);
        }
    }

    Ok(String::default())
}

pub async fn get_pato_info(id: String) -> Result<PatoInfo, Error> {
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_info", id).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), PatoInfoResponse).unwrap_or_default();

            let pato_info = PatoInfo {
                id: response.id.clone(),
                name: response.name.clone(),
                sn: response.sn,
                registered_datetime: response.registered_datetime.clone(),
                balance: response.balance,
                tags: response.tags.clone(),
                avatar: response.avatar.clone(),
                cover: response.cover.clone(),
                matrix_datetime: response.registered_datetime,
            };
            Ok(pato_info)
        }
        Err(e) => Err(anyhow!("request_pato_info error: {}", e)),
    }
}
pub async fn retrieve_pato_by_name(name: String) -> Result<String, Error> {
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_by_name", name).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), NameResponse).unwrap_or_default();

            let mut patos: Vec<PortalPatoOfPro> = vec![];
            for pato in response.name_pros.iter() {
                let i = PortalPatoOfPro {
                    id: pato.id.clone(),
                    subjects: pato.pros.clone(),
                    name: pato.name.clone(),
                };
                patos.push(i);
            }
            Ok(serde_json::to_string(&patos).unwrap_or_default())
        }
        Err(e) => Err(anyhow!("request_pato_info error: {}", e)),
    }
}
pub async fn get_name_by_id(ids: Vec<String>) -> Result<String, Error> {
    let req = super::NameRequest { id: ids };
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_by_ids", req).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), NameResponse).unwrap_or_default();

            let mut patos: Vec<PortalPatoOfPro> = vec![];
            for pato in response.name_pros.iter() {
                let i = PortalPatoOfPro {
                    id: pato.id.clone(),
                    subjects: pato.pros.clone(),
                    name: pato.name.clone(),
                };
                patos.push(i);
            }
            Ok(serde_json::to_string(&patos).unwrap_or_default())
        }
        Err(e) => Err(anyhow!("request_pato_info error: {}", e)),
    }
}

pub async fn archive_pato_session(id: String, session: String, content: String) -> Result<String, Error> {
    let local_name = "chat_messages.txt".to_string();

    upload_knowledge_save_in_canister(session, id, local_name, content.as_bytes().to_vec()).await
}

pub async fn request_generate_image(
    id: String,
    session: String,
    prompt: String,
) -> Result<String, Error> {
    let answer = gen_image_save_in_canister(prompt, session, id).await?;

    Ok(answer)
}
pub async fn request_submit_tags_with_proxy(
    id: String,
    session: String,
    tags: Vec<String>
) -> Result<(), Error> {
    submit_tags_with_proxy(tags, session, id).await?;

    Ok(())
}
pub async fn get_pato_chat_messages(
    id: String,
    session: String,
) -> Result<String, Error> {
    let local_name = "chat_messages.txt".to_string();
    let query_result = read_session_file(id, session, local_name).await?;

    Ok(from_utf8(&query_result).unwrap_or_default().to_string())
}
pub async fn get_topic_chat_history(
    id: String,
    session: String,
) -> Result<String, Error> {
    let local_name = "chat_messages.txt".to_string();
    let query_result = read_session_file(id, session, local_name).await?;

    Ok(from_utf8(&query_result).unwrap_or_default().to_string())
}
pub async fn get_predefined_tags() -> Result<String, Error> {
    match call_update_method(AGENT_SMITH_CANISTER, "request_predefined_tags", ()).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), String).unwrap_or_default();
            Ok(response)
        }
        Err(e) => {
            Err(anyhow!("get_predefined_tags error: {}", e))
        }
    }
}
pub async fn submit_tags(id: String, session: String, tags: Vec<String>) -> Result<String, Error> {
    let request = SubmitTagsRequest { id: id.clone(), tags, session  };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_submit_tags".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", 
        req).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), SubmitTagsResponse).unwrap_or_default();
            Ok(response.avatar)
        }
        Err(e) => {
            Err(e.into())
        }
    }
}

pub async fn refresh_pato_auth_token(id: String) -> Result<String, Error> {
    let mut token = "".to_string();

    match call_update_method(AGENT_SMITH_CANISTER, "refresh_battery_auth", id).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), String).unwrap_or_default();
            token = response.clone();
        }
        Err(e) => {
            log!("request_pato_auth_token error: {}", e);
        }
    }

    Ok(token)
}
pub async fn query_pato_kol_token(id: String) -> Result<TokenResponse, Error> {
    match call_update_method(AGENT_SMITH_CANISTER, "query_pato_kol_token", id).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), TokenResponse).unwrap_or_default();
            Ok(response)
        }
        Err(e) => {
            Err(e.into())
        }
    }
}
pub async fn query_pato_by_kol_token(token: String) -> Result<TokenResponse, Error> {
    match call_update_method(AGENT_SMITH_CANISTER, "query_pato_by_kol_token", token).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), TokenResponse).unwrap_or_default();
            Ok(response)
        }
        Err(e) => {
            Err(e.into())
        }
    }
}
pub async fn query_pato_auth_token(token: String) -> Result<(String, String), Error> {
    let mut id = "".to_string();
    let mut name = "".to_string();

    match call_update_method(AGENT_SMITH_CANISTER, "query_pato_by_auth_token", token).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), TokenResponse).unwrap_or_default();
            id = response.id.clone();
            name = response.name.clone();
        }
        Err(e) => {
            log!("query_pato_auth_token error: {}", e);
        }
    }

    Ok((id, name))
}
pub async fn query_kol_rooms() -> Result<String, Error> {
    let mut kols: Vec<KolInfo> = vec![];
    match call_update_method(AGENT_SMITH_CANISTER, "request_kol_list", ()).await {
        Ok(result) => {
            let resp = Decode!(result.as_slice(), Vec<KolRelations>).unwrap_or_default();

            for response in resp.iter() {
                let avatar_link = format!("{}/ai/{}/avatar.png", XFILES_SERVER, response.id);
                let info = KolInfo {
                    id: response.id.clone(),
                    name: response.name.clone(),
                    followers: response.follower.clone(),
                    avatar: avatar_link,
                };
                kols.push(info);
            }
        }
        Err(e) => {
            return Err(e.into());
        }
    }

    Ok(serde_json::to_string(&kols).unwrap_or_default())
}
pub async fn become_kol(id: String, from: String) -> Result<String, Error> {
    let request = BecomeKolRequest { id: id.clone(), from };

    let req = prepare_battery_call_args(id, "".to_string(), -1, "become_kol".to_string(), request);
    println!("become_kol req: {}", req);
    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), SimpleResponse).unwrap_or_default();
            Ok(response.message)
        }
        Err(e) => {
            Err(e.into())
        }
    }
}
pub async fn follow_kol(kol: String, follower: String, from: String) -> Result<(), Error> {
    let request = JoinKolRoomRequest {
        key: String::default(),
        kol: kol.clone(),
        follower,
        from,
    };

    let req = prepare_battery_call_args(
        kol,
        "".to_string(),
        -1,
        "request_join_kol_room".to_string(),
        request,
    );

    call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await?;

    Ok(())
}
pub async fn query_document_embeddings(
    id: String,
    sig: String,
    file_name: String,
) -> Result<String, Error> {
    let query_result = read_session_file(id, sig, file_name).await?;

    Ok(from_utf8(&query_result).unwrap_or_default().to_string())
}
pub async fn query_document_summary(id: String, sig: String, file_name: String) -> Result<String, Error> {
    let query_result = read_session_file(id, sig, file_name).await?;

    Ok(from_utf8(&query_result).unwrap_or_default().to_string())
}

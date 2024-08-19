use std::io::Read;
use std::path::Path;
use std::{fs::OpenOptions, io::Write};
use std::time::SystemTime;
use std::env;
use anyhow::{anyhow, Error};
use candid::Decode;
use metapower_framework::icp::{call_update_method, AGENT_SMITH_CANISTER, NAIS_MATRIX_CANISTER};
use metapower_framework::service::metapowermatrix_battery_mod::battery_grpc::SubmitTagsRequest;
use metapower_framework::{AI_MATRIX_DIR, XFILES_LOCAL_DIR, XFILES_SERVER};
use metapower_framework::{
    get_now_date_str, log, service::metapowermatrix_battery_mod::battery_grpc::{
            meta_power_matrix_battery_svc_client::MetaPowerMatrixBatterySvcClient, ArchiveMessageRequest, BecomeKolRequest, CallRequest, ContinueRequest, DocumentSummaryRequest, EditeReqeust, EmptyRequest, GameAnswerRequest, GetMessageRequest, GetProMessageRequest, ImageAnswerRequest, ImageChatRequest, ImageContextRequest, ImageGenPromptRequest, InstructRequest, JoinKolRoomRequest, JoinRoomRequest, KnowLedgesRequest, PatoIssEditRequest, QueryEmbeddingRequest, RevealAnswerRequest, ShareKnowLedgesRequest, SummaryAndEmbeddingRequest, SvcImageDescriptionRequest
        }, ChatMessage, PatoInfo, SessionMessages, BATTERY_GRPC_REST_SERVER, BATTERY_GRPC_SERVER_PORT_START
};
use serde::{Deserialize, Serialize};

use crate::service::{CreateResonse, HotAiResponse, HotTopicResponse, KolListResponse, NameResponse, PatoInfoResponse, RoomCreateResponse, RoomListResponse, SharedKnowledgesResponse, SimpleResponse, SnResponse, TokenResponse, TopicChatHisResponse};
use crate::{KolInfo, PortalRoomInfo};

#[derive(Deserialize, Debug, Default, Serialize)]
struct PortalHotAi{
    id: String,
    name: String,
    talks: i32,
    pros: String,
}
#[derive(Deserialize, Debug, Default, Serialize)]
struct PortalPatoOfPro{
    id: String,
    name: String,
    subjects: Vec<String>
}
#[derive(Deserialize, Debug, Default, Serialize)]
struct PortalKnowledge{
    sig: String,
    title: String,
    owner: String,
    summary: String,
}

pub async fn town_login(id: String) -> Result<(), Error> {
    let req = crate::service::LoginRequest { id };

    match call_update_method(NAIS_MATRIX_CANISTER, "request_pato_login", req).await{
        Ok(_) => {
            log!("login success");
        }
        Err(e) => { log!("connect matrix error: {}", e); }
    }

    Ok(())
}
pub async fn town_hots() -> String{
    let req = super::EmptyRequest{};
    match call_update_method(NAIS_MATRIX_CANISTER, "request_hot_ai", req).await{
        Ok(response) => {
            let result = Decode!(response.as_slice(), HotAiResponse).unwrap_or_default();
            let hots = result.sheniu;
            let resp = hots.iter().map(|h|
                PortalHotAi{
                    id: h.id.clone(),
                    name: h.name.clone(),
                    talks: h.talks,
                    pros: h.pros.clone(),
                }
            ).collect::<Vec<PortalHotAi>>();
        
            return serde_json::to_string(&resp).unwrap_or_default();        
        }
        Err(e) => { log!("connect matrix error: {}", e); }
    }

    String::default()
}
pub async fn town_hot_topics() -> String{
    let req = super::EmptyRequest{};
    match call_update_method(NAIS_MATRIX_CANISTER, "request_hot_topics", req).await{
        Ok(response) => {
            let result = Decode!(response.as_slice(), HotTopicResponse).unwrap_or_default();
            let topics = result.topics.clone();
            return serde_json::to_string(&topics).unwrap_or_default();
        }
        Err(e) => { log!("call matrix canister error: {}", e); }
    }

    String::default()
}
pub async fn shared_knowledges() -> String{
    let req = super::EmptyRequest{};
    match call_update_method(NAIS_MATRIX_CANISTER, "request_shared_knowledges", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SharedKnowledgesResponse).unwrap_or_default();
                let hots = response.books;
                let resp = hots.iter().map(|h|
                    PortalKnowledge{
                        sig: h.sig.clone(),
                        title: h.title.clone(),
                        owner: h.owner.clone(),
                        summary: h.summary.clone(),
                    }
                ).collect::<Vec<PortalKnowledge>>();
            
                return serde_json::to_string(&resp).unwrap_or_default();        
            }
        Err(e) => { log!("connect matrix error: {}", e); }
    }

    String::default()
}

pub async fn town_register(name: String) -> Result<String, Error> {
    let req = super::CreateRequest { name };
    match call_update_method(NAIS_MATRIX_CANISTER, "request_create_pato", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), CreateResonse).unwrap_or_default();
            return Ok(response.id);
        }
        Err(e) => { log!("connect matrix error: {}", e); }
    }

    Ok(String::default())
}

pub async fn do_summary_and_embedding(id: String, link: String, transcript: String, knowledge: String, 
    tanscript_sig: String, knowledge_sig: String, link_sig: String
) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let request = tonic::Request::new(SummaryAndEmbeddingRequest {
                link,
                knowledge_file: knowledge,
                transcript_file: transcript,
                knowledge_file_sig: knowledge_sig,
                transcript_file_sig: tanscript_sig,
                link_sig,
            });
            println!("doc summary request {:?}", request);
            let _ = client.request_summary_and_embedding(request).await?;
        }
    }

    Ok(())
}

pub async fn get_pato_info(id: String) -> Result<PatoInfo, Error> {
    let req = super::SimpleRequest {
        id,
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_info", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), PatoInfoResponse).unwrap_or_default();

            let pato_info = PatoInfo {
                id: response.id.clone(),
                name: response.name.clone(),
                sn: response.sn,
                matrix_datetime: get_now_date_str(),
                registered_datetime: response.registered_datetime.clone(),
                professionals: response.professionals.clone(),
                balance: response.balance,
                tags: response.tags.clone(),
                avatar: response.avatar.clone(),
            };
            Ok(pato_info)
        }
        Err(e) => { 
            Err(anyhow!("request_pato_info error: {}", e))
        }
    }
}
pub async fn retrieve_pato_by_name(name: String) -> Result<String, Error> {
    let req = super::SimpleRequest {
        id: name,
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_by_name", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), NameResponse).unwrap_or_default();

            let mut patos: Vec<PortalPatoOfPro> = vec![];
            for pato in response.name_pros.iter() {
                let i = PortalPatoOfPro{
                    id: pato.id.clone(),
                    subjects: pato.pros.clone(),
                    name: pato.name.clone(),
                };
                patos.push(i);
            }
            Ok(serde_json::to_string(&patos).unwrap_or_default())
        }
        Err(e) => { 
            Err(anyhow!("request_pato_info error: {}", e))
        }
    }
}
pub async fn get_name_by_id(ids: Vec<String>) -> Result<String, Error> {
    let req = super::NameRequest {
        id: ids,
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_by_ids", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), NameResponse).unwrap_or_default();

            let mut patos: Vec<PortalPatoOfPro> = vec![];
            for pato in response.name_pros.iter() {
                let i = PortalPatoOfPro{
                    id: pato.id.clone(),
                    subjects: pato.pros.clone(),
                    name: pato.name.clone(),
                };
                patos.push(i);
            }
            Ok(serde_json::to_string(&patos).unwrap_or_default())
        }
        Err(e) => { 
            Err(anyhow!("request_pato_info error: {}", e))
        }
    }
}

pub async fn get_pato_iss(id: String) -> Result<String, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(EmptyRequest {});
            if let Ok(resp) = client.request_pato_iss(req).await{
                return Ok(resp.get_ref().iss.clone());
            }
        }
    }

    Err(anyhow!("get_pato_iss error"))
}

pub async fn archive_pato_session(id: String, session: String, date: String) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(ArchiveMessageRequest { session, date });
            if client.archive_chat_messages(req).await.is_ok(){
                return Ok(());
            }
        }
    }

    Err(anyhow!("change_pato_iss error"))
}
pub async fn edit_pato_iss(id: String, iss: String) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(PatoIssEditRequest { iss });
            if client.change_pato_iss(req).await.is_ok(){
                return Ok(());
            }
        }
    }

    Err(anyhow!("change_pato_iss error"))
}
pub async fn send_pato_instruct(id: String, command: String, pro: String) -> Result<String, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(InstructRequest { 
                message: command, 
                reply_to: id,
                kol: pro,
            });
            match client.request_instruct(req).await{
                Ok(answer) => {
                    return Ok(answer.get_ref().answer.clone());
                }
                Err(e) => {
                    println!("send_pato_instruct error: {:?}", e);
                }
            }
        }
    }

    Err(anyhow!("send_pato_instruct error"))
}
pub async fn create_game_room(id: String, title: String, description: String, town: String) -> Result<Vec<String>, Error> {
    let req = super::RoomCreateRequest {
        owner: id.clone(),
        title,
        town,
        description,
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_room_create", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), RoomCreateResponse).unwrap_or_default();
            return Ok(vec![response.room_id.clone(), response.cover.clone()]);
        }
        Err(e) => {
            log!("request_create_room error: {}", e);
        }    
    }

    Err(anyhow!("request_create_room error"))
}
pub async fn join_game(id: String, owner: String, room_id: String, room_name: String, game_level: i32) -> Result<(i32, String), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(JoinRoomRequest { 
                id,
                name: room_name,
                room_id,
                level: game_level,
            });
            match client.request_join_room(req).await{
                Ok(answer) => {
                    return Ok((answer.get_ref().scene_count, answer.get_ref().last_scene.clone()));
                }
                Err(e) => {
                    return Err(anyhow!("call request_join_room fail: {}",e));
                }
            }
        }
    }
    Err(anyhow!("request_create_room error"))
}
pub async fn request_game_clue(id: String, owner: String, message: String, image_url: String) -> Result<Vec<String>, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(ImageChatRequest { 
                reply_to: owner,
                message,
                image_url,
                room_id: "".to_string(),
                level: 1,
            });
            match client.request_clue_from_image_chat(req).await{
                Ok(answer) => {
                    return Ok(vec![answer.get_ref().answer.clone(), answer.get_ref().answer_voice.clone()]);
                }
                Err(e) => {
                    return Err(anyhow!("call request_game_clue fail: {}",e));
                }
            }
        }
    }
    Err(anyhow!("request_game_clue error"))
}
pub async fn request_game_context(id: String, prompt: String, input: String, image_url: String) -> Result<String, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(ImageContextRequest { 
                image_url,
                prompt,
                input,
            });
            match client.request_image_context(req).await{
                Ok(answer) => {
                    return Ok(answer.get_ref().context.clone());
                }
                Err(e) => {
                    return Err(anyhow!("call request_game_context fail: {}",e));
                }
            }
        }
    }
    Err(anyhow!("request_game_context error"))
}
pub async fn request_image_prompt(id: String, description: String, history: String, architecture: String) -> Result<String, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(ImageGenPromptRequest { 
                description,
                historical: history,
                architectural: architecture,
            });
            match client.request_image_gen_prompt(req).await{
                Ok(answer) => {
                    return Ok(answer.get_ref().context.clone());
                }
                Err(e) => {
                    return Err(anyhow!("call request_image_prompt fail: {}",e));
                }
            }
        }
    }
    Err(anyhow!("request_image_prompt error"))
}
pub async fn send_answer(id: String, owner: String, room_id: String, room_name: String, answer: String, level: i32) -> Result<Vec<String>, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(GameAnswerRequest { 
                id,
                name: room_name,
                answer,
                room_id,
                level,
            });
            match client.receive_game_answer(req).await{
                Ok(resp) => {
                    return Ok(resp.get_ref().correct_gamers.clone());
                }
                Err(e) => {
                    println!("send_answer error: {:?}", e);
                    return Err(anyhow!("call send_answer fail: {}",e));
                }
            }
        }
    }
    Err(anyhow!("send_answer error"))
}
pub async fn accept_answer(owner: String, room_id: String, room_name: String) -> Result<Vec<String>, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![owner.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(EmptyRequest {});
            match client.accept_game_answer(req).await {
                Ok(answer) => {
                    return Ok(answer.get_ref().correct_gamers.clone());
                }
                Err(e) => {
                    return Err(anyhow!("call accept_answer fail: {}",e));
                }
            }
        }
    }
    Err(anyhow!("accept_answer error"))
}
pub async fn chat_with_image(id: String, command: String, pro: String, image_url: String) -> Result<String, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(ImageChatRequest { 
                message: command, 
                reply_to: id,
                image_url,
                room_id: "".to_string(),
                level: 1,
            });
            match client.request_chat_with_image(req).await{
                Ok(answer) => {
                    let answer_txt = answer.get_ref().answer.clone();
                    let answer_voice = answer.get_ref().answer_voice.clone();
                    return Ok(serde_json::to_string(&vec![answer_txt, answer_voice]).unwrap_or_default());
                }
                Err(e) => {
                    println!("send_pato_instruct error: {:?}", e);
                }
            }
        }
    }

    Err(anyhow!("send_pato_instruct error"))
}
pub async fn reveal_answer(id: String, room_id: String, level: i32, owner: String) -> Result<String, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![owner.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(RevealAnswerRequest { 
                room_id,
                level,
                id,
                owner,
            });
            match client.request_reveal_answer(req).await{
                Ok(answer) => {
                    return Ok(answer.get_ref().answer.clone());
                }
                Err(e) => {
                    println!("send_pato_instruct error: {:?}", e);
                }
            }
        }
    }

    Err(anyhow!("send_pato_instruct error"))
}
pub async fn answer_image(id: String, image_url: String, room_id: String, level: i32, input: String, prompt: String) -> Result<String, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(ImageAnswerRequest { 
                image_url,
                room_id,
                level,
                input,
                prompt,
            });
            match client.request_answer_image(req).await{
                Ok(answer) => {
                    return Ok(answer.get_ref().context.clone());
                }
                Err(e) => {
                    println!("send_pato_instruct error: {:?}", e);
                }
            }
        }
    }

    Err(anyhow!("send_pato_instruct error"))
}
pub async fn image_description(id: String, demo_image_file: String) -> Result<String, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0{
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(SvcImageDescriptionRequest {
                image_url: demo_image_file.clone(),
            });
            match client.request_image_description(req).await{
                Ok(resp) => {
                    return Ok(resp.get_ref().description.clone() + "##" + &demo_image_file);
                }
                Err(e) => {
                    println!("generate_scene error: {:?}", e);
                }
            }
        }
    }

    Err(anyhow!("generate_scene error"))
}
pub async fn call_pato(id: String, callid: String, topic: String) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(CallRequest { id: callid, topic });
            if client.request_pato_call(req).await.is_ok(){
                return Ok(());
            }
        }
    }

    Err(anyhow!("call pato error"))
}
pub async fn get_pato_chat_messages(id: String, date: String) -> Result<Vec<SessionMessages>, Error> {
    let mut messages: Vec<SessionMessages> = vec![];
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(GetMessageRequest { id, date });
            // println!("get_pato_chat_messages: {:?}", req);
            if let Ok(resp) = client.get_chat_messages(req).await{
                let messages_json_str = resp.get_ref().content.clone();
                // println!("get_pato_chat_messages: {:?}", messages_json_str);
                messages = serde_json::from_str(&messages_json_str).unwrap_or_default();
            }
        }
    }

    Ok(messages)
}
pub async fn edit_pato_chat_messages(id: String, kol: String, messages: Vec<ChatMessage>) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let messages_str = serde_json::to_string(&messages).unwrap_or_default();
            let req = tonic::Request::new(EditeReqeust { initial: id, kol, messages: messages_str });
            if let Err(e) = client.request_edit_messages(req).await{
                println!("edit messages error: {}", e);
            }
        }
    }

    Ok(())
}
pub async fn continue_pato_chat(id: String, date: String, session: String, continued: bool) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(ContinueRequest { date, session, continued });
            // println!("get_pato_chat_messages: {:?}", req);
            if let Err(e) = client.request_continue_chat(req).await{
                println!("continue chat error: {}", e);
            }
        }
    }

    Ok(())
}
pub async fn topic_chat(id: String, topic: String, town: String) -> Result<(), Error> {
    let req = super::TopicChatRequest {
        initial: id.clone(),
        topic,
        town,
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_topic_chat", req).await{
        Ok(_) => {}
        Err(e) => { log!("request_topic_chat error: {}", e); }
    }

    Ok(())
}
pub async fn log_user_activity(id: String, page: String, action: String) -> Result<(), Error> {
    let req = super::UserActiveRequest {
        id,
        page,
        action,
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_topic_chat", req).await{
        Ok(_) => {}
        Err(e) => { log!("request_topic_chat error: {}", e); }
    }

    Ok(())
}
pub async fn get_topic_chat_history(id: String, topic: String, town: String) -> Result<String, Error> {
    let req = super::TopicChatRequest {
        initial: id.clone(),
        topic,
        town,
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_topic_chat_history", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), TopicChatHisResponse).unwrap_or_default();
            return Ok(serde_json::to_string(&response.history).unwrap_or_default());
        }
        Err(e) => {
            println!("request_topic_chat_history error: {}", e);
        }
    }

    Err(anyhow!("request_topic_chat_history error"))
}
pub async fn get_pro_chat_messages(id: String, proid:String, date: String) -> Result<Vec<ChatMessage>, Error> {
    let mut messages: Vec<ChatMessage> = vec![];
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(GetProMessageRequest { id, date, proid});
            // println!("get_pato_chat_messages: {:?}", req);
            if let Ok(resp) = client.get_pro_chat_messages(req).await{
                let messages_json_str = resp.get_ref().content.clone();
                // println!("get_pro_chat_messages: {:?}", messages_json_str);
                messages = serde_json::from_str(&messages_json_str).unwrap_or_default();
            }
        }
    }

    Ok(messages)
}
pub async fn get_pro_knowledges(id: String) -> Result<String, Error> {
    let mut messages: Vec<PortalKnowledge> = vec![];
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(KnowLedgesRequest { id: id.clone() });
            if let Ok(resp) = client.request_pato_knowledges(req).await{
                for knowledge in resp.get_ref().knowledge_info.clone(){
                    let info = PortalKnowledge{
                        sig: knowledge.sig.clone(),
                        title: knowledge.title.clone(),
                        owner: knowledge.owner.clone(),
                        summary: knowledge.summary.clone(),
                    };
                    messages.push(info);
                }
            }
        }
    }

    Ok(serde_json::to_string(&messages).unwrap_or_default())
}
pub async fn pato_self_talk(id: String) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(EmptyRequest {});
            if let Ok(resp) = client.request_self_talk_for_today_plan(req).await{
            }
        }
    }

    Ok(())
}
pub fn get_predefined_tags() -> Result<String, Error> {
    let tags_json_file = format!("{}/template/tags.json", AI_MATRIX_DIR);
    let mut file = OpenOptions::new().read(true).open(tags_json_file)?;
    let mut content: String = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}
pub async fn submit_tags(id: String, tags: Vec<String>) -> Result<String, Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(SubmitTagsRequest {
                tags, 
            });
            match client.request_submit_tags(req).await{
                Ok(resp) => {
                    return Ok(resp.get_ref().avatar.clone());
                }
                Err(e) =>{
                    return Err(anyhow!("submit tags is something wrong with {}",e));
                }
            }
        }
    }

    Err(anyhow!("submit tags is something wrong with"))
}

pub async fn share_pro_knowledge(id: String, sig: String, title: String, shared: bool) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(ShareKnowLedgesRequest {
                sig,
                title,
                owner: id, 
            });
            if let Err(e) = client.request_share_knowledge(req).await{
                log!("share knowledge error: {}", e);
            }
        }
    }

    Ok(())
}
pub async fn add_shared_knowledge(id: String, sig: String, title: String, owner: String) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(ShareKnowLedgesRequest {
                sig,
                title,
                owner, 
            });
            if let Err(e) = client.add_shared_knowledge(req).await{
                log!("add_shared_knowledge error: {}", e);
            }
        }
    }

    Ok(())
}
pub async fn gen_pato_auth_token(id: String) -> Result<String, Error> {
    let mut token = "".to_string();

    let req = super::SimpleRequest { id };
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_auth_token", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SimpleResponse).unwrap_or_default();
            token = response.message.clone();
        }
        Err(e) => { log!("request_pato_auth_token error: {}", e); }
    }

    Ok(token)
}
pub async fn query_pato_auth_token(token: String) -> Result<(String, String), Error> {
    let mut id = "".to_string();
    let mut name = "".to_string();

    let req = super::TokenRequest { token };
    match call_update_method(AGENT_SMITH_CANISTER, "query_pato_auth_token", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), TokenResponse).unwrap_or_default();
            id = response.id.clone();
            name = response.name.clone();
        }
        Err(e) => { log!("query_pato_auth_token error: {}", e); }
    }

    Ok((id, name))
}
pub async fn query_game_rooms(town: String) -> Result<String, Error> {
    let mut rooms: Vec<PortalRoomInfo> = vec![];
    let req = super::SimpleRequest { id: town};
    match call_update_method(AGENT_SMITH_CANISTER, "request_room_list", req).await{
        Ok(result) => {
            let resp = Decode!(result.as_slice(), RoomListResponse).unwrap_or_default();

            for response in resp.rooms.iter(){
                let info = PortalRoomInfo{
                    room_id: response.room_id.clone(),
                    owner: response.owner.clone(),
                    title: response.title.clone(),
                    description: response.description.clone(),
                    town: response.town.clone(),
                    cover: response.cover.clone(),
                };
                rooms.push(info);
            }
        }
        Err(e) => { log!("request_room_list error: {}", e); }
    }

    Ok(serde_json::to_string(&rooms).unwrap_or_default())
}
pub async fn query_kol_rooms() -> Result<String, Error> {
    let mut rooms: Vec<KolInfo> = vec![];
    let req = super::EmptyRequest {    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_kol_list", req).await{
        Ok(result) => {
            let resp = Decode!(result.as_slice(), KolListResponse).unwrap_or_default();

            for response in resp.relations.iter(){
                let mut avatar_link = format!("{}/avatar/{}/avatar.png", XFILES_SERVER, response.id);
                let avatar = format!("{}/avatar/{}/avatar.png", XFILES_LOCAL_DIR, response.id);
                if !Path::new(&avatar).exists() {
                    avatar_link = "".to_string();
                }
                let info = KolInfo{
                    id: response.id.clone(),
                    name: response.name.clone(),
                    followers: response.follower.clone(),
                    avatar: avatar_link,
                };
                rooms.push(info);
            }
        }
        Err(e) => { log!("request_kol_list error: {}", e); }
    }

    Ok(serde_json::to_string(&rooms).unwrap_or_default())
}
pub async fn become_kol(id: String) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(BecomeKolRequest { key: String::default() });
            println!("become_kol: {:?}", req);
            if let Err(e) = client.become_kol(req).await{
                println!("become_kol error: {}", e);
            }
        }
    }

    Ok(())
}
pub async fn follow_kol(kol: String, follower: String) -> Result<(), Error> {
    let mut sn: i64 = -1;
    let req = super::SnRequest {
        id: vec![follower.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(JoinKolRoomRequest { key: String::default(), kol, follower });
            println!("request_join_kol_room: {:?}", req);
            if let Err(e) = client.request_join_kol_room(req).await{
                println!("request_join_kol_room error: {}", e);
            }
        }
    }

    Ok(())
}
pub async fn query_document_embeddings(id: String, sig: String, query: String) -> Result<String, Error>{
    let mut sn: i64 = -1;
    let mut query_result = String::new();
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(QueryEmbeddingRequest { 
                query, collection_name: sig 
            });
            let query_resp = client.request_query_embedding(req).await?;
            query_result = query_resp.get_ref().result.clone();
        }
    }

    Ok(query_result)
}
pub async fn query_document_summary(id: String, sig: String) -> Result<String, Error>{
    let mut sn: i64 = -1;
    let mut query_result = String::new();
    let req = super::SnRequest {
        id: vec![id.clone()],
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
        Ok(result) => {
            let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
            let resp = response.pato_sn_id;
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("get_pato_sn: not found this one");
            }
        }
        Err(e) => { log!("get_pato_sn error: {}", e); }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(DocumentSummaryRequest { 
                document: sig,
            });
            let summary_resp = client.request_document_summary(req).await?;
            query_result = summary_resp.get_ref().summary.clone();
        }
    }

    Ok(query_result)
}
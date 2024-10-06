use anyhow::{anyhow, Error};
use candid::Decode;
use metapower_framework::icp::{
    call_query_method, call_update_method, AGENT_BATTERY_CANISTER, AGENT_SMITH_CANISTER,
    NAIS_MATRIX_CANISTER,
};
use metapower_framework::{get_now_date_str, log};
use metapower_framework::{
    ChatMessage, PatoInfo, SessionMessages, AI_MATRIX_DIR, XFILES_LOCAL_DIR, XFILES_SERVER,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Read;
use std::path::Path;
use std::time::SystemTime;
use std::{fs::OpenOptions, io::Write};

use crate::service::{
    CreateResonse, HotAiResponse, HotTopicResponse, KolListResponse, NameResponse,
    PatoInfoResponse, RoomCreateResponse, SharedKnowledgesResponse,
    SimpleResponse, TokenResponse, TopicChatHisResponse,
};
use crate::KolInfo;

use super::{
    ArchiveMessageRequest, BecomeKolRequest, CallRequest, ContinueRequest, DocumentSummaryRequest,
    EditeReqeust, EmptyRequest, GameAnswerRequest, GetMessageRequest, ImageAnswerRequest,
    ImageChatRequest, ImageGenPromptRequest, InstructRequest, JoinKolRoomRequest,
    KnowLedgesRequest, QueryEmbeddingRequest, ShareKnowLedgesRequest, SubmitTagsRequest,
    SummaryAndEmbeddingRequest, SvcImageDescriptionRequest,
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

fn prepare_battery_call_args<T: Serialize>(
    id: String,
    token: String,
    sn: i64,
    method_name: String,
    arg: T,
) -> (String, String, i64, String, String) {
    (
        id,
        token,
        sn,
        method_name,
        serde_json::to_string(&arg).unwrap_or_default(),
    )
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
    let req = super::EmptyRequest {};
    match call_update_method(NAIS_MATRIX_CANISTER, "request_hot_ai", req).await {
        Ok(response) => {
            let result = Decode!(response.as_slice(), HotAiResponse).unwrap_or_default();
            let hots = result.sheniu;
            let resp = hots
                .iter()
                .map(|h| PortalHotAi {
                    id: h.id.clone(),
                    name: h.name.clone(),
                    talks: h.talks,
                    pros: h.pros.clone(),
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
    let req = super::EmptyRequest {};
    match call_update_method(NAIS_MATRIX_CANISTER, "request_hot_topics", req).await {
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
    let req = super::EmptyRequest {};
    match call_update_method(NAIS_MATRIX_CANISTER, "request_shared_knowledges", req).await {
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

pub async fn do_summary_and_embedding(
    id: String,
    link: String,
    transcript: String,
    knowledge: String,
    tanscript_sig: String,
    knowledge_sig: String,
    link_sig: String,
) -> Result<(), Error> {
    let request = SummaryAndEmbeddingRequest {
        link,
        knowledge_file: knowledge,
        transcript_file: transcript,
        knowledge_file_sig: knowledge_sig,
        transcript_file_sig: tanscript_sig,
        link_sig,
    };
    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_summary_and_embedding".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(result) => {}
        Err(e) => {
            log!("get_pato_sn error: {}", e);
        }
    }

    Ok(())
}

pub async fn get_pato_info(id: String) -> Result<PatoInfo, Error> {
    match call_query_method(AGENT_SMITH_CANISTER, "request_pato_info", id).await {
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
        Err(e) => Err(anyhow!("request_pato_info error: {}", e)),
    }
}
pub async fn retrieve_pato_by_name(name: String) -> Result<String, Error> {
    let req = super::SimpleRequest { id: name };
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_by_name", req).await {
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

pub async fn archive_pato_session(id: String, session: String, date: String) -> Result<(), Error> {
    let request = ArchiveMessageRequest { session, date };
    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "archive_chat_messages".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(result) => {}
        Err(e) => {
            log!("get_pato_sn error: {}", e);
        }
    }

    Ok(())
}

pub async fn send_pato_instruct(id: String, command: String, pro: String) -> Result<String, Error> {
    let mut answer = "我这会儿有点忙～～".to_string();

    let request = InstructRequest {
        message: command,
        reply_to: id.clone(),
        kol: pro,
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_instruct".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("get_pato_sn error: {}", e);
        }
    }

    Ok(answer)
}

pub async fn create_game_room(
    id: String,
    title: String,
    description: String,
    town: String,
) -> Result<Vec<String>, Error> {
    let req = super::RoomCreateRequest {
        owner: id.clone(),
        title,
        town,
        description,
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_room_create", req).await {
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
pub async fn request_image_prompt(
    id: String,
    description: String,
    history: String,
    architecture: String,
) -> Result<String, Error> {
    let mut answer = "".to_string();

    let request = ImageGenPromptRequest {
        description,
        historical: history,
        architectural: architecture,
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_image_gen_prompt".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("get_pato_sn error: {}", e);
        }
    }

    Ok(answer)
}
pub async fn send_answer(
    id: String,
    owner: String,
    room_id: String,
    room_name: String,
    answer: String,
    level: i32,
) -> Result<Vec<String>, Error> {
    let mut answers = vec![];

    let request = GameAnswerRequest {
        id: id.clone(),
        name: room_name,
        answer,
        room_id,
        level,
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "receive_game_answer".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("get_pato_sn error: {}", e);
        }
    }

    Ok(answers)
}
pub async fn chat_with_image(
    id: String,
    command: String,
    pro: String,
    image_url: String,
) -> Result<String, Error> {
    let mut answer = "".to_string();

    let request = ImageChatRequest {
        message: command,
        reply_to: id.clone(),
        image_url,
        room_id: "".to_string(),
        level: 1,
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_chat_with_image".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("get_pato_sn error: {}", e);
        }
    }

    Ok(answer)
}
pub async fn answer_image(
    id: String,
    image_url: String,
    room_id: String,
    level: i32,
    input: String,
    prompt: String,
) -> Result<String, Error> {
    let mut answer = "这幅画有点深奥啊。。。".to_string();

    let request = ImageAnswerRequest {
        image_url,
        room_id,
        level,
        input,
        prompt,
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_answer_image".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("get_pato_sn error: {}", e);
        }
    }

    Ok(answer)
}
pub async fn image_description(id: String, demo_image_file: String) -> Result<String, Error> {
    let mut answer = "这幅画有点深奥啊。。。".to_string();

    let request = SvcImageDescriptionRequest {
        image_url: demo_image_file.clone(),
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_image_description".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(answer)
}
pub async fn call_pato(id: String, callid: String, topic: String) -> Result<(), Error> {
    let request = CallRequest { id: callid, topic };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_pato_call".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(())
}
pub async fn get_pato_chat_messages(
    id: String,
    date: String,
) -> Result<Vec<SessionMessages>, Error> {
    let mut messages: Vec<SessionMessages> = vec![];

    let request = GetMessageRequest {
        id: id.clone(),
        date,
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "get_chat_messages".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(messages)
}
pub async fn edit_pato_chat_messages(
    id: String,
    kol: String,
    messages: Vec<ChatMessage>,
) -> Result<(), Error> {
    let messages_str = serde_json::to_string(&messages).unwrap_or_default();
    let request = EditeReqeust {
        initial: id.clone(),
        kol,
        messages: messages_str,
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_edit_messages".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(())
}
pub async fn continue_pato_chat(
    id: String,
    date: String,
    session: String,
    continued: bool,
) -> Result<(), Error> {
    let request = ContinueRequest {
        date,
        session,
        continued,
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_continue_chat".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
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
    match call_update_method(AGENT_SMITH_CANISTER, "request_topic_chat", req).await {
        Ok(_) => {}
        Err(e) => {
            log!("request_topic_chat error: {}", e);
        }
    }

    Ok(())
}
pub async fn log_user_activity(id: String, page: String, action: String) -> Result<(), Error> {
    let req = super::UserActiveRequest { id, page, action };
    match call_update_method(AGENT_SMITH_CANISTER, "request_topic_chat", req).await {
        Ok(_) => {}
        Err(e) => {
            log!("request_topic_chat error: {}", e);
        }
    }

    Ok(())
}
pub async fn get_topic_chat_history(
    id: String,
    topic: String,
    town: String,
) -> Result<String, Error> {
    let req = super::TopicChatRequest {
        initial: id.clone(),
        topic,
        town,
    };
    match call_update_method(AGENT_SMITH_CANISTER, "request_topic_chat_history", req).await {
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
pub async fn get_pro_knowledges(id: String) -> Result<String, Error> {
    let mut messages: Vec<PortalKnowledge> = vec![];

    let request = KnowLedgesRequest { id: id.clone() };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_pato_knowledges".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(serde_json::to_string(&messages).unwrap_or_default())
}
pub async fn pato_self_talk(id: String) -> Result<(), Error> {
    let request = EmptyRequest {};

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_self_talk_for_today_plan".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(())
}
pub async fn get_predefined_tags() -> Result<String, Error> {
    match call_update_method(AGENT_SMITH_CANISTER, "request_predefined_tags", ()).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), String).unwrap_or_default();
            return Ok(response);
        }
        Err(e) => {
            return Err(anyhow!("get_predefined_tags error: {}", e));
        }
    }
}
pub async fn submit_tags(id: String, tags: Vec<String>) -> Result<String, Error> {
    let avatar = String::default();
    let request = SubmitTagsRequest { tags };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_submit_tags".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(avatar)
}

pub async fn share_pro_knowledge(
    id: String,
    sig: String,
    title: String,
    shared: bool,
) -> Result<(), Error> {
    let request = ShareKnowLedgesRequest {
        sig,
        title,
        owner: id.clone(),
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_share_knowledge".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(())
}
pub async fn add_shared_knowledge(
    id: String,
    sig: String,
    title: String,
    owner: String,
) -> Result<(), Error> {
    let request = ShareKnowLedgesRequest { sig, title, owner };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "add_shared_knowledge".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(())
}
pub async fn gen_pato_auth_token(id: String) -> Result<String, Error> {
    let mut token = "".to_string();

    let req = super::SimpleRequest { id };
    match call_update_method(AGENT_SMITH_CANISTER, "request_pato_auth_token", req).await {
        Ok(result) => {
            let response = Decode!(result.as_slice(), SimpleResponse).unwrap_or_default();
            token = response.message.clone();
        }
        Err(e) => {
            log!("request_pato_auth_token error: {}", e);
        }
    }

    Ok(token)
}
pub async fn query_pato_auth_token(token: String) -> Result<(String, String), Error> {
    let mut id = "".to_string();
    let mut name = "".to_string();

    let req = super::TokenRequest { token };
    match call_update_method(AGENT_SMITH_CANISTER, "query_pato_auth_token", req).await {
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
    let mut rooms: Vec<KolInfo> = vec![];
    let req = super::EmptyRequest {};
    match call_update_method(AGENT_SMITH_CANISTER, "request_kol_list", req).await {
        Ok(result) => {
            let resp = Decode!(result.as_slice(), KolListResponse).unwrap_or_default();

            for response in resp.relations.iter() {
                let mut avatar_link =
                    format!("{}/avatar/{}/avatar.png", XFILES_SERVER, response.id);
                let avatar = format!("{}/avatar/{}/avatar.png", XFILES_LOCAL_DIR, response.id);
                if !Path::new(&avatar).exists() {
                    avatar_link = "".to_string();
                }
                let info = KolInfo {
                    id: response.id.clone(),
                    name: response.name.clone(),
                    followers: response.follower.clone(),
                    avatar: avatar_link,
                };
                rooms.push(info);
            }
        }
        Err(e) => {
            log!("request_kol_list error: {}", e);
        }
    }

    Ok(serde_json::to_string(&rooms).unwrap_or_default())
}
pub async fn become_kol(id: String) -> Result<(), Error> {
    let request = BecomeKolRequest { key: id.clone() };

    let req = prepare_battery_call_args(id, "".to_string(), -1, "become_kol".to_string(), request);

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(())
}
pub async fn follow_kol(kol: String, follower: String) -> Result<(), Error> {
    let request = JoinKolRoomRequest {
        key: String::default(),
        kol: kol.clone(),
        follower,
    };

    let req = prepare_battery_call_args(
        kol,
        "".to_string(),
        -1,
        "request_join_kol_room".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(())
}
pub async fn query_document_embeddings(
    id: String,
    sig: String,
    query: String,
) -> Result<String, Error> {
    let mut query_result = String::new();

    let request = QueryEmbeddingRequest {
        query,
        collection_name: sig,
    };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_query_embedding".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(query_result)
}
pub async fn query_document_summary(id: String, sig: String) -> Result<String, Error> {
    let mut query_result = String::new();

    let request = DocumentSummaryRequest { document: sig };

    let req = prepare_battery_call_args(
        id,
        "".to_string(),
        -1,
        "request_document_summary".to_string(),
        request,
    );

    match call_update_method(AGENT_BATTERY_CANISTER, "do_battery_service", req).await {
        Ok(answer) => {}
        Err(e) => {
            log!("request_image_description error: {}", e);
        }
    }

    Ok(query_result)
}

pub mod dao;
pub mod model;
pub mod service;

use crate::service::ai_town::{image_description, log_user_activity, town_register};
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{
    http::header::{ContentDisposition, ContentType, DispositionType},
    middleware, web, App, Error, HttpResponse, HttpServer, Responder,
};
use futures::StreamExt;
use futures::TryStreamExt;
use metapower_framework::{
    get_now_secs_str_zh, ChatMessage, DataResponse, AI_AGENT_DIR, AI_PATO_DIR, XFILES_SERVER,
};
use serde::{Deserialize, Serialize};
use service::{
    ai_town::{
        add_shared_knowledge, become_kol, call_pato, continue_pato_chat, do_summary_and_embedding, edit_pato_chat_messages, follow_kol, get_name_by_id, get_pato_chat_messages, get_pato_info, get_predefined_tags, get_pro_knowledges, get_topic_chat_history, pato_self_talk, query_document_summary, query_kol_rooms, query_pato_auth_token, query_pato_by_kol_token, query_pato_kol_token, refresh_pato_auth_token, retrieve_pato_by_name, share_pro_knowledge, shared_knowledges, submit_tags, topic_chat, town_hot_topics, town_hots, town_login
    },
    bsc_proxy::{monitor_pab_transfer_event, proxy_contract_call_query_kol_staking},
};
use sha1::Digest;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

#[derive(Deserialize, Debug)]
struct MessagesEditInfo {
    id: String,
    kol: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize, Debug)]
struct ContinueTalkInfo {
    id: String,
    session: String,
    date: String,
    continued: bool,
}

#[derive(Deserialize, Debug)]
struct TopicChatInfo {
    id: String,
    topic: String,
    town: String,
}

#[derive(Deserialize, Debug)]
struct ShareKnowledgeInfo {
    id: String,
    owner: String,
    sig: String,
    title: String,
    shared: bool,
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    name: String,
    gender: u8,
    personality: String,
}

#[derive(Deserialize, Debug)]
struct ImageChatInfo {
    id: String,
    pro: String,
    message: String,
    image_url: String,
    room_id: String,
    level: i32,
}

#[derive(Deserialize, Debug)]
struct InstructInfo {
    id: String,
    pro: String,
    message: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct PortalRoomInfo {
    room_id: String,
    owner: String,
    title: String,
    description: String,
    cover: String,
    town: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct ImagePromptRequest {
    id: String,
    description: String,
    history: String,
    architecture: String,
}
#[derive(Deserialize, Debug, Serialize)]
struct GenImageAnswerInfo {
    room_id: String,
    level: i32,
    id: String,
    input: String,
    image_url: String,
    prompt: String,
}

#[derive(Deserialize, Debug)]
struct EmbedInfo {
    id: String,
    sig: String,
    query: String,
}

#[derive(Deserialize, Debug, Default)]
struct CallInfo {
    id: String,
    callid: String,
    topic: String,
}

#[derive(Deserialize, Debug, Default)]
struct DescribeSceneInfo {
    id: String,
    room_id: String,
    scene: String,
}

#[derive(Deserialize, Debug, Default)]
struct UserActive {
    id: String,
    page: String,
    action: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct KolInfo {
    id: String,
    name: String,
    avatar: String,
    followers: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ArchiveInfo {
    id: String,
    session: String,
    date: String,
}

#[derive(Deserialize, Debug, Default)]
struct MessageInfo {
    sender: String,
    receiver: String,
    message: String,
}

#[derive(Deserialize, Debug, Default)]
struct SummaryKnowledgeInfo {
    id: String,
    link: String,
    transcript: String,
    shared: String,
}

#[derive(Deserialize, Debug, Default)]
struct KnowledgeInfo {
    id: String,
    message: String,
    #[serde(default)]
    domain: String,
}

#[derive(Deserialize)]
pub struct PathInfo {
    absolute_path: String,
}

#[derive(Deserialize, Debug, Default)]
struct TravelSceneInfo {
    pub id: String,
    pub room_id: String,
    pub description: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("monitor event staking");
    tokio::spawn({ monitor_pab_transfer_event() });

    println!("metapower portal rest api @ 8030");
    HttpServer::new(|| {
        App::new()
            .configure(config_app)
            .wrap(Cors::permissive().supports_credentials().max_age(3600))
            .wrap(middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8030))?
    .run()
    .await
}

async fn pong(_query: Option<web::Query<String>>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("pong")
}
async fn get_index() -> HttpResponse {
    HttpResponse::Ok().content_type("text/html").body(
        r#"
                <title>MetaPowerMatrix原力接口，随便想两个数，填在下面，点击提交</title>
                <form action="/whoareyou" method="post">
                <input type="text" name="n"/>
                <input type="text" name="m"/>
                <button type="submit">幸运数字</button>
                </form>
            "#,
    )
}
async fn portal_register(user_info: web::Json<UserInfo>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let info = user_info.into_inner();

    match town_register(info.name).await {
        Ok(id) => {
            resp.content = id;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_kol_list() -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match query_kol_rooms().await {
        Ok(kols) => {
            resp.content = kols;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_become_kol(info: web::Path<String>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let id = info.into_inner();

    match become_kol(id).await {
        Ok(token) => {
            resp.content = token;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_query_kol_staking(info: web::Path<String>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: "0".to_string(),
        code: String::from("404"),
    };

    let id = info.into_inner();

    match proxy_contract_call_query_kol_staking(id).await {
        Ok(staking) => {
            resp.content = staking.to_string();
            resp.code = String::from("200");
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }

    Ok(web::Json(resp))
}
async fn portal_join_kol(info: web::Path<(String, String)>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (follower, kol) = info.into_inner();

    if let Err(e) = follow_kol(kol, follower).await {
        println!("error: {}", e);
        resp.code = String::from("500");
    }

    Ok(web::Json(resp))
}
async fn portal_upload_knowledge(mut payload: Multipart) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    // Initialize variables to hold the file bytes and the message
    let mut file_bytes = Vec::new();
    let mut message_json = String::new();
    let mut hasher = sha1::Sha1::new();
    let mut filename = String::new();
    let mut transcript_file_sig = String::new();
    let mut has_file_uploaded = false;
    let mut link_file_sig = String::new();
    let mut saved_file_name = String::default();

    // Iterate over multipart/form-data
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.unwrap().get_name().unwrap_or("");
        // Get the filename from the content-disposition header
        if filename.is_empty() {
            filename = content_disposition
                .unwrap()
                .get_filename()
                .unwrap_or("knowledge file")
                .to_string();
            println!("Filename: {}", filename);
        }

        match field_name {
            "file" => {
                has_file_uploaded = true;
                // Field with the file
                // println!("file field: {:?}", field);
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    hasher.update(&data);
                    file_bytes.extend_from_slice(&data);
                }
            }
            "message" => {
                // Field with the JSON string
                // println!("message field: {:?}", field);
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    message_json.extend(data.iter().map(|&b| b as char));
                }
            }
            _ => {}
        }
    }

    // Deserialize JSON string to Message struct
    let message: SummaryKnowledgeInfo = serde_json::from_str(&message_json).unwrap_or_default();
    let saved_file_sig = format!("{}/{}/knowledge/knowledge.sig", AI_PATO_DIR, message.id);

    // save the file bytes  to a file
    if !message.transcript.is_empty() {
        match OpenOptions::new()
            .read(true)
            .open(message.transcript.clone())
        {
            Ok(mut file) => {
                let mut transcript = Vec::new();
                file.read(&mut transcript).unwrap_or_default();
                let mut hasher = sha1::Sha1::new();
                hasher.update(&transcript);
                transcript_file_sig = format!("{:x}", hasher.finalize());
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
        if let Ok(mut sig_file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&saved_file_sig)
        {
            let _ = sig_file.write_all(
                format!(
                    "recording-{}#{}\n",
                    get_now_secs_str_zh(),
                    transcript_file_sig
                )
                .as_bytes(),
            );
        }
    }

    if has_file_uploaded {
        saved_file_name = format!("{:x}", hasher.finalize());
        if let Ok(mut sig_file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&saved_file_sig)
        {
            let _ = sig_file.write_all(format!("{}#{}\n", filename, saved_file_name).as_bytes());
        }
        let saved_file_path = format!(
            "{}/{}/knowledge/{}",
            AI_PATO_DIR, message.id, saved_file_name
        );
        println!("saved knowledge path: {}", saved_file_path);
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(saved_file_path)
        {
            Ok(mut file) => {
                file.write_all(&file_bytes).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
                resp.code = String::from("500");
            }
        }
    }

    if !message.link.is_empty() {
        let mut hasher = sha1::Sha1::new();
        hasher.update(&message.link);
        link_file_sig = format!("{:x}", hasher.finalize());

        if let Ok(mut sig_file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&saved_file_sig)
        {
            let _ = sig_file.write_all(format!("{}#{}\n", message.link, link_file_sig).as_bytes());
        }
    }

    match do_summary_and_embedding(
        message.id,
        message.link.clone(),
        message.transcript,
        saved_file_name.clone(),
        transcript_file_sig.clone(),
        saved_file_name.clone(),
        link_file_sig.clone(),
    )
    .await
    {
        Ok(_) => {
            resp.content =
                serde_json::to_string(&vec![saved_file_name, transcript_file_sig, link_file_sig])
                    .unwrap_or_else(|e| {
                        println!("error: {}", e);
                        resp.code = String::from("500");
                        "".to_string()
                    });
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_login(id: web::Path<String>) -> actix_web::Result<impl Responder> {
    let resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let _ = town_login(id.into_inner()).await;

    Ok(web::Json(resp))
}
async fn portal_town_hots() -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    resp.content = town_hots().await;

    Ok(web::Json(resp))
}
async fn portal_town_hot_topics() -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    resp.content = town_hot_topics().await;

    Ok(web::Json(resp))
}

async fn portal_shared_knowledges() -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    resp.content = shared_knowledges().await;

    Ok(web::Json(resp))
}

async fn portal_query_summary(
    data: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (id, sig) = data.into_inner();

    match query_document_summary(id, sig).await {
        Ok(info) => {
            resp.content = info;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_get_predefined_tags() -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match get_predefined_tags().await {
        Ok(content) => resp.content = content,
        Err(e) => {
            println!("read tags json file error: {}", e);
            resp.code = String::from("404")
        }
    }

    Ok(web::Json(resp))
}
async fn portal_submit_tags(
    id: web::Path<String>,
    tags: web::Json<Vec<String>>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match submit_tags(id.into_inner(), tags.into_inner()).await {
        Ok(avatar_url) => resp.content = avatar_url,
        Err(e) =>{
            resp.code = String::from("500");
            resp.content = e.to_string();
        }
    }

    Ok(web::Json(resp))
}

async fn portal_get_pato_info(id: web::Path<String>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let id = id.into_inner();

    match get_pato_info(id).await {
        Ok(info) => {
            resp.content = serde_json::to_string(&info).unwrap_or_else(|e| {
                println!("error: {}", e);
                resp.code = String::from("500");
                "".to_string()
            });
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}

async fn portal_call_pato(call: web::Json<CallInfo>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    if call_pato(call.id.clone(), call.callid.clone(), call.topic.clone())
        .await
        .is_err()
    {
        resp.code = String::from("500");
    }

    Ok(web::Json(resp))
}
async fn portal_upload_image_for_description(
    mut payload: Multipart,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    // Initialize variables to hold the file bytes and the message
    let mut file_bytes = Vec::new();
    let mut message_json = String::new();
    let mut hasher = sha1::Sha1::new();
    let mut has_file_uploaded = false;

    // Iterate over multipart/form-data
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.unwrap().get_name().unwrap_or("");

        match field_name {
            "file" => {
                has_file_uploaded = true;
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    hasher.update(&data);
                    file_bytes.extend_from_slice(&data);
                }
            }
            "message" => {
                // Field with the JSON string
                // println!("message field: {:?}", field);
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    message_json.extend(data.iter().map(|&b| b as char));
                }
            }
            _ => {}
        }
    }

    // Deserialize JSON string to Message struct
    let scene: TravelSceneInfo = serde_json::from_str(&message_json).unwrap_or_default();
    println!("TravelSceneInfo: {:?}", scene);

    let sample_image_url: String;
    if has_file_uploaded {
        // save the file bytes  to a file
        let saved_image_file_name = format!("{:x}", hasher.finalize());
        let saved_image_file_path = format!("/data/www/xfiles/{}", saved_image_file_name);
        sample_image_url = format!("{}/{}", XFILES_SERVER, saved_image_file_name);
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(saved_image_file_path)
        {
            Ok(mut file) => {
                file.write_all(&file_bytes).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
                resp.code = String::from("500");
            }
        }
    } else {
        sample_image_url = scene.description;
    }

    if let Ok(description) = image_description(scene.id.clone(), sample_image_url.clone()).await {
        resp.content = description;
    } else {
        resp.code = String::from("500");
    }

    Ok(web::Json(resp))
}
async fn portal_image_description(
    form: web::Json<DescribeSceneInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    // Deserialize JSON string to Message struct
    let scene: DescribeSceneInfo = form.into_inner();
    println!("DescribeSceneInfo: {:?}", scene);

    if let Ok(description) = image_description(scene.id.clone(), scene.scene.clone()).await {
        resp.content = description;
    } else {
        resp.code = String::from("500");
    }

    Ok(web::Json(resp))
}
async fn portal_log_user_active(
    activity: web::Json<UserActive>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    println!("activity: {:?}", activity);

    if let Err(e) = log_user_activity(
        activity.id.clone(),
        activity.page.clone(),
        activity.action.clone(),
    )
    .await
    {
        println!("error: {}", e);
        resp.code = String::from("500");
    }

    Ok(web::Json(resp))
}
async fn portal_query_embeddings(data: web::Json<EmbedInfo>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let embed = data.into_inner();
    println!("embed: {:?}", embed);

    match service::ai_town::query_document_embeddings(embed.id, embed.sig, embed.query).await {
        Ok(answer) => {
            resp.content = answer;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_send_pato_instruct(
    data: web::Json<InstructInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let command = data.into_inner();
    println!("command: {:?}", command);

    match service::ai_town::send_pato_instruct(command.id, command.message, command.pro).await {
        Ok(answer) => {
            resp.content = answer;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_create_game_room(
    data: web::Json<PortalRoomInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let room = data.into_inner();
    println!("room: {:?}", room);

    match service::ai_town::create_game_room(room.owner, room.title, room.description, room.town)
        .await
    {
        Ok(room_id) => {
            resp.content = serde_json::to_string(&room_id).unwrap_or_default();
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_ask_for_image_prompt(
    data: web::Json<ImagePromptRequest>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let action = data.into_inner();
    println!("room: {:?}", action);

    match service::ai_town::request_image_prompt(
        action.id,
        action.description,
        action.history,
        action.architecture,
    )
    .await
    {
        Ok(context) => {
            resp.content = context;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_chat_with_image(
    data: web::Json<ImageChatInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let command = data.into_inner();
    println!("command: {:?}", command);

    match service::ai_town::chat_with_image(
        command.id,
        command.message,
        command.pro,
        command.image_url,
    )
    .await
    {
        Ok(answer) => {
            resp.content = answer;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_answer_image(
    data: web::Json<GenImageAnswerInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let command = data.into_inner();
    println!("answer image: {:?}", command);

    match service::ai_town::answer_image(
        command.id,
        command.image_url,
        command.room_id,
        command.level,
        command.input,
        command.prompt,
    )
    .await
    {
        Ok(answer) => {
            resp.content = answer;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_archive_pato_session(
    form: web::Json<ArchiveInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let archive = form.into_inner();
    println!("archive: {:?}", archive);

    if let Err(e) =
        service::ai_town::archive_pato_session(archive.id, archive.session, archive.date).await
    {
        println!("error: {}", e);
        resp.code = String::from("500");
    }

    Ok(web::Json(resp))
}
async fn portal_get_pato_auth_token(id: web::Path<String>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match refresh_pato_auth_token(id.into_inner()).await {
        Ok(token) => {
            resp.content = token;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_get_pato_kol_token(id: web::Path<String>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match query_pato_kol_token(id.into_inner()).await {
        Ok(token) => {
            resp.content = serde_json::to_string(&token).unwrap_or_default();
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_get_pato_by_kol_token(token: web::Path<String>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match query_pato_by_kol_token(token.into_inner()).await {
        Ok(token) => {
            resp.content = serde_json::to_string(&token).unwrap_or_default();
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_get_pato_chat_messages(
    id: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (id, date) = id.into_inner();

    match get_pato_chat_messages(id, date).await {
        Ok(info) => {
            resp.content = serde_json::to_string(&info).unwrap_or_else(|e| {
                println!("error: {}", e);
                resp.code = String::from("500");
                "".to_string()
            });
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_edit_pato_chat_messages(
    data: web::Json<MessagesEditInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match edit_pato_chat_messages(data.id.clone(), data.kol.clone(), data.messages.clone()).await {
        Ok(info) => {
            resp.content = serde_json::to_string(&info).unwrap_or_else(|e| {
                println!("error: {}", e);
                resp.code = String::from("500");
                "".to_string()
            });
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_retrieve_pato_by_name(
    data: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let name = data.into_inner();

    match retrieve_pato_by_name(name).await {
        Ok(info) => {
            resp.content = info;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_get_name_by_id(data: web::Json<Vec<String>>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let ids = data.into_inner();

    match get_name_by_id(ids).await {
        Ok(info) => {
            resp.content = info;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_continue_pato_chat(
    data: web::Json<ContinueTalkInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match continue_pato_chat(
        data.id.clone(),
        data.date.clone(),
        data.session.clone(),
        data.continued,
    )
    .await
    {
        Ok(info) => {
            resp.content = serde_json::to_string(&info).unwrap_or_else(|e| {
                println!("error: {}", e);
                resp.code = String::from("500");
                "".to_string()
            });
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_chat_with_topic(
    data: web::Json<TopicChatInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match topic_chat(data.id.clone(), data.topic.clone(), data.town.clone()).await {
        Ok(_) => {}
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_chat_with_topic_history(
    data: web::Json<TopicChatInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match get_topic_chat_history(data.id.clone(), data.topic.clone(), data.town.clone()).await {
        Ok(his) => {
            resp.content = his;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_get_pro_knowledge(id: web::Path<String>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let id = id.into_inner();

    match get_pro_knowledges(id).await {
        Ok(info) => {
            resp.content = info;
        }
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_pato_self_talk(id: web::Path<String>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let id = id.into_inner();

    match pato_self_talk(id).await {
        Ok(_) => {}
        Err(e) => {
            println!("error: {}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
async fn portal_share_knowledge(
    data: web::Json<ShareKnowledgeInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    if let Err(e) = share_pro_knowledge(
        data.id.clone(),
        data.sig.clone(),
        data.title.clone(),
        data.shared,
    )
    .await
    {
        println!("error: {}", e);
        resp.code = String::from("500");
    }

    Ok(web::Json(resp))
}
async fn portal_add_shared_knowledge(
    data: web::Json<ShareKnowledgeInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    if let Err(e) = add_shared_knowledge(
        data.id.clone(),
        data.sig.clone(),
        data.title.clone(),
        data.owner.clone(),
    )
    .await
    {
        println!("error: {}", e);
        resp.code = String::from("500");
    }

    Ok(web::Json(resp))
}
pub async fn download_generated_file(
    path: web::Path<(String, String, String, String, String, String, String)>,
) -> Result<NamedFile, Error> {
    let (token, session, convert, index, txt_digest, form, digest) = path.into_inner();

    let filepath = format!(
        "{}/download/{}/{}/{}/{}/{}/{}/{}",
        AI_AGENT_DIR, token, session, convert, index, txt_digest, form, digest
    );
    println!("download filename {}", filepath);
    let file = NamedFile::open(filepath)?;
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

pub async fn download_generated_file_with_path(
    info: web::Query<PathInfo>,
) -> Result<NamedFile, Error> {
    let download_file_path = info.absolute_path.clone();

    println!("download filename {}", download_file_path);
    let file = NamedFile::open(download_file_path)?;
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(web::resource("").route(web::get().to(get_index)))
            .service(web::resource("ping").route(web::get().to(pong)))
            .service(
                web::scope("api")
                    .service(web::scope("stats").service(
                        web::resource("user/active").route(web::post().to(portal_log_user_active)),
                    ))
                    .service(
                        web::scope("kol")
                            .service(
                                web::resource("image/description")
                                    .route(web::post().to(portal_image_description)),
                            )
                            .service(
                                web::resource("game/scene/prompt")
                                    .route(web::post().to(portal_ask_for_image_prompt)),
                            )
                            .service(
                                web::resource("chat/image")
                                    .route(web::post().to(portal_chat_with_image)),
                            )
                            .service(
                                web::resource("game/answer/image")
                                    .route(web::post().to(portal_answer_image)),
                            )
                            .service(web::resource("hots").route(web::get().to(portal_town_hots)))
                            .service(
                                web::resource("hot/topics")
                                    .route(web::get().to(portal_town_hot_topics)),
                            )
                            .service(
                                web::resource("create/game")
                                    .route(web::post().to(portal_create_game_room)),
                            )
                            .service(
                                web::resource("become/kol/{id}")
                                    .route(web::get().to(portal_become_kol)),
                            )
                            .service(
                                web::resource("query/staking/{id}")
                                    .route(web::get().to(portal_query_kol_staking)),
                            )
                            .service(
                                web::resource("follow/kol/{follower}/{kol}")
                                    .route(web::get().to(portal_join_kol)),
                            )
                            .service(
                                web::resource("kol/list").route(web::get().to(portal_kol_list)),
                            ),
                    )
                    .service(
                        web::scope("pato")
                            .service(
                                web::resource("tags")
                                    .route(web::get().to(portal_get_predefined_tags)),
                            )
                            .service(
                                web::resource("submit/tags/{id}")
                                    .route(web::post().to(portal_submit_tags)),
                            )
                            .service(
                                web::resource("info/{id}")
                                    .route(web::get().to(portal_get_pato_info)),
                            )
                            .service(
                                web::resource("info/kol/token/{token}")
                                    .route(web::get().to(portal_get_pato_by_kol_token)),
                            )
                            .service(
                                web::resource("messages/{id}/{date}")
                                    .route(web::get().to(portal_get_pato_chat_messages)),
                            )
                            .service(
                                web::resource("archive")
                                    .route(web::post().to(portal_archive_pato_session)),
                            )
                            .service(
                                web::resource("instruct")
                                    .route(web::post().to(portal_send_pato_instruct)),
                            )
                            .service(
                                web::resource("auth/refresh/{id}")
                                    .route(web::get().to(portal_get_pato_auth_token)),
                            )
                            .service(
                                web::resource("kol/auth/query/{id}")
                                    .route(web::get().to(portal_get_pato_kol_token)),
                            )
                            .service(
                                web::resource("edit/messages")
                                    .route(web::post().to(portal_edit_pato_chat_messages)),
                            )
                            .service(
                                web::resource("continue/chat")
                                    .route(web::post().to(portal_continue_pato_chat)),
                            )
                            .service(
                                web::resource("knowledge/all/{id}")
                                    .route(web::get().to(portal_get_pro_knowledge)),
                            )
                            .service(
                                web::resource("share/knowledge")
                                    .route(web::post().to(portal_share_knowledge)),
                            )
                            .service(
                                web::resource("add/shared/knowledge")
                                    .route(web::post().to(portal_add_shared_knowledge)),
                            )
                            .service(
                                web::resource("topic/chat")
                                    .route(web::post().to(portal_chat_with_topic)),
                            )
                            .service(
                                web::resource("retrieve/{name}")
                                    .route(web::get().to(portal_retrieve_pato_by_name)),
                            )
                            .service(
                                web::resource("names").route(web::post().to(portal_get_name_by_id)),
                            )
                            .service(
                                web::resource("self/talk/{id}")
                                    .route(web::get().to(portal_pato_self_talk)),
                            ),
                    )
                    .service(
                        web::resource("topic/chat/history")
                            .route(web::post().to(portal_chat_with_topic_history)),
                    )
                    .service(
                        web::resource("knowledge/shared")
                            .route(web::get().to(portal_shared_knowledges)),
                    )
                    .service(web::resource("login/{id}").route(web::get().to(portal_login)))
                    .service(web::resource("register").route(web::post().to(portal_register)))
                    .service(
                        web::resource("study/knowledge")
                            .route(web::post().to(portal_upload_knowledge)),
                    )
                    .service(
                        web::resource("knowledge/summary/{id}/{sig}")
                            .route(web::get().to(portal_query_summary)),
                    )
                    .service(
                        web::resource("knowledge/query")
                            .route(web::post().to(portal_query_embeddings)),
                    )
                    .service(web::resource("call").route(web::post().to(portal_call_pato))),
            ),
    );
}

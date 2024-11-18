pub mod dao;
pub mod model;
pub mod service;

use crate::service::ai_town::town_register;
use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{
    http::header::ContentType,
    middleware, web, App, HttpResponse, HttpServer, Responder,
};
use candid::CandidType;
use futures::StreamExt;
use futures::TryStreamExt;
use metapower_framework::{
    dao::crawler::download_image, ensure_directory_exists, DataResponse, XFILES_LOCAL_DIR, XFILES_SERVER
};
use serde::{Deserialize, Serialize};
use service::ai_town::request_submit_tags_with_proxy;
use service::{
    ai_town::{
        become_kol, follow_kol, get_name_by_id, get_pato_chat_messages, get_pato_info,
        get_predefined_tags, get_topic_chat_history,
        query_document_summary, query_kol_rooms, query_pato_by_kol_token,
        query_pato_kol_token, refresh_pato_auth_token, retrieve_pato_by_name, submit_tags, town_hot_topics, town_hots, town_login,
    },
    bsc_proxy::{monitor_pab_transfer_event, proxy_contract_call_query_kol_staking, proxy_contract_call_query_kol_ticket}, llm_proxy::{upload_image_save_in_canister, upload_knowledge_save_in_canister},
};
use sha1::Digest;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct TopicChatInfo {
    id: String,
    topic: String,
    session: String,
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    pub name: String,
    pub gender: u8,
    pub personality: String,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum VecQuery {
    Embeddings(Vec<f32>),
}
#[derive(CandidType, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlainDoc {
    pub content: String,
}
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct VecDoc {
    pub content: String,
    pub embeddings: Vec<f32>,
}

#[derive(Deserialize, Debug)]
struct QueryEmbedInfo {
    input: String,
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
    content: String,
}

#[derive(Deserialize)]
pub struct PathInfo {
    absolute_path: String,
    saved_name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("monitor event staking");
    tokio::spawn(monitor_pab_transfer_event());

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
async fn portal_become_kol(info: web::Path<(String,String)>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (id, from) = info.into_inner();

    match become_kol(id, from).await {
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

    let from = info.into_inner();

    match proxy_contract_call_query_kol_staking(from).await {
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
async fn portal_query_kol_ticket(info: web::Path<String>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: "0".to_string(),
        code: String::from("404"),
    };

    let from = info.into_inner();

    match proxy_contract_call_query_kol_ticket(from).await {
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
async fn portal_join_kol(info: web::Path<(String, String, String)>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (follower, kol, from) = info.into_inner();

    if let Err(e) = follow_kol(kol, follower, from).await {
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
    let mut has_file_uploaded = false;

    // Iterate over multipart/form-data
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.unwrap().get_name().unwrap_or("");
        // Get the filename from the content-disposition header
        if filename.is_empty() {
            filename = content_disposition
                .unwrap()
                .get_filename()
                .unwrap_or("knowledge-file")
                .to_string();
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
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    message_json.extend(data.iter().map(|&b| b as char));
                }
            }
            _ => {}
        }
    }


    let id: String = message_json;

    if has_file_uploaded {
        let mut session = format!("{:x}", hasher.finalize());
        let filename_saved = "content.txt".to_string();
        
        let header: [u8; 4] = file_bytes.as_slice()[0..4].try_into().unwrap_or_default();
        if &header == b"%PDF" {
            println!("pdf file detected");
            let content = pdf_extract::extract_text_from_mem(&file_bytes).unwrap_or_default();
            let mut hasher = sha1::Sha1::new();
            hasher.update(&content);
            session = format!("{:x}", hasher.finalize());
        }
        
        match upload_knowledge_save_in_canister(session, id,  filename_saved, file_bytes).await
        {
            Ok(url) => {
                resp.content = url;
            }
            Err(e) => {
                resp.content = format!("{}", e);
                resp.code = String::from("500");
            }
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

async fn portal_query_summary(
    data: web::Path<(String, String, String)>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (id, sig, file_name) = data.into_inner();

    match query_document_summary(id, sig, file_name).await {
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
    id: web::Path<(String, String)>,
    tags: web::Json<Vec<String>>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (id, session) = id.into_inner();
    match submit_tags(id, session, tags.into_inner()).await {
        Ok(avatar_url) => resp.content = avatar_url,
        Err(e) => {
            resp.code = String::from("500");
            resp.content = e.to_string();
        }
    }

    Ok(web::Json(resp))
}

async fn proxy_submit_tags(
    data: web::Path<(String, String)>,
    tags: web::Json<Vec<String>>,
) -> actix_web::Result<impl Responder> {
    let resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (id, session) = data.into_inner();

    match request_submit_tags_with_proxy(id, session, tags.into_inner()).await {
        Ok(_) => (),
        Err(e) => {
            println!("request_submit_tags_with_proxy error: {}", e);
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

async fn portal_upload_image(
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

    let id: String = message_json;

    if has_file_uploaded {
        let session = format!("{:x}", hasher.finalize());
        match upload_image_save_in_canister(session, id, file_bytes).await
        {
            Ok(url) => {
                resp.content = url;
            }
            Err(e) => {
                resp.content = format!("{}", e);
                resp.code = String::from("500");
            }
        }
    }

    Ok(web::Json(resp))
}
async fn portal_query_embeddings(data: web::Json<QueryEmbedInfo>) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let embed = data.into_inner();
    println!("embed: {:?}", embed);

    match service::ai_town::query_document_embeddings(embed.input).await {
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

    match service::ai_town::archive_pato_session(archive.id, archive.session, archive.content).await
    {
        Ok(file_url) => {
            resp.content = file_url;
        }
        Err(e) => {
            resp.content = format!("{}", e);
            resp.code = String::from("500");
        }
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
async fn portal_get_pato_by_kol_token(
    token: web::Path<String>,
) -> actix_web::Result<impl Responder> {
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
async fn portal_chat_with_topic_history(
    data: web::Json<TopicChatInfo>,
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    match get_topic_chat_history(data.id.clone(), data.session.clone()).await {
        Ok(his) => {
            resp.content = his;
        }
        Err(e) => {
            resp.content = format!("{}", e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}
pub async fn download_generated_file_with_path(
    id: web::Path<String>, path: web::Json<PathInfo>,
)  -> actix_web::Result<impl Responder>  {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let id = id.into_inner();

    let _ = ensure_directory_exists(&format!("{}/ai/{}", XFILES_LOCAL_DIR, id));
    let saved_local_file = format!("{}/ai/{}/{}", XFILES_LOCAL_DIR, id, path.saved_name);

    println!("download ai resource {:?}, saved to {}", path.absolute_path, saved_local_file);

    if Path::new(&saved_local_file).exists() {
        resp.content = format!("{}/ai/{}/{}", XFILES_SERVER, id, path.saved_name);
        println!("file already exists, return link: {}", resp.content);
        return Ok(web::Json(resp));
    }

    let xfiles_link = format!("{}/ai/{}/{}", XFILES_SERVER, id, path.saved_name);
    match download_image(&path.absolute_path, &saved_local_file).await {
        Ok(_) => {
            resp.content = xfiles_link;
        }
        Err(e) => {
            resp.code = "500".to_string();
            println!("download ai resource error: {}", e);
        }
    }
    
    Ok(web::Json(resp))
}

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(web::resource("").route(web::get().to(get_index)))
            .service(web::resource("ping").route(web::get().to(pong)))
            .service(
                web::scope("api")
                    .service(web::resource("download/ai/resource/{id}").route(web::post().to(download_generated_file_with_path)))
                    .service(
                        web::scope("kol")
                            .service(
                                web::resource("hot/topics")
                                    .route(web::get().to(portal_town_hot_topics)),
                            )
                            .service(
                                web::resource("become/kol/{id}/{from}")
                                    .route(web::get().to(portal_become_kol)),
                            )
                            .service(
                                web::resource("query/staking/{id}")
                                    .route(web::get().to(portal_query_kol_staking)),
                            )
                            .service(
                                web::resource("query/ticket/{id}")
                                    .route(web::get().to(portal_query_kol_ticket)),
                            )
                            .service(
                                web::resource("follow/kol/{follower}/{kol}/{from}")
                                    .route(web::get().to(portal_join_kol)),
                            )
                            .service(
                                web::resource("kol/list").route(web::get().to(portal_kol_list)),
                            ),
                    )
                    .service(
                        web::scope("pato")
                            .service(web::resource("hots").route(web::get().to(portal_town_hots)))
                            .service(
                                web::resource("tags")
                                    .route(web::get().to(portal_get_predefined_tags)),
                            )
                            .service(
                                web::resource("upload/image")
                                    .route(web::post().to(portal_upload_image)),
                            )
                            .service(
                                web::resource("submit/tags/{id}/{session}")
                                    .route(web::post().to(portal_submit_tags)),
                            )
                            .service(
                                web::resource("proxy/submit/tags/{id}/{session}")
                                    .route(web::post().to(proxy_submit_tags)),
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
                                web::resource("auth/refresh/{id}")
                                    .route(web::get().to(portal_get_pato_auth_token)),
                            )
                            .service(
                                web::resource("kol/auth/query/{id}")
                                    .route(web::get().to(portal_get_pato_kol_token)),
                            )
                            .service(
                                web::resource("retrieve/{name}")
                                    .route(web::get().to(portal_retrieve_pato_by_name)),
                            )
                            .service(
                                web::resource("names").route(web::post().to(portal_get_name_by_id)),
                            )
                    )
                    .service(
                        web::resource("topic/chat/history")
                            .route(web::post().to(portal_chat_with_topic_history)),
                    )
                    .service(web::resource("login/{id}").route(web::get().to(portal_login)))
                    .service(web::resource("register").route(web::post().to(portal_register)))
                    .service(
                        web::resource("upload/knowledge")
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
            ),
    );
}

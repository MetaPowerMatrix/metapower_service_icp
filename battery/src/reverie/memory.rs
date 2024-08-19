use metapower_framework::service::llmchat_model::llmchat_grpc::chat_svc_client::ChatSvcClient;
use metapower_framework::service::llmchat_model::llmchat_grpc::SomeDocs;
use metapower_framework::service::metapowermatrix_agent_mod::agent_grpc::meta_power_matrix_agent_svc_client::MetaPowerMatrixAgentSvcClient;
use metapower_framework::service::metapowermatrix_agent_mod::agent_grpc::SimpleRequest;
use metapower_framework::{ensure_directory_exists, get_now_date_str, read_and_writeback_json_file, ChatMessage, AGENT_GRPC_REST_SERVER, LLMCHAT_GRPC_REST_SERVER};
use tempfile::NamedTempFile;
use std::path::{Path, PathBuf};
use std::{fs::OpenOptions, io::Write};
use std::fs::{self, File};
use std::time::SystemTime;
use std::env;
use metapower_framework::{log, AI_PATO_DIR};

pub fn save_kol_chat_message(initial: String, kol: String, chat_messages: &mut Vec<ChatMessage>, append: bool) 
{
    let chat_session_message_path = format!(
        "{}/{}/db/kol/{}",
        AI_PATO_DIR,
        initial,
        kol,
    );
    let _ = ensure_directory_exists(&chat_session_message_path);
    let message_file = chat_session_message_path + "/message.json";
    if append {
        if let Err(e) = read_and_writeback_json_file(&message_file, chat_messages){
            log!("read_and_writeback_json_file error: {}", e);
        }
    }else if let Ok(file) = OpenOptions::new().write(true).truncate(true).create(true).open(message_file.clone()){
        if let Err(e) = serde_json::to_writer(file, chat_messages){
            log!("save kol chat message error: {}", e);
        }
    }
}

pub fn get_chat_his_with_kol(kol: String, sender_id: String, sender_name: String, receiver_name: String) -> (Vec<String>, String){
    let chat_session_path = format!(
        "{}/{}/db/kol/{}/message.json",
        AI_PATO_DIR,
        sender_id.clone(),
        kol,
    );
    log!("chat_session_path: {}", chat_session_path);

    let file = File::open(chat_session_path);
    if let Ok(file) = file {
        match serde_json::from_reader::<File, Vec<ChatMessage>>(file){
            Ok(messages) => {
                // messages.sort_by(|m1, m2| m1.created_at.cmp(&m2.created_at));
                let his  = messages.iter().map(|m| {
                    if sender_id == m.sender {
                        format!("{}: {} \n {}: {}", sender_name, m.question, receiver_name, m.answer)
                    }else{
                        format!("{}: {} \n {}: {}", receiver_name, m.question, sender_name, m.answer)
                    }
                }).collect();
                if !messages.is_empty(){
                    if messages.last().unwrap().receiver.is_empty(){
                        return (his, messages.last().unwrap().sender.clone());
                    }else {
                        return (his, "".to_string());
                    }
                }
            }
            Err(e) => {
                log!("read chat messages from file error: {}", e);
            }
        }
    }

    (vec![], String::default())
}

pub fn get_chat_his_by_session(session: String, sender_id: String, sender_name: String, receiver_name: String) -> (Vec<String>, String){
    let chat_session_path = format!(
        "{}/{}/db/{}/{}/message.json",
        AI_PATO_DIR,
        sender_id,
        get_now_date_str(),
        session,
    );
    log!("chat_session_path: {}", chat_session_path);

    let file = File::open(chat_session_path);
    if let Ok(file) = file {
        match serde_json::from_reader::<File, Vec<ChatMessage>>(file){
            Ok(messages) => {
                // messages.sort_by(|m1, m2| m1.created_at.cmp(&m2.created_at));
                let his  = messages.iter().map(|m| {
                    if sender_id == m.sender {
                        format!("{}: {} \n {}: {}", sender_name, m.question, receiver_name, m.answer)
                    }else{
                        format!("{}: {} \n {}: {}", receiver_name, m.question, sender_name, m.answer)
                    }
                }).collect();
                if !messages.is_empty(){
                    if messages.last().unwrap().receiver.is_empty(){
                        return (his, messages.last().unwrap().sender.clone());
                    }else {
                        return (his, "".to_string());
                    }
                }
            }
            Err(e) => {
                log!("read chat messages from file error: {}", e);
            }
        }
    }

    (vec![], String::default())
}

pub async fn get_pato_knowledges(id: String) -> Vec<String> {
    let mut professionals:Vec<String> = vec![];
    if let Ok(mut agent_client) = MetaPowerMatrixAgentSvcClient::connect(AGENT_GRPC_REST_SERVER).await
    {
        let pro_request = tonic::Request::new(SimpleRequest { id });
        if let Ok(pro_resp) = agent_client.request_pato_knowledges(pro_request).await {
            for pro in pro_resp.get_ref().professionals.iter(){
                let knowledges = pro.split(',').map(|k| k.to_owned()) .collect::<Vec<String>>();
                professionals.extend(knowledges);
            }
        }
    }

    professionals
}
pub fn find_chat_session_dirs(id: String, date: String) -> Vec<PathBuf> {
    let chat_session_path = format!(
        "{}/{}/db/{}",
        AI_PATO_DIR,
        id,
        date,
    );

    let mut db_dirs = Vec::new();
    let path = Path::new(chat_session_path.as_str());
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                db_dirs.push(path);
            }
        }
    }
    db_dirs
}
pub fn get_kol_messages(id: String, kol: String) -> Vec<ChatMessage> {
    let mut session_messages: Vec<ChatMessage> = vec![];
    let chat_session_path = format!(
        "{}/{}/db/kol/{}",
        AI_PATO_DIR,
        id,
        kol,
    );
    log!("kol_chat_session_path: {:?}", chat_session_path);
    let message_file = chat_session_path + "/message.json";
    
    if let Ok(file) = File::open(message_file.clone()){
        match serde_json::from_reader::<File, Vec<ChatMessage>>(file){
            Ok(messages) =>{
                session_messages = messages;
            }
            Err(e) => {
                log!("read kol chat messages from file error: {}", e);
            }
        }
    }else{
        log!("error read {:?}", message_file);
    }

    session_messages
}
pub async fn get_kol_messages_summary(summary_content: String) -> Option<String> {
    if let Ok(mut llm_client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
        if let Ok(mut temp_file) = NamedTempFile::new(){
            if temp_file.write_all(summary_content.as_bytes()).is_ok(){
                let _ = temp_file.flush();
                let llmrequest = tonic::Request::new(SomeDocs {
                    doc_file: temp_file.path().to_str().unwrap().to_string(),
                    doc_format: "txt".to_string(),
                });
                log!("llmrequest for kol summary: {:?}", llmrequest);
                match llm_client.got_documents_summary(llmrequest).await{
                    Ok(sum_resp) => {
                        let summary = sum_resp.get_ref().summary.clone();
                        return Some(summary);    
                    }
                    Err(e) => {
                        log!("got_documents_summary from LLM error: {}", e);
                    }
                }
            }else{
                log!("write temp file error");
            }
        }else{
            log!("create temp file error");
        }
    }

    None
}

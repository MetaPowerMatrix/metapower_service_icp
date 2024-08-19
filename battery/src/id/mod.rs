pub mod identity;

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;
use std::{env, fs, io};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use candid::Decode;
use metapower_framework::dao::crawler::download_image;
use metapower_framework::icp::{call_update_method, FollowKolRequest, KolRegistrationRequest, NameRequest, NameResponse, SnRequest, SnResponse, AGENT_SMITH_CANISTER};
use metapower_framework::mqtt::publish::publish_battery_actions;
use metapower_framework::service::llmchat_model::llmchat_grpc::{BestTalkRequest, BetterTalkRequest, CharacterGenRequest, DocsRequest, ImageChatRequest as LLMImageChatRequest, ImageDescriptionRequest, ImageGenRequest, ImagePromptRequest, QueryEmbeddingsRequest, QuestionRequest, SomeDocs, TextToSpeechRequest};
use metapower_framework::service::metapowermatrix_battery_mod::battery_grpc::meta_power_matrix_battery_svc_client::MetaPowerMatrixBatterySvcClient;
use metapower_framework::service::metapowermatrix_battery_mod::battery_grpc::{ArchiveMessageRequest, BecomeKolRequest, CallRequest, ContinueLiveRequest, ContinueRequest, CreateLiveSessionRequest, DocumentSummaryRequest, DocumentSummaryResponse, EditeReqeust, EventResponse, GameAnswerRequest, GameAnswerResponse, GetMessageRequest, GetMessageResponse, GetProMessageRequest, GoTownRequest, ImageAnswerRequest, ImageChatRequest, ImageChatResponse, ImageContextRequest, ImageContextResponse, ImageGenPromptRequest, InstructRequest, InstructResponse, JoinKolRoomRequest, JoinRoomRequest, JoinRoomResponse, KnowLedgeInfo, KnowLedgesRequest, KnowLedgesResponse, OpenLiveRequest, OpenLiveResponse, PatoIssEditRequest, PatoIssResponse, PatoNameResponse, QueryEmbeddingRequest, QueryEmbeddingResponse, RestoreLiveRequest, RevealAnswerRequest, RevealAnswerResponse, SceneRequest, SceneResponse, ShareKnowLedgesRequest, SubmitTagsRequest, SubmitTagsResponse, SummaryAndEmbeddingRequest, SummaryAndEmbeddingResponse, SvcImageDescriptionRequest, SvcImageDescriptionResponse
};
use metapower_framework::{ensure_directory_exists, get_event_subjects, get_now_secs, log, mqtt, read_and_writeback_json_file, ChatMessage, SessionMessages, AGENT_GRPC_REST_SERVER, AI_MATRIX_DIR, BATTERY_GRPC_REST_SERVER, BATTERY_GRPC_SERVER_PORT_START, TICK, XFILES_LOCAL_DIR, XFILES_SERVER};
use metapower_framework::{
    get_now_date_str, model::Battery, service::{
        llmchat_model::llmchat_grpc::{chat_svc_client::ChatSvcClient, EventTopic}, metapowermatrix_battery_mod::battery_grpc::{
            meta_power_matrix_battery_svc_server::MetaPowerMatrixBatterySvc, EmptyRequest, EventRequest, MessageRequest, TalkResponse
        }
    }, AI_PATO_DIR, LLMCHAT_GRPC_REST_SERVER
};
use tempfile::NamedTempFile;
use tokio::time::sleep;
use tonic::Response;
use rand::prelude::SliceRandom;

use crate::id::identity::{ask_pato_knowledges, ask_pato_name, get_pato_name};
use crate::reverie::generate_prompt;
use crate::reverie::memory::{
    find_chat_session_dirs, get_kol_messages, get_kol_messages_summary, get_pato_knowledges,
    save_kol_chat_message,
};

const MAX_SUBJECT_LEN: i32 = 22;

pub trait BatteryInstance {
    fn new_instance(location: String) -> Battery;
    fn got_salary_daily(&self);
}

impl BatteryInstance for Battery {
    fn new_instance(location: String) -> Battery {
        Battery {
            id: uuid::Uuid::new_v4().to_string(),
            wallets: vec![],
            connections: vec![],
        }
    }

    fn got_salary_daily(&self) {}
}

#[derive(Debug, Clone)]
pub struct MetaPowerMatrixBatteryService {
    id: String,
}

impl MetaPowerMatrixBatteryService {
    pub fn new(id: String) -> Self {
        MetaPowerMatrixBatteryService { id }
    }

    async fn get_session_messages_summary(
        &self,
        summary_file: PathBuf,
        summary_content: String,
    ) -> Option<String> {
        // log!("summary_file: {:?}", summary_file);
        if summary_file.exists() && summary_file.is_file() {
            let mut buffer = Vec::new();
            if let Ok(mut file) = File::open(summary_file) {
                match file.read_to_end(&mut buffer) {
                    Ok(_) => {
                        if let Ok(content) = String::from_utf8(buffer) {
                            return Some(content);
                        }
                    }
                    Err(e) => {
                        log!("read summary file error: {}", e);
                    }
                }
            }
        } else if let Ok(mut llm_client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            if let Ok(mut temp_file) = NamedTempFile::new() {
                if temp_file.write_all(summary_content.as_bytes()).is_ok() {
                    let _ = temp_file.flush();
                    let llmrequest = tonic::Request::new(SomeDocs {
                        doc_file: temp_file.path().to_str().unwrap().to_string(),
                        doc_format: "txt".to_string(),
                    });
                    log!("llmrequest: {:?}", llmrequest);
                    match llm_client.got_documents_summary(llmrequest).await {
                        Ok(sum_resp) => {
                            // log!("sum_resp: {:?}", sum_resp.get_ref().summary.clone());
                            let summary = sum_resp.get_ref().summary.clone();
                            if let Ok(mut file) = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(summary_file.clone())
                            {
                                let _ = write!(file, "{}", summary);
                            }
                            let _ = fs::remove_file(temp_file.path());
                            return Some(summary);
                        }
                        Err(e) => {
                            log!("got_documents_summary from LLM error: {}", e);
                        }
                    }
                } else {
                    log!("write temp file error");
                }
            } else {
                log!("create temp file error");
            }
        }

        None
    }
    fn notify_gamers(&self, room_id: String, topic: String, message: String) {
        let game_room_path = format!("{}/{}/db/game_room/{}", AI_PATO_DIR, self.id, room_id);
        if let Ok(file) = OpenOptions::new()
            .read(true)
            .open(format!("{}/gamer.txt", game_room_path))
        {
            let reader = io::BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(gamer_id) = line
                    .split('#')
                    .map(|g| g.to_owned())
                    .collect::<Vec<String>>()
                    .first()
                {
                    let message = format!("{}: {}", topic, message);
                    let _ = publish_battery_actions(room_id.clone() + "/" + gamer_id, message);
                }
            }
        }
    }
    fn continue_chat(&self, session: String, date: String, continued: bool) {
        let mut continue_message = "byebye".to_string();
        if continued {
            continue_message = "真的很有收获呢，我们继续聊吧！".to_string();
        }
        let chat_message = ChatMessage {
            created_at: get_now_secs() as i64,
            session: session.clone(),
            place: String::default(),
            sender: self.id.clone(),
            receiver: String::default(),
            question: continue_message,
            answer: String::default(),
            subject: String::default(),
            sender_role: "user".to_string(),
        };
        let chat_session_message_file = format!(
            "{}/{}/db/{}/{}/message.json",
            AI_PATO_DIR, self.id, date, session,
        );
        if let Err(e) =
            read_and_writeback_json_file(&chat_session_message_file, &mut vec![chat_message])
        {
            log!("append continue message error: {}", e);
        }
    }
    async fn talk(
        &self,
        request: tonic::Request<MessageRequest>,
    ) -> std::result::Result<tonic::Response<TalkResponse>, tonic::Status> {
        let chat_content = request.into_inner();
        let subject = chat_content.subject.clone();
        let input = chat_content.message.clone();
        let prompt = chat_content.prompt.clone();
        let db_path = format!("{}/{}/db/knowledge_chromadb", AI_PATO_DIR, self.id);
        let knowledges = get_pato_knowledges(self.id.clone()).await;
        let collection = if knowledges.is_empty() {
            "general".to_string()
        } else {
            knowledges[0].clone()
        };

        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let chat_request = tonic::Request::new(BestTalkRequest {
                question: input,
                collection_name: collection,
                db_path,
                prompt,
            });
            // println!("chat_request: {:?}", chat_request);
            match client.talk_best(chat_request).await {
                Ok(answer) => {
                    // log!("- I({}) said: {}", self.id, answer.get_ref().answer.clone());
                    let response = TalkResponse {
                        speaker: self.id.clone(),
                        message: answer.get_ref().answer.clone(),
                    };
                    return Ok(Response::new(response));
                }
                Err(e) => {
                    log!("My AI {} is something wrong: {}", self.id, e);
                }
            }
        }

        Err(tonic::Status::unavailable("um, I didn't hear clearly"))
    }

    async fn create_event(
        &self,
        request: tonic::Request<EventRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        let mut subject = String::default();
        let subjects = get_event_subjects();
        let topic = request.get_ref().topic.clone().replace('\n', " and ");
        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let topic_subject_request = tonic::Request::new(EventTopic {
                topic: topic.clone(),
                subjects: subjects.iter().map(|s| s.to_string()).collect(),
            });
            if let Ok(response) = client.got_topic_subject(topic_subject_request).await {
                subject = response.get_ref().subject.clone();
                subject = subject
                    .trim()
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != ' ')
                    .to_string();
                if subject.len() > MAX_SUBJECT_LEN as usize {
                    subject = subjects
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .to_string();
                }
            }
        }
        println!("create event: {}#{}", request.get_ref().topic, subject);
        let eventfilename = format!(
            "{}/{}/db/event_{}.txt",
            AI_PATO_DIR,
            self.id,
            get_now_date_str()
        );
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(eventfilename)?;

        writeln!(file, "{}#{}", topic, subject)?;

        let response = EmptyRequest {};

        Ok(Response::new(response))
    }
    async fn get_chat_messages(
        &self,
        request: tonic::Request<GetMessageRequest>,
    ) -> std::result::Result<tonic::Response<GetMessageResponse>, tonic::Status> {
        let mut session_messages: Vec<SessionMessages> = vec![];
        let chat_sessions = find_chat_session_dirs(self.id.clone(), request.get_ref().date.clone());
        // log!("chat_sessions: {:?}", chat_sessions);
        for session_dir in chat_sessions {
            let message_file = session_dir.join("message.json");
            let summary_file = session_dir.join("summary.txt");
            let session = session_dir
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let mut ids: Vec<String> = vec![];

            if !message_file.exists() {
                continue;
            }

            if let Ok(file) = File::open(message_file.clone()) {
                match serde_json::from_reader::<File, Vec<ChatMessage>>(file) {
                    Ok(mut messages) => {
                        for message in messages.iter() {
                            if ids.contains(&message.sender) {
                                continue;
                            }
                            if ids.contains(&message.receiver) {
                                continue;
                            }
                            ids.push(message.sender.clone());
                            ids.push(message.receiver.clone());
                        }
                        let req = NameRequest {
                            id: ids.clone(),
                        };
                        match call_update_method(AGENT_SMITH_CANISTER, "request_pato_by_ids", req).await{
                            Ok(result) => {
                                let name_pro_resp = Decode!(result.as_slice(), NameResponse).unwrap_or_default();
                                let resp = name_pro_resp.name_pros;
                                // println!("resp: {:?}", resp);
                                for message in messages.iter_mut() {
                                    for name_pro in resp.iter() {
                                        // println!("name_pro: {:?}, message: {}-{}", name_pro, message.sender, message.receiver);
                                        if name_pro.id == message.sender {
                                            message.sender = format!(
                                                "{}({})",
                                                name_pro.name,
                                                name_pro.pros.join(",")
                                            );
                                        }
                                        if name_pro.id == message.receiver {
                                            message.receiver = format!(
                                                "{}({})",
                                                name_pro.name,
                                                name_pro.pros.join(",")
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log!("request_pato_name_and_pro error: {}", e);
                            }
                        }
                        let his: Vec<String> = messages
                            .iter()
                            .map(|m| {
                                let mut receiver = m.receiver.clone();
                                if m.sender == m.receiver {
                                    receiver = m.receiver.clone() + "#2";
                                }
                                format!(
                                    "{}: {} \n {}: {}",
                                    m.sender, m.question, receiver, m.answer
                                )
                            })
                            .collect();
                        let summary = self
                            .get_session_messages_summary(summary_file.clone(), his.join("\n"))
                            .await;
                        // log!("summary: {:?}", summary);
                        let session_message = SessionMessages {
                            session,
                            summary: summary.unwrap_or_default(),
                            messages,
                        };
                        session_messages.push(session_message);
                    }
                    Err(e) => {
                        log!("read chat messages from file error: {}", e);
                    }
                }
            } else {
                log!("error read {:?}", message_file);
            }
        }
        let content = serde_json::to_string(&session_messages).unwrap_or_default();

        let response = GetMessageResponse { content };

        Ok(Response::new(response))
    }

    async fn request_pato_event(
        &self,
        _request: tonic::Request<EmptyRequest>,
    ) -> std::result::Result<tonic::Response<EventResponse>, tonic::Status> {
        let eventfilename = format!(
            "{}/{}/db/event_{}.txt",
            AI_PATO_DIR,
            self.id,
            get_now_date_str()
        );
        let mut lines: Vec<String> = vec![];
        if let Ok(file) = File::open(eventfilename) {
            let reader = io::BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                lines.push(line);
            }
        }
        let response = EventResponse { events: lines };

        Ok(Response::new(response))
    }

    async fn request_pato_name(
        &self,
        _request: tonic::Request<EmptyRequest>,
    ) -> std::result::Result<tonic::Response<PatoNameResponse>, tonic::Status> {
        let mut name = String::default();
        let name_file = format!("{}/{}/db/name.txt", AI_PATO_DIR, self.id);
        if let Ok(file) = File::open(name_file) {
            let reader = BufReader::new(file);
            if let Some(Ok(last_line)) = reader.lines().last() {
                name = last_line;
            }
        }

        let response = PatoNameResponse { name };

        Ok(Response::new(response))
    }

    async fn request_pato_iss(
        &self,
        _request: tonic::Request<EmptyRequest>,
    ) -> std::result::Result<tonic::Response<PatoIssResponse>, tonic::Status> {
        let mut iss = String::new();
        if let Ok(mut file) = OpenOptions::new()
            .read(true)
            .open(format!("{}/{}/db/character.txt", AI_PATO_DIR, self.id))
        {
            file.read_to_string(&mut iss)?;
        }
        let response = PatoIssResponse { iss };

        Ok(Response::new(response))
    }

    async fn change_pato_iss(
        &self,
        request: tonic::Request<PatoIssEditRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/{}/db/character.txt", AI_PATO_DIR, self.id))
        {
            writeln!(file, "{}", request.get_ref().iss)?;
        }

        Ok(Response::new(EmptyRequest {}))
    }

    async fn request_pato_call(
        &self,
        request: tonic::Request<CallRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        let eventfilename = format!("{}/{}/db/call.txt", AI_PATO_DIR, self.id);
        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(eventfilename)
        {
            Ok(mut file) => {
                writeln!(
                    file,
                    "{}#{}#waiting",
                    request.get_ref().id,
                    request.get_ref().topic
                )?;
            }
            Err(e) => {
                log!("request_pato_call write file error: {}", e);
            }
        }

        Ok(Response::new(EmptyRequest {}))
    }

    async fn archive_chat_messages(
        &self,
        request: tonic::Request<ArchiveMessageRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        let session = request.get_ref().session.clone();
        let date = request.get_ref().date.clone();

        let chat_session_path = format!("{}/{}/db/{}/{}", AI_PATO_DIR, self.id, date, session);

        let archive_session_path = format!(
            "{}/{}/db/{}/{}/archive",
            AI_PATO_DIR, self.id, date, session
        );

        let _ = ensure_directory_exists(&archive_session_path);
        // Copy the file to the new location
        fs::copy(
            chat_session_path.clone() + "/message.json",
            archive_session_path + "/message.json",
        )?;

        // Delete the original file
        let _ = fs::remove_file(chat_session_path + "/message.json");

        Ok(Response::new(EmptyRequest {}))
    }
    async fn request_instruct(
        &self,
        request: tonic::Request<InstructRequest>,
    ) -> std::result::Result<tonic::Response<InstructResponse>, tonic::Status> {
        let mut response = InstructResponse {
            answer: String::default(),
        };
        let mut curr_input: Vec<String> = vec![];
        let kol_id = request.get_ref().kol.clone();

        let kol_name = ask_pato_name(kol_id.clone()).await.unwrap_or_default();
        let my_name = get_pato_name(self.id.clone()).unwrap_or_default();
        let session_messages: Vec<ChatMessage> = get_kol_messages(
            request.get_ref().reply_to.clone(),
            request.get_ref().kol.clone(),
        );
        let raw_messages = session_messages
            .iter()
            .map(|m| my_name.clone() + ":" + &m.question + "\n" + &kol_name + ":" + &m.answer)
            .collect::<Vec<String>>();
        let summary_content = raw_messages.join("\n");
        let summary = get_kol_messages_summary(summary_content.clone())
            .await
            .unwrap_or_default();
        let filtered_messages = raw_messages
            .iter()
            .filter(|m| m.len() < 800)
            .map(|m| m.to_owned())
            .collect::<Vec<String>>();

        curr_input.push(my_name.clone()); //0
        curr_input.push(kol_name.clone()); //1
        curr_input.push(my_name.clone()); //2
        curr_input.push(kol_name.clone()); //3
        curr_input.push(summary); //4
        curr_input.push(filtered_messages.join("\n")); //5
        curr_input.push(kol_name.clone()); //6
        curr_input.push(my_name.clone()); //7
        curr_input.push(request.get_ref().message.clone()); //8
        curr_input.push(kol_name.clone()); //9
        curr_input.push(my_name.clone()); //10
        curr_input.push(kol_name.clone()); //11
        curr_input.push(kol_name.clone()); //12

        let prompt_lib_file = format!("{}/template/plan/agent_chat_pro.txt", AI_MATRIX_DIR);
        let prompt = generate_prompt(curr_input, &prompt_lib_file);
        log!("kol_chat_prompt: {}", prompt);

        let knowledges = ask_pato_knowledges(kol_id.clone()).await;
        let filtered_knowledges = knowledges
            .iter()
            .filter(|k| k.owner == kol_id)
            .map(|k| k.to_owned())
            .collect::<Vec<KnowLedgeInfo>>();
        println!("kol_chat_knowledges: {:?}", filtered_knowledges);
        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let chat_request = tonic::Request::new(BetterTalkRequest {
                question: request.get_ref().message.clone(),
                prompt,
                collection_name: filtered_knowledges
                    .iter()
                    .map(|k| "sig".to_string() + &k.sig)
                    .collect::<Vec<String>>(),
                db_path: format!("{}/{}/db/knowledge_chromadb", AI_PATO_DIR, kol_id),
            });
            // println!("chat_request: {:?}", chat_request);
            match client.talk_better(chat_request).await {
                Ok(answer) => {
                    response.answer = answer.get_ref().answer.clone();
                    let _ = publish_battery_actions(
                        request.get_ref().reply_to.clone() + "/instruct",
                        answer.get_ref().answer.clone(),
                    );

                    let message = ChatMessage {
                        created_at: get_now_secs() as i64,
                        session: String::default(),
                        place: "online".to_string(),
                        sender: request.get_ref().reply_to.clone(),
                        receiver: kol_id.clone(),
                        question: request.get_ref().message.clone(),
                        answer: response.answer.clone(),
                        sender_role: "user".to_string(),
                        subject: "consultant".to_string(),
                    };
                    save_kol_chat_message(
                        request.get_ref().reply_to.clone(),
                        kol_id.clone(),
                        &mut vec![message],
                        true,
                    );

                    let tts_request = tonic::Request::new(TextToSpeechRequest {
                        text: answer.get_ref().answer.clone(),
                    });
                    match client.text_to_speech(tts_request).await {
                        Ok(audio_file) => {
                            let audio_url =
                                XFILES_SERVER.to_string() + "/" + &audio_file.get_ref().audio_url;
                            let _ = publish_battery_actions(
                                request.get_ref().reply_to.clone() + "/instruct/voice",
                                audio_url,
                            );
                        }
                        Err(e) => {
                            log!("Instuct Text to speech is something wrong: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log!("Instruct AI is something wrong: {}", e);
                }
            }
        }
        Ok(Response::new(response))
    }

    async fn get_pro_chat_messages(
        &self,
        request: tonic::Request<GetProMessageRequest>,
    ) -> std::result::Result<tonic::Response<GetMessageResponse>, tonic::Status> {
        let session_messages: Vec<ChatMessage> = get_kol_messages(
            request.get_ref().id.clone(),
            request.get_ref().proid.clone(),
        );
        let content = serde_json::to_string(&session_messages).unwrap_or_default();
        let response = GetMessageResponse { content };

        Ok(Response::new(response))
    }

    async fn request_continue_chat(
        &self,
        request: tonic::Request<ContinueRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        log!("set to continue chat {}", request.get_ref().continued);
        self.continue_chat(
            request.get_ref().session.clone(),
            request.get_ref().date.clone(),
            request.get_ref().continued,
        );
        let response = EmptyRequest {};

        Ok(Response::new(response))
    }

    async fn request_edit_messages(
        &self,
        request: tonic::Request<EditeReqeust>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        match serde_json::from_str::<Vec<ChatMessage>>(&request.get_ref().messages) {
            Ok(mut messages) => {
                save_kol_chat_message(
                    request.get_ref().initial.clone(),
                    request.get_ref().kol.clone(),
                    &mut messages,
                    false,
                );
            }
            Err(e) => {
                log!("edited messages format error: {}", e);
            }
        }

        let response = EmptyRequest {};

        Ok(Response::new(response))
    }

    async fn request_go_town(
        &self,
        request: tonic::Request<GoTownRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        let subject = request.get_ref().town.clone();
        let mut topic = request.get_ref().topic.clone();
        topic = if topic.is_empty() {
            match subject.as_str() {
                "music" => "聊聊音乐吧".to_string(),
                "invest" => "聊聊投资吧".to_string(),
                "literature" => "聊聊文学吧".to_string(),
                "web3" => "聊聊区块链吧".to_string(),
                "science" => "聊聊科学吧".to_string(),
                _ => "随便聊聊吧".to_string(),
            }
        } else {
            topic
        };
        println!("go town: {}#{}", subject, topic);
        let eventfilename = format!(
            "{}/{}/db/event_{}.txt",
            AI_PATO_DIR,
            self.id,
            get_now_date_str()
        );
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(eventfilename)?;

        writeln!(file, "{}#{}", topic, subject)?;

        let mapfilename = format!("{}/{}/db/town.txt", AI_PATO_DIR, self.id);
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(mapfilename)?;

        writeln!(file, "{}", subject)?;

        let response = EmptyRequest {};

        Ok(Response::new(response))
    }

    async fn request_summary_and_embedding(
        &self,
        request: tonic::Request<SummaryAndEmbeddingRequest>,
    ) -> std::result::Result<tonic::Response<SummaryAndEmbeddingResponse>, tonic::Status> {
        let link = request.get_ref().link.clone();
        let knowledge_file = request.get_ref().knowledge_file.clone();
        let knowledge_file_fullpath = format!(
            "{}/{}/knowledge/{}",
            AI_PATO_DIR,
            self.id,
            request.get_ref().knowledge_file
        );
        let transcript_file = request.get_ref().transcript_file.clone();
        let collection_prefix = "sig".to_string();
        let subjects = get_event_subjects();
        let mut my_subjects: Vec<String> = vec![];

        let mut file_format = String::from("txt");
        if let Ok(mut file) = File::open(knowledge_file_fullpath.clone()) {
            let mut buf = vec![0; 4096]; // Read more bytes to improve accuracy
            if file.read(&mut buf).is_ok() {
                match infer::get(&buf) {
                    Some(kind) => {
                        println!("File type: {:?}", kind.mime_type());
                        file_format = kind.mime_type().to_string();
                    }
                    None => println!("Could not determine file type"),
                }
            }
        }
        if let Ok(mut llm_client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            if !knowledge_file.is_empty() {
                let llmrequest = tonic::Request::new(SomeDocs {
                    doc_file: knowledge_file_fullpath.clone(),
                    doc_format: file_format.clone(),
                });
                log!("file llmrequest: {:?}", llmrequest);
                match llm_client.got_documents_summary(llmrequest).await {
                    Ok(sum_resp) => {
                        let summary = sum_resp.get_ref().summary.clone();
                        let summary_file_path = format!(
                            "{}/{}/knowledge/{}.summary",
                            AI_PATO_DIR,
                            self.id,
                            request.get_ref().knowledge_file_sig.clone()
                        );
                        match OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(summary_file_path)
                        {
                            Ok(mut sig_file) => {
                                let _ = sig_file.write_all(summary.as_bytes());
                            }
                            Err(e) => {
                                log!("write summary file error: {}", e);
                            }
                        }
                        let topic_subject_request = tonic::Request::new(EventTopic {
                            topic: summary,
                            subjects: subjects
                                .iter()
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>(),
                        });
                        if let Ok(response) =
                            llm_client.got_topic_subject(topic_subject_request).await
                        {
                            if response.get_ref().subject.clone().len() < MAX_SUBJECT_LEN as usize {
                                my_subjects.push(response.get_ref().subject.clone());
                            }
                        }
                    }
                    Err(e) => {
                        log!("got_documents_summary error: {}", e);
                    }
                };
                let embed_request = tonic::Request::new(DocsRequest {
                    doc_file: knowledge_file_fullpath,
                    collection: collection_prefix.clone()
                        + &request.get_ref().knowledge_file_sig.clone(),
                    db_path: format!("{}/{}/db/knowledge_chromadb", AI_PATO_DIR, self.id.clone()),
                    doc_id: request.get_ref().knowledge_file_sig.clone(),
                    doc_format: file_format,
                });

                if let Err(e) = llm_client.embed_documents(embed_request).await {
                    log!("embed_documents error: {}", e);
                }
            }
            if !transcript_file.is_empty() {
                // process transcript file
                let file_format = String::from("txt");
                let llmrequest = tonic::Request::new(SomeDocs {
                    doc_file: transcript_file.clone(),
                    doc_format: file_format.clone(),
                });
                log!("record llmrequest: {:?}", llmrequest);
                let sum_resp = llm_client.got_documents_summary(llmrequest).await?;
                let summary = sum_resp.get_ref().summary.clone();
                let summary_file_path = format!(
                    "{}/{}/knowledge/{}.summary",
                    AI_PATO_DIR,
                    self.id,
                    request.get_ref().transcript_file_sig.clone()
                );
                if let Ok(mut sig_file) = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&summary_file_path)
                {
                    let _ = sig_file.write_all(summary.as_bytes());
                }
                let topic_subject_request = tonic::Request::new(EventTopic {
                    topic: summary,
                    subjects: subjects
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                });
                if let Ok(response) = llm_client.got_topic_subject(topic_subject_request).await {
                    my_subjects.push(response.get_ref().subject.clone());
                }
                let embed_request = tonic::Request::new(DocsRequest {
                    doc_file: transcript_file,
                    collection: collection_prefix.clone()
                        + &request.get_ref().transcript_file_sig.clone(),
                    db_path: format!("{}/{}/db/knowledge_chromadb", AI_PATO_DIR, self.id.clone()),
                    doc_id: request.get_ref().transcript_file_sig.clone(),
                    doc_format: file_format,
                });

                if let Err(e) = llm_client.embed_documents(embed_request).await {
                    log!("record embed_documents error: {}", e);
                }
            }
            if !link.is_empty() {
                // process web link
                let file_format = String::from("link");
                let llmrequest = tonic::Request::new(SomeDocs {
                    doc_file: link.clone(),
                    doc_format: file_format.clone(),
                });
                log!("link llmrequest: {:?}", llmrequest);
                let sum_resp = llm_client.got_documents_summary(llmrequest).await?;
                let summary = sum_resp.get_ref().summary.clone();
                let summary_file_path = format!(
                    "{}/{}/knowledge/{}.summary",
                    AI_PATO_DIR,
                    self.id,
                    request.get_ref().link_sig.clone()
                );
                if let Ok(mut sig_file) = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&summary_file_path)
                {
                    let _ = sig_file.write_all(summary.as_bytes());
                }
                let topic_subject_request = tonic::Request::new(EventTopic {
                    topic: summary,
                    subjects: subjects
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                });
                if let Ok(response) = llm_client.got_topic_subject(topic_subject_request).await {
                    my_subjects.push(response.get_ref().subject.clone());
                }
                let embed_request = tonic::Request::new(DocsRequest {
                    doc_file: link.clone(),
                    collection: collection_prefix + &request.get_ref().link_sig.clone(),
                    db_path: format!("{}/{}/db/knowledge_chromadb", AI_PATO_DIR, self.id.clone()),
                    doc_id: request.get_ref().link_sig.clone(),
                    doc_format: file_format,
                });

                if let Err(e) = llm_client.embed_documents(embed_request).await {
                    log!("link embed_documents error: {}", e);
                }
            }
        }

        let response = SummaryAndEmbeddingResponse {
            knowledge_file_sig: request.get_ref().knowledge_file_sig.clone(),
            transcript_file_sig: request.get_ref().transcript_file_sig.clone(),
        };
        let collections = [
            request.get_ref().knowledge_file_sig.clone(),
            request.get_ref().transcript_file_sig.clone(),
            request.get_ref().link_sig.clone(),
        ];
        Ok(Response::new(response))
    }

    async fn request_query_embedding(
        &self,
        request: tonic::Request<QueryEmbeddingRequest>,
    ) -> std::result::Result<tonic::Response<QueryEmbeddingResponse>, tonic::Status> {
        let db_path = format!("{}/{}/db/knowledge_chromadb", AI_PATO_DIR, self.id.clone());
        let collection_prefix = "sig".to_string();
        let mut result = String::default();
        if let Ok(mut llm_client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let llmrequest = tonic::Request::new(QueryEmbeddingsRequest {
                collection_name: collection_prefix + &request.get_ref().collection_name.clone(),
                question: request.get_ref().query.clone(),
                db_path,
            });
            log!("query request: {:?}", llmrequest);
            let query_resp = llm_client.query_embbeedings(llmrequest).await?;
            result = query_resp.get_ref().result.clone();
        }

        let response = QueryEmbeddingResponse { result };

        Ok(Response::new(response))
    }

    async fn request_document_summary(
        &self,
        request: tonic::Request<DocumentSummaryRequest>,
    ) -> std::result::Result<tonic::Response<DocumentSummaryResponse>, tonic::Status> {
        let mut summary = String::new();
        let summary_file_path = format!(
            "{}/{}/knowledge/{}.summary",
            AI_PATO_DIR,
            self.id,
            request.get_ref().document
        );
        if let Ok(mut sig_file) = OpenOptions::new().read(true).open(summary_file_path) {
            let _ = sig_file.read_to_string(&mut summary);
        }

        let response = DocumentSummaryResponse { summary };

        Ok(Response::new(response))
    }

    async fn request_pato_knowledges(
        &self,
        _request: tonic::Request<KnowLedgesRequest>,
    ) -> std::result::Result<tonic::Response<KnowLedgesResponse>, tonic::Status> {
        let saved_file_sig = format!("{}/{}/knowledge/knowledge.sig", AI_PATO_DIR, self.id);
        let mut knowledges: Vec<KnowLedgeInfo> = vec![];

        let mut my_knowledges: Vec<String> = vec![];
        let file = OpenOptions::new().read(true).open(saved_file_sig)?;
        let reader = io::BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            my_knowledges.push(line);
        }
        for knowledge in my_knowledges.iter() {
            let mut summary = String::new();
            let line = knowledge
                .split('#')
                .map(|f| f.to_string())
                .collect::<Vec<String>>();
            let title = if !line.is_empty() {
                line[0].clone()
            } else {
                String::default()
            };
            let sig = if line.len() > 1 {
                line[1].clone()
            } else {
                String::default()
            };
            let owner = if line.len() > 2 {
                line[2].clone()
            } else {
                self.id.clone()
            };
            let summary_file_path =
                format!("{}/{}/knowledge/{}.summary", AI_PATO_DIR, self.id, sig);
            if let Ok(mut sig_file) = OpenOptions::new().read(true).open(summary_file_path) {
                let _ = sig_file.read_to_string(&mut summary);
            }
            let info = KnowLedgeInfo {
                title: title.to_string(),
                sig: sig.to_string(),
                summary,
                owner,
            };
            knowledges.push(info);
        }
        let mut set = HashSet::new();
        let mut result = Vec::new();
        for item in knowledges {
            if set.insert(item.sig.clone()) {
                result.push(item.clone());
            }
        }

        let response = KnowLedgesResponse {
            knowledge_info: result,
        };

        Ok(Response::new(response))
    }

    async fn request_share_knowledge(
        &self,
        request: tonic::Request<ShareKnowLedgesRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        let share_file = format!("{}/{}/knowledge/shared.txt", AI_PATO_DIR, self.id.clone());
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(share_file)
        {
            Ok(mut file) => {
                writeln!(
                    file,
                    "{}#{}\n",
                    request.get_ref().title.clone(),
                    request.get_ref().sig.clone()
                )?;
            }
            Err(e) => {
                log!("share_knowledge write file error: {}", e);
            }
        }

        let response = EmptyRequest {};

        Ok(Response::new(response))
    }

    async fn add_shared_knowledge(
        &self,
        request: tonic::Request<ShareKnowLedgesRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        let saved_file_sig = format!("{}/{}/knowledge/knowledge.sig", AI_PATO_DIR, self.id);
        if let Ok(mut sig_file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(saved_file_sig)
        {
            let _ = sig_file.write_all(
                format!(
                    "{}#{}#{}\n",
                    request.get_ref().title,
                    request.get_ref().sig,
                    request.get_ref().owner.clone()
                )
                .as_bytes(),
            );
        }

        let response = EmptyRequest {};

        Ok(Response::new(response))
    }

    async fn request_generate_scene(
        &self,
        request: tonic::Request<SceneRequest>,
    ) -> std::result::Result<tonic::Response<SceneResponse>, tonic::Status> {
        let description = request.get_ref().description.clone();
        let room_id = request.get_ref().room_id.clone();
        let image_file_name = uuid::Uuid::new_v4().to_string();
        let mut resp = SceneResponse {
            scene_image: String::default(),
        };

        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let final_image_prompt = format!(
                "(equirectangular panorama 360 panoramic:1.1) picture of, professional high quality, photography of a ({}),
                 architect portfolio,, Artstation, by Brandon Barré,, 8k resolution, detailed, focus,",
                description
            );
            let image_request = tonic::Request::new(ImageGenRequest {
                prompt: final_image_prompt.clone(),
            });
            println!("image gen request: {:?}", image_request);
            match client.gen_image_with_prompt(image_request).await {
                Ok(answer) => {
                    let image_url = answer.get_ref().image_url.clone();
                    let saved_local_file = format!("{}/game/{}", XFILES_LOCAL_DIR, image_file_name);
                    let xfiles_link = format!("{}/game/{}", XFILES_SERVER, image_file_name);
                    match download_image(&image_url, &saved_local_file).await {
                        Ok(_) => {
                            let _ = publish_battery_actions(
                                request.get_ref().room_id.clone(),
                                "notification: 场景切换中".to_string(),
                            );
                            resp.scene_image = xfiles_link;
                            let game_room_path =
                                format!("{}/{}/db/game_room/{}", AI_PATO_DIR, self.id, room_id);
                            let _ = ensure_directory_exists(&game_room_path);
                            if let Ok(mut file) = OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(format!("{}/scene.txt", game_room_path))
                            {
                                writeln!(file, "{}", resp.scene_image)?;
                            }
                            let mut levels = 0;
                            if let Ok(file) = OpenOptions::new()
                                .read(true)
                                .open(format!("{}/scene.txt", game_room_path))
                            {
                                let reader = io::BufReader::new(file);
                                levels = reader.lines().count();
                            }
                            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(
                                format!("{}/scene_{}_prompt.txt", game_room_path, levels - 1),
                            ) {
                                writeln!(file, "{}", final_image_prompt)?;
                            }
                        }
                        Err(e) => {
                            log!("download image error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log!("image_request AI is something wrong: {}", e);
                }
            }
        }

        Ok(Response::new(resp))
    }

    async fn request_image_description(
        &self,
        request: tonic::Request<SvcImageDescriptionRequest>,
    ) -> std::result::Result<tonic::Response<SvcImageDescriptionResponse>, tonic::Status> {
        let sample_image = request.get_ref().image_url.clone();
        let mut resp = SvcImageDescriptionResponse {
            description: String::default(),
        };

        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            log!("sample image file url: {}", sample_image);
            let image_description_request = tonic::Request::new(ImageDescriptionRequest {
                image_url: sample_image,
            });
            match client
                .request_image_description(image_description_request)
                .await
            {
                Ok(answer) => {
                    let description = answer.get_ref().description.clone();
                    resp.description = description;
                }
                Err(e) => {
                    log!("image_description_request AI is something wrong: {}", e);
                }
            }
        }
        Ok(Response::new(resp))
    }

    async fn request_chat_with_image(
        &self,
        request: tonic::Request<ImageChatRequest>,
    ) -> std::result::Result<tonic::Response<ImageChatResponse>, tonic::Status> {
        let mut response = ImageChatResponse {
            answer: String::default(),
            answer_voice: String::default(),
        };
        let local_xfile = request
            .get_ref()
            .image_url
            .split('/')
            .last()
            .unwrap_or_default();
        let local_file = format!("{}/game/{}", XFILES_LOCAL_DIR, local_xfile);
        log!("local_file: {}", local_file);
        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let chat_request = tonic::Request::new(LLMImageChatRequest {
                image_url: request.get_ref().image_url.clone(),
                question: request.get_ref().message.clone(),
            });
            println!("chat_image_request: {:?}", chat_request);
            match client.request_image_chat(chat_request).await {
                Ok(answer) => {
                    response.answer = answer.get_ref().description.clone();
                    let tts_request = tonic::Request::new(TextToSpeechRequest {
                        text: answer.get_ref().description.clone(),
                    });
                    match client.text_to_speech(tts_request).await {
                        Ok(audio_file) => {
                            let audio_url =
                                XFILES_SERVER.to_string() + "/" + &audio_file.get_ref().audio_url;
                            response.answer_voice = audio_url;
                        }
                        Err(e) => {
                            log!(
                                "request_chat_with_image Text to speech is something wrong: {}",
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    log!("request_chat_with_image AI is something wrong: {}", e);
                }
            }
        }
        Ok(Response::new(response))
    }

    async fn request_join_room(
        &self,
        request: tonic::Request<JoinRoomRequest>,
    ) -> std::result::Result<tonic::Response<JoinRoomResponse>, tonic::Status> {
        let room_id = request.get_ref().room_id.clone();
        let gamer_id = request.get_ref().id.clone();
        let gamer_name = request.get_ref().name.clone();
        let game_level = request.get_ref().level;
        let game_room_path = format!("{}/{}/db/game_room/{}", AI_PATO_DIR, self.id, room_id);
        let _ = ensure_directory_exists(&game_room_path);
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/gamer.txt", game_room_path))
        {
            writeln!(file, "{}#{}#{}", gamer_id, gamer_name, game_level)?;
        }

        let message = format!("notification:{}进入房间", gamer_name);
        let _ = publish_battery_actions(room_id.clone(), message);

        let mut scene = String::default();
        let mut scene_count = 0;
        log!("game_room_path: {}", game_room_path);
        if let Ok(file) = OpenOptions::new()
            .read(true)
            .open(format!("{}/scene.txt", game_room_path))
        {
            let reader = io::BufReader::new(file);
            let lines = reader.lines();
            for (i, line) in lines.map_while(Result::ok).enumerate() {
                scene_count += 1;
                log!("scene: {}", scene);
                if i == game_level as usize {
                    scene = line;
                }
            }
        }
        let response = JoinRoomResponse {
            scene_count,
            last_scene: scene,
        };

        Ok(Response::new(response))
    }

    async fn request_clue_from_image_chat(
        &self,
        request: tonic::Request<ImageChatRequest>,
    ) -> std::result::Result<tonic::Response<ImageChatResponse>, tonic::Status> {
        let mut sn: i64 = -1;
        let owner = request.get_ref().reply_to.clone();
        let req = SnRequest {
            id: vec![owner.clone()],
        };
        match call_update_method(AGENT_SMITH_CANISTER, "request_sn", req).await{
            Ok(result) => {
                let response = Decode!(result.as_slice(), SnResponse).unwrap_or_default();
                let resp = response.pato_sn_id;
                if !resp.is_empty() {
                    sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
                } else {
                    println!("send_pato_instruct: not found this one");
                }
            }
            Err(e) => { 
                println!("request_sn error: {}", e);
            }
        }
        if sn >= 0 {
            let battery_address = format!(
                "{}:{}",
                BATTERY_GRPC_REST_SERVER,
                sn + BATTERY_GRPC_SERVER_PORT_START
            );
            if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await
            {
                let req = tonic::Request::new(ImageChatRequest {
                    message: request.get_ref().message.clone(),
                    reply_to: self.id.clone(),
                    image_url: request.get_ref().image_url.clone(),
                    room_id: "".to_string(),
                    level: 0,
                });
                match client.request_chat_with_image(req).await {
                    Ok(answer) => {
                        return Ok(answer);
                    }
                    Err(e) => {
                        println!("send_pato_instruct error: {:?}", e);
                    }
                }
            }
        }

        Ok(Response::new(ImageChatResponse::default()))
    }

    async fn accept_game_answer(
        &self,
        _request: tonic::Request<EmptyRequest>,
    ) -> std::result::Result<tonic::Response<GameAnswerResponse>, tonic::Status> {
        let win_gamers: Vec<String> = vec![];
        let resp = GameAnswerResponse {
            correct_gamers: win_gamers,
        };

        Ok(Response::new(resp))
    }

    async fn receive_game_answer(
        &self,
        request: tonic::Request<GameAnswerRequest>,
    ) -> std::result::Result<tonic::Response<GameAnswerResponse>, tonic::Status> {
        let room_id = request.get_ref().room_id.clone();
        let gamer_id = request.get_ref().id.clone();
        let answer = request.get_ref().answer.clone();
        let gamer_name = request.get_ref().name.clone();
        let game_level = request.get_ref().level;

        let game_room_path = format!("{}/{}/db/game_room/{}", AI_PATO_DIR, self.id, room_id);
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/gamer_answer.txt", game_room_path))
        {
            writeln!(
                file,
                "{}#{}#{}#{}",
                gamer_id, gamer_name, game_level, answer
            )?;
        }
        let mut buffer = String::new();
        if let Ok(mut file) = OpenOptions::new()
            .read(true)
            .open(format!("{}/answer_{}.txt", game_room_path, game_level))
        {
            let _ = file.read_to_string(&mut buffer);
        }

        let message = format!("notification:{}发送答案", gamer_name);
        let _ = publish_battery_actions(room_id.clone(), message);

        let prompt = format!(
            r#"请仔细阅读下面的背景说明：
            上下文-1:
            以下一个问题的标准答案：
            {}

            上下文-2:
            以下是用户{}基于问题所作的回答：
            {}

            根据上面提供的上下文, 判断用户{}的回答是否和标准答案的表述基本一致, 如果一致请输出yes, 否则请输出no.

            {}:{}
            AI:
            "#,
            buffer, gamer_name, answer, gamer_name, gamer_name, answer
        );

        let mut winner: Vec<String> = vec![];
        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let chat_request = tonic::Request::new(QuestionRequest {
                question: prompt,
                subject: String::default(),
                persona: String::default(),
            });
            println!("chat_request: {:?}", chat_request);
            match client.talk(chat_request).await {
                Ok(answer) => {
                    log!("check_game_answer: {:?}", answer.get_ref().answer);
                    if answer.get_ref().answer.contains("yes")
                        || answer.get_ref().answer.contains("Yes")
                    {
                        let _ = publish_battery_actions(
                            room_id.clone(),
                            format!("notification:{}回答正确", gamer_name),
                        );
                        winner.push(gamer_id);
                    }
                }
                Err(e) => {
                    log!("check_game_answer AI is something wrong: {}", e);
                }
            }
        }

        let response = GameAnswerResponse {
            correct_gamers: winner,
        };

        Ok(Response::new(response))
    }

    async fn request_answer_image(
        &self,
        request: tonic::Request<ImageAnswerRequest>,
    ) -> std::result::Result<tonic::Response<ImageContextResponse>, tonic::Status> {
        let mut response = ImageContextResponse {
            context: String::default(),
        };
        let image_url = request.get_ref().image_url.clone();
        let input = request.get_ref().input.clone();
        let mut prompt = String::new();
        let room_id = request.get_ref().room_id.clone();
        let level = request.get_ref().level;

        let game_room_path = format!("{}/{}/db/game_room/{}", AI_PATO_DIR, self.id, room_id);

        if let Ok(mut file) = OpenOptions::new()
            .read(true)
            .open(format!("{}/scene_{}_prompt.txt", game_room_path, level))
        {
            let _ = file.read_to_string(&mut prompt);
        }

        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let chat_request = tonic::Request::new(ImagePromptRequest {
                image_url,
                prompt,
                input,
            });
            match client
                .request_image_description_with_prompt(chat_request)
                .await
            {
                Ok(answer) => {
                    response.context = answer.get_ref().description.clone();
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(format!("{}/answer_{}.txt", game_room_path, level))
                    {
                        writeln!(file, "{}", response.context)?;
                    }
                }
                Err(e) => {
                    log!("request_answer_image AI is something wrong: {}", e);
                }
            }
        }
        Ok(Response::new(response))
    }

    async fn request_reveal_answer(
        &self,
        request: tonic::Request<RevealAnswerRequest>,
    ) -> std::result::Result<tonic::Response<RevealAnswerResponse>, tonic::Status> {
        let game_room_path = format!(
            "{}/{}/db/game_room/{}",
            AI_PATO_DIR,
            self.id,
            request.get_ref().room_id.clone()
        );
        let mut buffer = String::new();
        if let Ok(mut file) = OpenOptions::new().read(true).open(format!(
            "{}/answer_{}.txt",
            game_room_path,
            request.get_ref().level
        )) {
            let _ = file.read_to_string(&mut buffer);
        }

        let response = RevealAnswerResponse { answer: buffer };

        Ok(Response::new(response))
    }

    async fn become_kol(
        &self,
        request: tonic::Request<BecomeKolRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        let req = KolRegistrationRequest {
            id: self.id.clone(),
            key: request.get_ref().key.clone(),
        };
        call_update_method(AGENT_SMITH_CANISTER, "request_kol_registration", req).await;
        let response = EmptyRequest {};

        Ok(Response::new(response))
    }

    async fn request_join_kol_room(
        &self,
        request: tonic::Request<JoinKolRoomRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        let req = FollowKolRequest {
            key: request.get_ref().key.clone(),
            follower: request.get_ref().follower.clone(),
            id: request.get_ref().kol.clone(),
        };
        call_update_method(AGENT_SMITH_CANISTER, "request_add_kol_follower", req).await;

        let response = EmptyRequest {};

        Ok(Response::new(response))
    }

    async fn request_image_context(
        &self,
        request: tonic::Request<ImageContextRequest>,
    ) -> std::result::Result<tonic::Response<ImageContextResponse>, tonic::Status> {
        let image_url = request.get_ref().image_url.clone();
        let input = request.get_ref().input.clone();
        let prompt = request.get_ref().prompt.clone();

        let mut context = String::default();
        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let chat_request = tonic::Request::new(ImagePromptRequest {
                image_url,
                prompt,
                input,
            });
            // println!("chat_request: {:?}", chat_request);
            match client
                .request_image_description_with_prompt(chat_request)
                .await
            {
                Ok(answer) => {
                    context = answer.get_ref().description.clone();
                }
                Err(e) => {
                    log!("request_image_context AI is something wrong: {}", e);
                }
            }
        }
        let response = ImageContextResponse { context };

        Ok(Response::new(response))
    }

    async fn request_image_gen_prompt(
        &self,
        request: tonic::Request<ImageGenPromptRequest>,
    ) -> std::result::Result<tonic::Response<ImageContextResponse>, tonic::Status> {
        let mut curr_input: Vec<String> = vec![];
        let prompt_lib_file = format!("{}/template/plan/agent_chat_maker.txt", AI_MATRIX_DIR);
        let description = request.get_ref().description.clone();
        let his = request.get_ref().historical.clone();
        let cul = request.get_ref().architectural.clone();
        let mut image_prompt = String::default();

        curr_input.push(description); //0
        curr_input.push(his); //1
        curr_input.push(cul); //2
        let maker_prompt = generate_prompt(curr_input, &prompt_lib_file);
        log!("maker_prompt: {}", maker_prompt);

        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let chat_request = tonic::Request::new(QuestionRequest {
                subject: String::default(),
                persona: maker_prompt,
                question: String::default(),
            });
            match client.talk(chat_request).await {
                Ok(answer) => {
                    image_prompt = answer.get_ref().answer.clone();
                }
                Err(e) => {
                    log!("Maker AI {} is something wrong: {}", self.id, e);
                }
            }
        }

        let response = ImageContextResponse {
            context: image_prompt,
        };

        Ok(Response::new(response))
    }

    async fn request_self_talk_for_today_plan(
        &self,
        _request: tonic::Request<EmptyRequest>,
    ) -> std::result::Result<tonic::Response<EmptyRequest>, tonic::Status> {
        let send_to = self.id.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
                    let chat_request = tonic::Request::new(QuestionRequest {
                        subject: String::default(),
                        persona: "I want to do something today".to_string(),
                        question: String::default(),
                    });
                    match client.talk(chat_request).await {
                        Ok(answer) => {
                            let _ = publish_battery_actions(
                                send_to.clone(),
                                answer.get_ref().answer.clone(),
                            );
                        }
                        Err(e) => {
                            log!(
                                "request_self_talk_for_today_plan AI is something wrong: {}",
                                e
                            );
                        }
                    }
                }
                sleep(std::time::Duration::from_secs(TICK * 2)).await;
            }
        });

        Ok(Response::new(EmptyRequest {}))
    }

    async fn request_submit_tags(
        &self,
        request: tonic::Request<SubmitTagsRequest>,
    ) -> std::result::Result<tonic::Response<SubmitTagsResponse>, tonic::Status> {
        let mut resp = SubmitTagsResponse::default();

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/{}/db/tags.json", AI_PATO_DIR, self.id))
        {
            writeln!(
                file,
                "{}",
                serde_json::to_string(&request.get_ref().tags).unwrap_or_default()
            )?;
        }
        if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
            let tag_request = tonic::Request::new(CharacterGenRequest {
                tags: request.get_ref().tags.clone(),
                name: get_pato_name(self.id.clone()).unwrap_or("nobody".to_string()),
                gender: "Unknown".to_string(),
            });
            match client.gen_character_with_prompt(tag_request).await {
                Ok(answer) => {
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(format!("{}/{}/db/character.txt", AI_PATO_DIR, self.id))
                    {
                        writeln!(file, "{}", answer.get_ref().iss)?;
                    }
                    let image_request = tonic::Request::new(ImageGenRequest {
                        prompt: answer.get_ref().iss.clone(),
                    });
                    // println!("chat_request: {:?}", chat_request);
                    match client.gen_image_with_prompt(image_request).await {
                        Ok(answer) => {
                            resp.avatar = answer.get_ref().image_url.clone();
                            let _ = ensure_directory_exists(&format!(
                                "{}/avatar/{}",
                                XFILES_LOCAL_DIR, self.id
                            ));
                            let saved_local_file =
                                format!("{}/avatar/{}/avatar.png", XFILES_LOCAL_DIR, self.id);
                            let xfiles_link =
                                format!("{}/avatar/{}/avatar.png", XFILES_SERVER, self.id);
                            match download_image(&resp.avatar, &saved_local_file).await {
                                Ok(_) => {
                                    resp.avatar = xfiles_link;
                                }
                                Err(e) => {
                                    log!("download avatar error: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log!("image_request AI is something wrong: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log!("gen_character_with_prompt AI is something wrong: {}", e);
                }
            }
        }

        Ok(Response::new(resp))
    }

}

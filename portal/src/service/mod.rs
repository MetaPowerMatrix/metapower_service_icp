use candid::CandidType;
use serde::{Deserialize, Serialize};

pub mod ai_town;
pub mod bsc_proxy;
pub mod llm_proxy;

#[derive(Deserialize, CandidType)]
pub struct Knowledge {
    pub sig: String,
    pub title: String,
    pub owner: String,
    pub summary: String,
}

#[derive(Deserialize, CandidType, Default)]
pub struct SharedKnowledgesResponse {
    pub books: Vec<Knowledge>,
}

#[derive(Deserialize, CandidType, Default)]
pub struct HotTopicResponse {
    pub topics: Vec<String>,
}

#[derive(Deserialize, CandidType, Default, Debug)]
pub struct CreateResonse {
    pub id: String,
}

#[derive(Deserialize, CandidType)]
pub struct LoginRequest {
    pub id: String,
}

#[derive(Deserialize, CandidType, Serialize)]
pub struct AirdropRequest {
    pub id: String,
    pub amount: f32,
}

#[derive(Deserialize, CandidType, Default)]
pub struct SimpleResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, CandidType)]
pub struct PopulationRegistrationRequest {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, CandidType, Default, Serialize)]
pub struct PatoInfoResponse {
    pub id: String,
    pub name: String,
    pub sn: i64,
    pub registered_datetime: String,
    pub balance: f32,
    pub tags: Vec<String>,
    pub avatar: String,
    pub cover: String,
    pub followers: Vec<(String, String)>,
    pub followings: Vec<(String, String)>,
}

#[derive(Deserialize, CandidType, Debug)]
pub struct PatoOfPro {
    pub id: String,
    pub subjects: Vec<String>,
    pub name: String,
}

#[derive(Deserialize, CandidType)]
pub struct SnIdPaire {
    pub id: String,
    pub sn: String,
}

#[derive(Deserialize, Serialize, CandidType, Default)]
pub struct TokenResponse {
    pub id: String,
    pub name: String,
    pub token: String,
}

#[derive(Deserialize, CandidType, Default)]
pub struct TopicChatHisResponse {
    pub history: Vec<String>,
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

#[derive(Deserialize, CandidType, Serialize)]
pub struct KolRelations {
    pub id: String,
    pub name: String,
    pub follower: Vec<String>,
}

#[derive(Deserialize, CandidType, Default)]
pub struct KolListResponse {
    pub relations: Vec<KolRelations>,
}

#[derive(Deserialize, CandidType)]
pub struct UserActiveRequest {
    pub id: String,
    pub page: String,
    pub action: String,
}

#[derive(Deserialize, Serialize, CandidType)]
pub struct SubmitTagsRequest {
    pub id: String,
    pub session: String,
    pub tags: Vec<String>,
}

#[derive(Deserialize, Serialize, CandidType)]
pub struct BecomeKolRequest {
    pub id: String,
    pub from: String,
}

#[derive(Deserialize, Serialize, CandidType)]
pub struct JoinKolRoomRequest {
    pub kol: String,
    pub follower: String,
    pub key: String,
    pub from: String,
}

#[derive(Deserialize, Serialize, CandidType)]
pub struct QueryEmbeddingRequest {
    pub query: String,
    pub collection_name: String,
}

#[derive(Deserialize, Serialize, CandidType)]
pub struct DocumentSummaryRequest {
    pub document: String,
}

#[derive(Deserialize, CandidType, Serialize, Debug, Default)]
pub struct FileUploadInfo {
    pub file_name: String,
    pub file_type: String,
    pub biz: String,
}


use candid::CandidType;
use serde::{Deserialize, Serialize};

pub mod ai_town;

// from metapowermatrix_matrix

#[derive(Deserialize, CandidType, Default)]
pub struct EmptyResponse {}

#[derive(Deserialize, CandidType)]
pub struct CheckPayOrdersRequest {
    pub order: String,
    pub buyer_id: String,
}

#[derive(Deserialize, CandidType)]
pub struct CheckPayOrdersResponse {
    pub order: String,
    pub paid: bool,
}

#[derive(Deserialize, CandidType)]
pub struct CreditCardPayRequest {
    pub id: String,
    pub item: String,
    pub amount: i32,
}

#[derive(Deserialize, CandidType)]
pub struct CreditCardPayResponse {
    pub pay_url: String,
}

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

#[derive(Deserialize, CandidType, Debug)]
pub struct HotAi {
    pub id: String,
    pub name: String,
    pub talks: i32,
    pub pros: String,
}

#[derive(Deserialize, CandidType, Default)]
pub struct HotAiResponse {
    pub sheniu: Vec<HotAi>,
}

#[derive(Deserialize, CandidType, Default)]
pub struct HotTopicResponse {
    pub topics: Vec<String>,
}

#[derive(Deserialize, CandidType)]
pub struct SubscriptionRequest {
    pub id: String,
    pub amount: f32,
    pub sub_type: String,
}

#[derive(Deserialize, CandidType)]
pub struct DonationRequest {
    pub id: String,
    pub amount: f32,
}

#[derive(Deserialize, CandidType)]
pub struct CreateRequest {
    pub name: String,
}

#[derive(Deserialize, CandidType, Default)]
pub struct CreateResonse {
    pub id: String,
}

#[derive(Deserialize, CandidType)]
pub struct NearbyRequest {
    pub sn: i64,
}

#[derive(Deserialize, CandidType)]
pub struct NearbyRespnse {
    pub id: Vec<String>,
}

#[derive(Deserialize, CandidType)]
pub struct LoginRequest {
    pub id: String,
}

#[derive(Deserialize, CandidType)]
pub struct PrayRequest {
    pub id: String,
    pub message: String,
}

#[derive(Deserialize, CandidType)]
pub struct MakeProfessionRequest {
    pub id: String,
    pub message: String,
    pub knowledge: String,
    pub file_sig: String,
}

#[derive(Deserialize, CandidType)]
pub struct MakePlanRequest {
    pub id: String,
    pub message: String,
    pub refresh: bool,
}

#[derive(Deserialize, CandidType)]
pub struct MakePlanResponse {
    pub plan_file: String,
}

#[derive(Deserialize, CandidType)]
pub struct PlaceRequest {
    pub place_type: String,
}

#[derive(Deserialize, CandidType)]
pub struct PlaceResonse {
    pub sn: Vec<i64>,
}

#[derive(Deserialize, CandidType)]
pub struct BatteryInfo {
    pub sn: i64,
    pub id: String,
    pub canister: String,
}

// from metapowermatrix_agent


#[derive(Deserialize, CandidType, Serialize)]
pub struct EmptyRequest {}

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

#[derive(Deserialize, CandidType)]
pub struct SimpleRequest {
    pub id: String,
}

#[derive(Deserialize, CandidType, Default)]
pub struct PatoInfoResponse {
    pub id: String,
    pub name: String,
    pub sn: i64,
    pub registered_datetime: String,
    pub professionals: Vec<String>,
    pub balance: f32,
    pub tags: Vec<String>,
    pub avatar: String,
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

#[derive(Deserialize, CandidType)]
pub struct SnRequest {
    pub id: Vec<String>,
}

#[derive(Deserialize, CandidType, Default)]
pub struct SnResponse {
    pub pato_sn_id: Vec<SnIdPaire>,
}

#[derive(Deserialize, CandidType)]
pub struct AllPatosResponse {
    pub pato_sn_id: Vec<SnIdPaire>,
}

#[derive(Deserialize, CandidType)]
pub struct ChangeBalanceRequest {
    pub id: String,
    pub amount: f32,
    pub key: String,
}

#[derive(Deserialize, CandidType)]
pub struct InjectHumanVoiceRequest {
    pub id: String,
    pub roles: Vec<String>,
    pub session: String,
    pub message: String,
}

#[derive(Deserialize, CandidType)]
pub struct TokenRequest {
    pub token: String,
}

#[derive(Deserialize, CandidType, Default)]
pub struct TokenResponse {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, CandidType)]
pub struct TopicChatRequest {
    pub initial: String,
    pub topic: String,
    pub town: String,
}

#[derive(Deserialize, CandidType, Default)]
pub struct TopicChatHisResponse {
    pub history: Vec<String>,
}

#[derive(Deserialize, CandidType)]
pub struct ProfessionalsResponse {
    pub professionals: Vec<String>,
}

#[derive(Deserialize, CandidType)]
pub struct RoomCreateRequest {
    pub owner: String,
    pub title: String,
    pub description: String,
    pub town: String,
}

#[derive(Deserialize, CandidType, Default)]
pub struct RoomCreateResponse {
    pub room_id: String,
    pub cover: String,
}

#[derive(Deserialize, CandidType, Default)]
pub struct RoomListResponse {
    pub rooms: Vec<RoomInfo>,
}

#[derive(Deserialize, CandidType)]
pub struct RoomInfo {
    pub room_id: String,
    pub owner: String,
    pub title: String,
    pub description: String,
    pub cover: String,
    pub town: String,
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

#[derive(Deserialize, CandidType)]
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
pub struct PatoOfProResponse {
    pub patos: Vec<PatoOfPro>,
}

#[derive(Deserialize, CandidType)]
pub struct UserActiveRequest {
    pub id: String,
    pub page: String,
    pub action: String,
}

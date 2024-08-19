use std::{fs::File, io::{BufRead, BufReader}};

use metapower_framework::{service::{metapowermatrix_agent_mod::agent_grpc::{meta_power_matrix_agent_svc_client::MetaPowerMatrixAgentSvcClient, SnRequest}, metapowermatrix_battery_mod::battery_grpc::{meta_power_matrix_battery_svc_client::MetaPowerMatrixBatterySvcClient, EmptyRequest, KnowLedgeInfo, KnowLedgesRequest}}, AGENT_GRPC_REST_SERVER, AI_PATO_DIR, BATTERY_GRPC_REST_SERVER, BATTERY_GRPC_SERVER_PORT_START};

pub fn get_pato_name(id: String)-> Option<String>{
    let mut name = String::default();
    let name_file = format!("{}/{}/db/name.txt", AI_PATO_DIR, id);
    if let Ok(file) = File::open(name_file){
        let reader = BufReader::new(file);
        if let Some(Ok(last_line)) = reader.lines().last(){
            name = last_line;
        }
    }

    Some(name)
}  
pub async fn ask_pato_knowledges(pato_id: String) -> Vec<KnowLedgeInfo>{
    let mut knowledges: Vec<KnowLedgeInfo> = vec![];
    let mut sn: i64 = -1;
    if let Ok(mut client) = MetaPowerMatrixAgentSvcClient::connect(AGENT_GRPC_REST_SERVER).await {
        let req = tonic::Request::new(SnRequest {
            id: vec![pato_id.clone()],
        });
        if let Ok(response) = client.request_pato_sn(req).await {
            let resp = response.get_ref().pato_sn_id.clone();
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("ask_pato_name: not found this one");
            }
        }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(KnowLedgesRequest { id: pato_id });
            match client.request_pato_knowledges(req).await{
                Ok(answer) => {
                    knowledges = answer.get_ref().knowledge_info.clone();
                }
                Err(e) => {
                    println!("ask_pato_knowledges error: {:?}", e);
                }
            }
        }
    }

    knowledges
}
pub async fn ask_pato_name(pato_id: String)-> Option<String>{
    let mut sn: i64 = -1;
    if let Ok(mut client) = MetaPowerMatrixAgentSvcClient::connect(AGENT_GRPC_REST_SERVER).await {
        let req = tonic::Request::new(SnRequest {
            id: vec![pato_id.clone()],
        });
        if let Ok(response) = client.request_pato_sn(req).await {
            let resp = response.get_ref().pato_sn_id.clone();
            if !resp.is_empty(){
                sn = resp[0].sn.parse::<i64>().unwrap_or(-1);
            }else{
                println!("ask_pato_name: not found this one");
            }
        }
    }
    if sn >= 0 {
        let battery_address = format!(
            "{}:{}",
            BATTERY_GRPC_REST_SERVER,
            sn + BATTERY_GRPC_SERVER_PORT_START
        );
        if let Ok(mut client) = MetaPowerMatrixBatterySvcClient::connect(battery_address).await {
            let req = tonic::Request::new(EmptyRequest {});
            match client.request_pato_name(req).await{
                Ok(answer) => {
                    return Some(answer.get_ref().name.clone());
                }
                Err(e) => {
                    println!("request_pato_name error: {:?}", e);
                }
            }
        }
    }

    None
}
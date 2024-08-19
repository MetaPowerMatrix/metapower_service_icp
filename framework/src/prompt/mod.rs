use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod gen;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserScratch {
    pub vision_r: i32,
    pub att_bandwidth: i32,
    pub retention: i32,
    pub curr_time: String,
    pub curr_tile: Vec<i32>,
    pub daily_plan_req: String,
    pub name: String,
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    pub innate: String,
    pub learned: String,
    pub currently: String,
    pub lifestyle: String,
    pub living_area: String,
    pub concept_forget: i32,
    pub daily_reflection_time: i32,
    pub daily_reflection_size: i32,
    pub overlap_reflect_th: i32,
    pub kw_strg_event_reflect_th: i32,
    pub kw_strg_thought_reflect_th: i32,
    pub recency_w: i32,
    pub relevance_w: i32,
    pub importance_w: i32,
    pub recency_decay: f64,
    pub importance_trigger_max: i32,
    pub importance_trigger_curr: i32,
    pub importance_ele_n: i32,
    pub thought_count: i32,
    pub daily_req: Vec<String>,
    pub f_daily_schedule: Vec<(String, i32)>,
    pub f_daily_schedule_hourly_org: Vec<(String, i32)>,
    pub act_address: String,
    pub act_start_time: String,
    pub act_duration: i32,
    pub act_description: String,
    pub act_pronunciatio: String,
    pub act_event: Vec<String>,
    pub act_obj_description: String,
    pub act_obj_pronunciatio: String,
    pub act_obj_event: Vec<String>,
    pub chatting_with: Option<String>, // Use Option for nullable fields
    pub chat: Option<String>,
    pub chatting_with_buffer: HashMap<String, i32>,
    pub chatting_end_time: Option<String>, // Nullable field
    pub act_path_set: bool,
    pub planned_path: Vec<String>, // Assuming this to be a vector of strings; adjust as necessary
}

fn get_str_iss(us: &UserScratch) -> String {
    let commonset = format!(
        "Name: {}\nAge: {}\nInnate traits: {}\nLearned traits: {}\nCurrently: {}\nLifestyle: {}\nDaily plan requirement: {}\nCurrent Date: {}",
        us.name,
        us.age,
        us.innate,
        us.learned,
        us.currently,
        us.lifestyle,
        us.daily_plan_req,
        us.curr_time,
    );
    commonset
}

pub fn create_prompt_input(persona: &UserScratch, test_input: Option<Vec<&str>>) -> Vec<String> {
    if let Some(input) = test_input {
        // If test_input is Some, return it directly, converting &str to String
        return input.into_iter().map(|s| s.to_string()).collect();
    }
    
    let prompt_input = vec![
        get_str_iss(persona).to_string(),
        persona.lifestyle.clone(),
        persona.first_name.clone(),
    ];
    
    prompt_input
}

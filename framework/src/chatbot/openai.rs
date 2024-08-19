use std::{ffi::OsString, process::{Command, Output}};

use reqwest::Client;
use serde_json::Value;

use crate::ActionInfo;

use super::GptParam;

pub struct OpenAIChatBot{
    pub params: GptParam,
    pub session: String,
}

impl Default for OpenAIChatBot {
    fn default() -> Self {
        OpenAIChatBot {
            params: GptParam {
                engine: "text-davinci-003".to_string(),
                max_tokens: 150,
                temperature: 0.7,
                top_p: 1.0,
                stream: false,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
                stop: None,
            },
            session: uuid::Uuid::new_v4().to_string(),
        }
    }    
}
impl OpenAIChatBot {
    pub async fn run_with_rust(&self, prompt: &str) -> Result<Vec<ActionInfo>, anyhow::Error> {
        let client = Client::new();
    
        let response = client.post("https://api.openai.com/v1/completions")
            .json(&self.params)
            .send()
            .await?
            .json::<Value>()
            .await?;
    
        let actions = vec![];
        Ok(actions)
    }
    fn exec_script_command(&self, script_file: String, args: Vec<String>) -> Option<Output>{
        let output = Command::new("bash")
            .arg(script_file)
            .arg(OsString::from(args[0].clone()).as_os_str())
            .output()
            .map_err(|e| println!("执行python脚本命令失败: {:?}", e));
    
        if let Ok(cmd) = output {
            return Some(cmd);
        }

        None
    }
    
    pub async fn run_with_python_script_plan(&self, prompt: &str) -> Result<Vec<ActionInfo>, anyhow::Error> {
        let actions = vec![];

        let script_file = "/data/bin/script/python/using_langchian_openai.py".to_string();
        let response  = self.exec_script_command(script_file, vec![prompt.to_string()]);

        if let Some(cmd) = response {
            println!("{:?}", cmd);
        }

        Ok(actions)
    }
}

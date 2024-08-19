use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

/*
use generative_models::{
    v1beta1::{
        generate_content_operation::OperationMetadata, generate_content_request::Request,
        generate_content_response::Response, generative_model_service_client::Client,
        SafetyBlockThreshold, SafetyCategory,
    },
    HarmfulContentDetection, Model, Settings,
};

pub async fn generate(
    project_id: &str,
    location: &str,
    model_id: &str,
    content: &str,
    max_tokens: i32,
    temperature: f32,
) -> Vec<String> {
    let client = Client::new().await.expect("Error creating client");

    let request = Request {
        model: Model::name(project_id, location, model_id),
        inputs: vec![content.to_owned()],
        generation_config: Some(crate::GenerationConfig {
            max_output_tokens: max_tokens,
            temperature,
            ..Default::default()
        }),
        safety_settings: Some(crate::SafetySettings {
            blacklists: vec![],
            detections: vec![HarmfulContentDetection {
                category: SafetyCategory::HateSpeech as i32,
                // TODO(russell-teller): actually make detection configurable.
                threshold: SafetyBlockThreshold::BlockMediumAndAbove as i32,
            }],
            whitelist: vec![],
        }),
        output_config: None,
    };

    // We will handle pagination manually so we can `await` to load each response as it is received
    // rather than waiting for the entire stream to be consumed.
    let mut paginator = client.generate_content(request).await.expect("Error making request");
    let mut responses = Vec::new();

    while let Some(response) = paginator.next().await {
        match response {
            Ok(Response {
                response: Some(resp),
                metadata: Some(_metadata),
            }) => responses.push(resp.output),
            _ => panic!("TODO(russell-teller): handle metadata-only responses"),
        }
    }

    responses
}
*/

#[derive(Serialize, Deserialize)]
struct GenerationConfig {
    max_output_tokens: u32,
    temperature: f32,
    top_p: u32,
    top_k: u32,
}

#[derive(Serialize, Deserialize)]
struct SafetySettings {
    harm_category_hate_speech: String,
    harm_category_dangerous_content: String,
    harm_category_sexually_explicit: String,
    harm_category_harassment: String,
}

async fn run() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let project_id = "symbolic-fire-413915";
    let location = "us-central1";
    let model_id = "gemini-pro-vision";
    let endpoint = format!("https://your-vertex-ai-endpoint/projects/{}/locations/{}/models/{}:generateContent", project_id, location, model_id);

    let generation_config = GenerationConfig {
        max_output_tokens: 2048,
        temperature: 0.4,
        top_p: 1,
        top_k: 32,
    };

    let safety_settings = json!({
        "HARM_CATEGORY_HATE_SPEECH": "BLOCK_MEDIUM_AND_ABOVE",
        "HARM_CATEGORY_DANGEROUS_CONTENT": "BLOCK_MEDIUM_AND_ABOVE",
        "HARM_CATEGORY_SEXUALLY_EXPLICIT": "BLOCK_MEDIUM_AND_ABOVE",
        "HARM_CATEGORY_HARASSMENT": "BLOCK_MEDIUM_AND_ABOVE",
    });

    let response = client.post(endpoint)
        .json(&json!({
            "generationConfig": generation_config,
            "safetySettings": safety_settings,
            // Additional parameters as required by the API
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let response_text = response.text().await?;
        println!("{}", response_text);
    } else {
        eprintln!("Failed to generate content");
    }

    Ok(())
}

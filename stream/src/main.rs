use std::process::Command;
use std::{fs::OpenOptions, time::SystemTime};
use std::env;
use std::io::Write;
use anyhow::Error;
use bytemuck::cast_slice;
// use ffmpeg_next::time::sleep;
use futures::SinkExt;
use hound::{WavSpec, WavWriter};
use metapower_framework::{log, service::llmchat_model::llmchat_grpc::{chat_svc_client::ChatSvcClient, SpeechToTextRequest}, LLMCHAT_GRPC_REST_SERVER};
use tempfile::NamedTempFile;
use tokio::time::sleep;
use warp::{filters::ws::Message, Filter};
use futures_util::stream::StreamExt;
use ffmpeg_next::{format::{self}, media};

#[tokio::main]
async fn main() {
    let _ = ffmpeg_next::init();
    let ws_route = warp::path("up")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_audio_stream_up)
        });
    let ws_route2 = warp::path("down")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_audio_stream_down)
        });
    let ws_route3 = warp::path("recorder")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_audio_stream_recorder)
        });

    let routes = ws_route.or(ws_route2).or(ws_route3);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 8040))
        .await;
}

async fn handle_audio_stream_down(websocket: warp::ws::WebSocket) {
}
async fn run_ffmpeg(iphone_video_file_name_mp4: String) -> Result<String, Error> {
    let temp_audio_file_name = format!(
        "/data/tmp/{}.mp3", uuid::Uuid::new_v4()
    );
    let ffmpeg_cmd = "/usr/bin/ffmpeg";
    
        Command::new(ffmpeg_cmd)
            .arg("-i")
            .arg(iphone_video_file_name_mp4)
            .arg("-async")
            .arg("1")
            .arg("-map")
            .arg("0:a")
            .arg(temp_audio_file_name.clone())
            .spawn()?;

        sleep(std::time::Duration::from_secs(4)).await;
        // Command::new(ffmpeg_cmd)
        //     .arg("-i")
        //     .arg(iphone_audio_file_name_mp4)
        //     .arg(temp_audio_file_name.clone())
        //     .spawn()?;

    Ok(temp_audio_file_name)
}

async fn do_speech_to_text(audio_file: String) -> Option<String>{
    if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
        let tts_request = tonic::Request::new(SpeechToTextRequest {
            audio_url: audio_file,
        });
        match client.speech_to_text(tts_request).await {
            Ok(response) => {
                let speech_text = response.get_ref().text.clone();
                log!("LLM Response: {:?}", speech_text);
                return Some(speech_text);
            }
            Err(e) => {
                log!("Error: {:?}", e);
            }
        }
    }
    None
}
async fn handle_audio_stream_up(websocket: warp::ws::WebSocket) {
    let (mut tx, mut rx) = websocket.split();
    let mut browser_type = "chrome";
    let mut temp_file: String;
    let temp_audio_file_name = format!(
        "/data/tmp/{}.wav", uuid::Uuid::new_v4()
    );

    while let Some(Ok(msg)) = rx.next().await {
        if msg.is_binary() {
            let webm_bytes = msg.as_bytes();
            let mut capture_audio_valid = false;

            if browser_type == "iphone" || browser_type == "macosx" {
                let iphone_video_file_name = format!(
                    "/data/tmp/{}.m4a", uuid::Uuid::new_v4()
                );
                println!("iphone audio save to {}", iphone_video_file_name);
                if let Ok(mut file) = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(iphone_video_file_name.clone()){
                        let _ = file.write_all(webm_bytes);
                }
                match run_ffmpeg(iphone_video_file_name).await{
                    Ok(wav_file) => {
                        if let Some(speech_text) = do_speech_to_text(wav_file.clone()).await{
                            tx.send(Message::text(speech_text)).await.unwrap();
                        }        
                    }
                    Err(e) => {
                        println!("convert mp4 to wav error: {}", e)
                    }
                }
                continue;
            }else{
                let webm_audio_file_name = format!(
                    "/data/tmp/{}.webm", uuid::Uuid::new_v4()
                );
                println!("webm audio save to {}", webm_audio_file_name);
                if let Ok(mut file) = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(webm_audio_file_name.clone()){
                        let _ = file.write_all(webm_bytes);
                }
                temp_file = webm_audio_file_name;
            }

            match format::input(&temp_file){
                Ok(mut ictx) => {
                    if let Some(audio_stream) = ictx.streams().best(media::Type::Audio){
                        let audio_stream_index = audio_stream.index();
                        let codec = ffmpeg_next::codec::context::Context::from_parameters(audio_stream.parameters()).unwrap();
                        let mut decoder = codec.decoder().audio().unwrap();
                        log!("Decoder format: {:?}", decoder.format());
                        log!("Decoder bitsrate: {:?}", decoder.bit_rate());

                        let spec = WavSpec {
                            channels: decoder.channels(),
                            sample_rate: decoder.rate(),
                            bits_per_sample: 16, // Assuming 16 bits per sample
                            sample_format: hound::SampleFormat::Int,
                        };
                        let mut wav_writer = WavWriter::create(temp_audio_file_name.clone(), spec).unwrap();

                        for (stream, packet) in ictx.packets() {
                            if stream.index() == audio_stream_index {
                                decoder.send_packet(&packet).unwrap();
                                let mut decoded = ffmpeg_next::util::frame::Audio::empty();
                                while decoder.receive_frame(&mut decoded).is_ok() {
                                    capture_audio_valid = true;
                                    let data = decoded.data(0); // Assuming the first plane for simplicity
                                    let f32_samples: &[f32] = cast_slice(data);

                                    // Convert samples from f32 to i16 and write to WAV
                                    let samples_i16: Vec<i16> = f32_samples.iter().map(|&s| (s * i16::MAX as f32) as i16).collect();
                                    for sample in samples_i16.iter() {
                                        if let Err(e) = wav_writer.write_sample(*sample){
                                            capture_audio_valid = false;
                                            println!("write wav file error: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error opening input file: {:?}", e);
                }
            }
            if capture_audio_valid {
                if let Some(speech_text) = do_speech_to_text(temp_audio_file_name.clone()).await{
                    if let Err(e) = tx.send(Message::text(speech_text)).await{
                        println!("send speech text error: {}",e);
                    }
                }
            }
        }
        if msg.is_text(){
            match msg.to_str().unwrap_or_default(){
                "ping" => {
                    match  tx.send(Message::text("pong")).await{
                        Ok(_) =>{}   
                        Err(e) =>{
                            println!("send heart beat failure: {}", e);
                        }
                    }
                    // println!("receive ping and send pong");
                }
                "iphone" => {
                    browser_type = "iphone";
                    println!("agent os is iphone");
                }
                "macosx" => {
                    browser_type = "macosx";
                    println!("agent os is macosx");
                }
                _ => {}
            }
        }
    }
}
async fn handle_audio_stream_recorder(websocket: warp::ws::WebSocket) {
    let (mut tx, mut rx) = websocket.split();

    while let Some(Ok(msg)) = rx.next().await {
        if msg.is_binary() {
            let mut capture_audio_valid = false;
            let temp_audio_file_name = format!(
                "/data/www/xfiles/{}.wav", uuid::Uuid::new_v4()
            );
            let temp_transcript_file_name = format!(
                "/data/tmp/{}.txt", uuid::Uuid::new_v4()
            );
            println!("temp_transcript_file_name: {:?}", temp_transcript_file_name);

            let webm_bytes = msg.as_bytes();
            let temp_file = match NamedTempFile::new(){
                Ok(mut temp_file) => {
                    match temp_file.write_all(webm_bytes){
                        Ok(_) => {
                            log!("Temp file written: {:?}", temp_file);
                        }
                        Err(e) => {
                            eprintln!("Error writing temp file: {:?}", e);
                            break;
                        }
                    }
                    temp_file
                }
                Err(e) => {
                    eprintln!("Error creating temp file: {:?}", e);
                    break;
                }
            };
            match format::input(&temp_file.path().to_str().unwrap()){
                Ok(mut ictx) => {
                    if let Some(audio_stream) = ictx.streams().best(media::Type::Audio){
                        // temp_file.keep().unwrap();
                        let audio_stream_index = audio_stream.index();
                        let codec = ffmpeg_next::codec::context::Context::from_parameters(audio_stream.parameters()).unwrap();
                        let mut decoder = codec.decoder().audio().unwrap();
                        log!("Decoder format: {:?}", decoder.format());
                        log!("Decoder bitsrate: {:?}", decoder.bit_rate());

                        let spec = WavSpec {
                            channels: decoder.channels(),
                            sample_rate: decoder.rate(),
                            bits_per_sample: 16, // Assuming 16 bits per sample
                            sample_format: hound::SampleFormat::Int,
                        };
                        let mut wav_writer = WavWriter::create(temp_audio_file_name.clone(), spec).unwrap();

                        for (stream, packet) in ictx.packets() {
                            if stream.index() == audio_stream_index {
                                decoder.send_packet(&packet).unwrap();
                                let mut decoded = ffmpeg_next::util::frame::Audio::empty();
                                while decoder.receive_frame(&mut decoded).is_ok() {
                                    capture_audio_valid = true;
                                    let data = decoded.data(0); // Assuming the first plane for simplicity
                                    let f32_samples: &[f32] = cast_slice(data);

                                    // Convert samples from f32 to i16 and write to WAV
                                    let samples_i16: Vec<i16> = f32_samples.iter().map(|&s| (s * i16::MAX as f32) as i16).collect();
                                    for sample in samples_i16.iter() {
                                        if let Err(e) = wav_writer.write_sample(*sample){
                                            capture_audio_valid = false;
                                            println!("write wav file error: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error opening input file: {:?}", e);
                }
            }
            if capture_audio_valid {
                if let Ok(mut client) = ChatSvcClient::connect(LLMCHAT_GRPC_REST_SERVER).await {
                    let tts_request = tonic::Request::new(SpeechToTextRequest {
                        audio_url: temp_audio_file_name.clone(),
                    });
                    println!("tts_request: {:?}", tts_request);
                    match client.speech_to_text(tts_request).await {
                        Ok(response) => {
                            let speech_text = response.get_ref().text.clone();
                            // log!("LLM Response: {:?}", speech_text);
                            match OpenOptions::new().create(true).write(true).truncate(true).open(temp_transcript_file_name.clone()){
                                Ok(mut file) => {
                                    match file.write_all(speech_text.as_bytes()){
                                        Ok(_) => {
                                            log!("Transcript file written: {:?}", temp_transcript_file_name);
                                        }
                                        Err(e) => {
                                            eprintln!("Error writing transcript file: {:?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error creating transcript file: {:?}", e);
                                }
                            }
                            if let Err(e) = tx.send(Message::text(temp_transcript_file_name)).await{
                                println!("send speech file error: {}",e);
                            }
                        }
                        Err(e) => {
                            log!("Error: {:?}", e);
                        }
                    }
                }
            }
        }
    }
}

use std::time::Duration;
use anyhow::{self, Ok};
use metapower_framework::METAPOWER_BROKER;

extern crate paho_mqtt as mqtt;

const METAPOWER_CLIENT:&str = "MetaPowerAgent";
const METAPOWER_TOPICS:&[&str] = &[
    "/metapower/text/done", 
    "/metapower/media/done",
    "/metapower/text/done", 
    "/metapower/media/done",
    "/metapower/media/done",
    "/metapower/media/done",
    "/metapower/media/done",
    "/metapower/text/done", 
    "/metapower/media/done",
    "/metapower/text/done", 
    "/metapower/text/done", 
    "/metapower/media/done", 
    "/metapower/text/done", 
    "/metapower/nothing/done",
];
const METAPOWER_MESSAGES: &[&str] = &[
    "translate/",
    "translatevoice/",
    "chat/",
    "voice/",
    "asset3d/",
    "music/",
    "pic/",
    "action/", 
    "video/", 
    "doc/", 
    "sensor/",
    "sensorvoice/",
    "groupchat/",
    "nothing/"
];
const METAPOWER_QOS: i32 = 0;

#[derive(Debug, Clone, Copy)]
pub enum NotifyWhat {
    TranslateDone = 0,
    TranslateVoiceDone,
    ChatTextDone,
    ChatVoiceDone,
    ThreeDModelDone,
    MusicDone,
    PicDone,
    ActionDone,
    VideoDone,
    DocDone,
    SensorDone,
    SensorVoiceDone,
    GroupchatDone,
    NothingDone,
}

pub fn notify_model_task_done(what: NotifyWhat, path: String, session_id: String) -> Result<(), anyhow::Error> {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(METAPOWER_BROKER)
        .client_id(METAPOWER_CLIENT.to_string())
        .finalize();

    let cli = mqtt::Client::new(create_opts)?;

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    cli.connect(conn_opts)?;
    let message = METAPOWER_MESSAGES[what as usize].to_string() + &path; 
    println!("notify mqtt client: {}", message);

    let msg = mqtt::Message::new(
        METAPOWER_TOPICS[what as usize].to_string() + "/" + &session_id, 
        message, 
        METAPOWER_QOS
    );
    cli.publish(msg)?;

    cli.disconnect(None)?;

    Ok(())

}

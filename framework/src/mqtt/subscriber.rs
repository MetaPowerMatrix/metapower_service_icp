use std::io::Write;
use std::time::SystemTime;
use std::env;
use std::{
    thread,
    time::Duration
};

use anyhow::Error;

use crate::{log, mqtt::{METAPOWER_CLIENT, METAPOWER_QOS}, METAPOWER_BROKER};
extern crate paho_mqtt as mqtt;

// Reconnect to the broker when connection is lost.
fn try_reconnect(cli: &mqtt::Client) -> bool
{
    println!("Connection lost. Waiting to retry connection");
    for _ in 0..12 {
        thread::sleep(Duration::from_millis(5000));
        if cli.reconnect().is_ok() {
            println!("Successfully reconnected");
            return true;
        }
    }
    println!("Unable to reconnect after several attempts.");
    false
}

// Subscribes to multiple topics.
fn subscribe_topics(cli: &mqtt::Client, topics: &[&str], qos: &[i32]) -> Result<(), Error> {
    if let Ok(resp) = cli.subscribe_many(topics, qos){
        println!("Subscribed to topics: {:?}", resp);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to subscribe to topics"))
    }
}

pub fn recv_client_done(topic: String, message: String) {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(METAPOWER_BROKER)
        .client_id(METAPOWER_CLIENT.to_string())
        .finalize();

    // Create a client.
    if let Ok(cli) = mqtt::Client::new(create_opts) {
        let rx = cli.start_consuming();
        let lwt = mqtt::MessageBuilder::new()
            .topic(topic.clone())
            .payload(message)
            .finalize();
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(false)
            .will_message(lwt)
            .finalize();
        match cli.connect(conn_opts) {
            Ok(_) => {
            }
            Err(e) => {
                log!("Unable to connect:\n\t{:?}", e);
            } 
        }
        let _ = subscribe_topics(&cli, &[&topic], &[METAPOWER_QOS]);
        println!("Processing requests...");
        for msg in rx.iter() {
            if let Some(msg) = msg {
                log!("{}", msg);
            }
            else if !cli.is_connected() {
                if try_reconnect(&cli) {
                    log!("Resubscribe topics...");
                    let _ = subscribe_topics(&cli, &[&topic], &[METAPOWER_QOS]);
                } else {
                    break;
                }
            }
        }
    }
}

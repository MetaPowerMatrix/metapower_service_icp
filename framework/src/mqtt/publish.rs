use std::io::Write;
use std::time::SystemTime;
use std::env;
use std::time::Duration;
use crate::{log, mqtt::{METAPOWER_CLIENT, METAPOWER_QOS}, METAPOWER_BROKER};
extern crate paho_mqtt as mqtt;

pub fn publish_battery_actions(topic: String, message: String) -> Result<(), anyhow::Error> {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(METAPOWER_BROKER)
        .client_id(METAPOWER_CLIENT.to_string())
        .finalize();

    match mqtt::Client::new(create_opts){
        Ok(cli) => {
            let conn_opts = mqtt::ConnectOptionsBuilder::new()
                .keep_alive_interval(Duration::from_secs(20))
                .clean_session(true)
                .finalize();
    
            match cli.connect(conn_opts){
                Ok(_) => {
                    let msg = mqtt::Message::new(
                        &topic,
                        message.clone(), 
                        METAPOWER_QOS
                    );
                    match cli.publish(msg){
                        Ok(_) => { 
                            // log!("publish to {} with {}", &topic, &message); 
                        }
                        Err(e) => { log!("publish Error: {}", e); }
                    }
                    let _ = cli.disconnect(None);        
                }
                Err(e) => { log!("connect Error: {}", e); }
            }
        }
        Err(e) => { log!("create client Error: {}", e); }
    }



    Ok(())

}

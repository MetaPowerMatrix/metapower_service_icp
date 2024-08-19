pub mod id;
pub mod runner;
pub mod reverie;

use std::net::SocketAddr;

use metapower_framework::{service::metapowermatrix_battery_mod::battery_grpc::meta_power_matrix_battery_svc_server::MetaPowerMatrixBatterySvcServer, BATTERY_GRPC_SERVER, BATTERY_GRPC_SERVER_PORT_START};
use tonic::transport::Server;
use clap::Parser;
use crate::id::MetaPowerMatrixBatteryService;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    identity: String,

    #[arg(short, long)]
    sn: i64,

    #[arg(short, long, default_value_t = false)]
    daemon: bool
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let id = args.identity;
    let sn = args.sn;

    let battey_listen_address = format!("{}:{}", BATTERY_GRPC_SERVER, sn + BATTERY_GRPC_SERVER_PORT_START);
    let address :SocketAddr = battey_listen_address.parse().unwrap();
    let battery_service = MetaPowerMatrixBatteryService::new(id.clone());

    tokio::task::spawn(async move {
        runner::BatteryRunner::new(id, sn).run_loop().await;
     });
 
    // println!("metapower battery grpc service @ {}", address);
    // Server::builder()
    //     .add_service(MetaPowerMatrixBatterySvcServer::new(battery_service))
    //     .serve(address)
    //     .await?;

    Ok(())
}

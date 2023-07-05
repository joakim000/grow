#![feature(error_in_core)]
#![allow(unused)]

mod cmd;
mod hardware;
mod init;

use core::error::Error;
use core::time::Duration;
use std::sync::Arc;

use tokio::time::sleep;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio_util::sync::CancellationToken;
pub type HouseMutex = Arc<TokioMutex<grow::House>>;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // construct a subscriber that prints formatted traces to stdout
    // let subscriber = tracing_subscriber::FmtSubscriber::new();
    // use that subscriber to process traces emitted after this point
    // tracing::subscriber::set_global_default(subscriber)?;
    // tracing_subscriber::fmt::try_init()?;

    // console_subscriber::init();
    // console_subscriber::ConsoleLayer::builder()
    // .retention(Duration::from_secs(60))
    // .server_addr(([127, 0, 0, 1], 5555))
    // .server_addr(([192, 168, 0, 106], 9090))
    // .init();
    // println!("Hej hopp!");

    let (shutdown_send, mut shutdown_recv) = mpsc::unbounded_channel::<bool>();
    let cancel_token = CancellationToken::new();

    let (house, manager) = init::init(cancel_token.clone()).await;
    let _cmd_task =
        cmd::manual_cmds(house.clone(), manager.clone(), shutdown_send);

    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = shutdown_recv.recv() => {},
    }

    // Cleanup
    cancel_token.cancel();
    println!("Start shutdown procedure");
    // cmd_task.unwrap().abort();
    sleep(Duration::from_millis(1000)).await;

    // println!("Cleanup successful");
    Ok(())
}

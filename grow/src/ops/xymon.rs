use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use core::error::Error;
use std::time::Duration;
use crate::ops::display::Indicator;
use crate::zone::ZoneDisplay;
use crate::zone::ZoneKind::*;
use std::thread;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct XymonSettings {
   pub port: u16,
   pub host: String,
   pub client: String,
}
macro_rules! xymon_match {
    ($x:ident, $y:ident, [$( $variant:tt ),+] ) => {
       match $x {
        $(
        ZoneDisplay::$variant {id, info} => {
            let ind = match info.indicator {
                Indicator::Blue => "clear",
                Indicator::Green => "green",
                Indicator::Yellow => "yellow",
                Indicator::Red => "red",
            };
            let info_msg = match &info.msg {
                Some(info_msg) => &info_msg,
                None => "No message",
            };
            $y.push_str(&format!("{:?}_{} {} &{} {}\nTimestamp: {}", $variant, &id, ind, ind, info_msg, info.changed));
           }
        )+
       }
   }
}
/// Send status update to Xymon server, format:
/// status[+LIFETIME][/group:GROUP] HOSTNAME.TESTNAME COLOR <additional text>
pub async fn send_status(data: &ZoneDisplay, x: Arc<XymonSettings>) -> Result<(), Box<dyn Error>> {
    let xymon = format!("{}:{}", x.host, x.port);
    let mut stream = match TcpStream::connect(&xymon).await {
        Ok(stream) =>  stream,
        Err(e) => return Err(Box::new(e))
    };
    let mut xymon_status = format!("status+1h {}.", x.client);
    xymon_match!(data, xymon_status, [Water, Air, Light, Aux, Tank, Pump, Arm]);
    match stream.write_all(xymon_status.as_bytes()).await {
        // Ok(_) => {println!("Sent to Xymon: {:?}", &xymon_status);},
        Ok(_) => (),
        Err(e) => return Err(Box::new(e))
    }
    let _ = stream.shutdown();
    // thread::sleep(Duration::from_millis(10));
    
    Ok(())
}

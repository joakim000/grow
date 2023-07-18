use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use core::error::Error;
use std::time::Duration;
use crate::ops::display::Indicator;
use crate::zone::ZoneDisplay;
use crate::zone::ZoneKind::*;
use std::thread;

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

pub async fn send_status(data: &ZoneDisplay) -> Result<(), Box<dyn Error>> {
    // status[+LIFETIME][/group:GROUP] HOSTNAME.TESTNAME COLOR <additional text>
    const XYMON_PORT: u16 = 1984;
    const XYMON_HOST: &str = "192.168.1.81";
    const XYMON_CLIENT: &str = "greenhouse";
    // const XYMON_HOST: &str = "172.168.1.81";
    // const XYMON_CLIENT: &str = "greenhouse.iris";

    let xymon = format!("{}:{}", XYMON_HOST, XYMON_PORT);
    let mut stream = match TcpStream::connect(&xymon).await {
        Ok(stream) =>  stream,
        Err(e) => return Err(Box::new(e))
    };
    // let mut xymon_status = format!("status+1h/group::gardeners {}.", XYMON_CLIENT);
    let mut xymon_status = format!("status+1h {}.", XYMON_CLIENT);
    xymon_match!(data, xymon_status, [Water, Air, Light, Aux, Tank, Pump, Arm]);
    match stream.write_all(xymon_status.as_bytes()).await {
        Ok(_) => {println!("====== Sent to Xymon: {:?}", &xymon_status);},
        // Ok(_) => (),
        Err(e) => return Err(Box::new(e))
    }
    let _shut = stream.shutdown();
    thread::sleep(Duration::from_millis(100));
    
    Ok(())
}


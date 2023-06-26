use crate::House;
use crate::Zone;
use crate::HouseMutex;
use crate::zone::*;
use crate::TIME_OFFSET;

use tokio::time::sleep as sleep;
use core::error::Error;
use core::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use std::sync::Mutex;
use std::sync::Arc;
use core::fmt::Debug;
use parking_lot::RwLock;
use time::OffsetDateTime;
// use time::macros::offset;
use time::format_description::well_known::{Rfc3339, Rfc2822};
use super::OpsChannelsTx;
use super::OpsChannelsRx;
use super::SysLog;
use super::TextDisplay;
use super::Board;
use super::remote;
use super::remote::*;
use crate::zone::irrigation::arm::Arm;
use super::input::ButtonPanel;

// pub struct Runner {
//     house: Vec<Zone>,

// }


// pub fn create_manager_channels() -> (TxToManagerChannels, RxInManagerChannels) {
//     let (irrigation_tx, mut irrigation_rx) = mpsc::channel::<(u8, f32)>(128);

//     let tx_to_manager = TxToManagerChannels {
//         irrigation: irrigation_tx,
//     };

//     let rx_in_manager = RxInManagerChannels {
//         irrigation: irrigation_rx,
//     };

//     (tx_to_manager, rx_in_manager)
// }
// #[derive(Clone, Debug, )]
// pub struct TxToManagerChannels {
//     pub irrigation: mpsc::Sender<(u8, f32)>,
// }
// #[derive(Debug, )]
// pub struct RxInManagerChannels {
//     pub irrigation: mpsc::Receiver<(u8, f32)>,
// }

enum RcModeExit {
    Confirm,
    Cancel,
    SwitchFromOpsMode,
    SwitchFromPositionMode,
    ElseExit
}


#[derive(Debug, )]
pub struct Manager {
    house: HouseMutex,
    board: Box<dyn Board>,
    display: Box<dyn TextDisplay>,
    remote: Box<dyn RemoteControl>,
    buttons: Box<dyn ButtonPanel>,
    log_enable: Option<watch::Sender<bool>>,
    // ops_tx: Option<OpsChannelsTx>,
    ops_tx: OpsChannelsTx,
}
impl Manager {
    pub fn new(house: HouseMutex, board: Box<dyn Board>, 
            display: Box<dyn TextDisplay>, remote: Box<dyn RemoteControl>,
            buttons: Box<dyn ButtonPanel>, ops_tx: OpsChannelsTx,
        ) -> Self {     
        Self {
            house,
            board, 
            display,
            remote,
            buttons,
            log_enable: None,
            // ops_tx: None, 
            ops_tx,
        }
    }
   

    pub fn init(&mut self, mut from_zones: ZoneChannelsRx, ops_tx: OpsChannelsTx,  mut ops_rx:OpsChannelsRx,
            selfmutex: crate::ManagerMutex) -> () {
        let (log_enable_tx, mut log_enable_rx) = tokio::sync::watch::channel(true);
        self.log_enable = Some(log_enable_tx);

        let manager_mutex = selfmutex.clone();
        let to_log = ops_tx.syslog.clone();
        let log_handler = tokio::spawn(async move {
            let mut enabled = *log_enable_rx.borrow(); 
            to_log.send(SysLog {msg: format!("Spawned log handler")}).await;
            loop {
                tokio::select! {
                    Ok(()) = log_enable_rx.changed() => {
                        enabled = *log_enable_rx.borrow();
                    }
                    Some(data) = ops_rx.syslog.recv() => {
                        if true { 
                            let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                            println!("{} {:?}", now.format(&Rfc2822).expect("Time formatting error"), &data); 
                        }
                    }
                    Some(data) = from_zones.zonelog.recv() => {
                        if enabled { 
                            let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                            println!("{} {:?}", now.format(&Rfc2822).expect("Time formatting error"), &data); 
                        }
                    }
                    Ok(data) = from_zones.zonestatus.recv() => {
                        { manager_mutex.lock().await.update_board().await;}
                    }
                    else => { break }
                };
            }
        });

        let to_log = ops_tx.syslog.clone();
        let house_mutex = self.house.clone();
        let zoneupdate_handler = tokio::spawn(async move {
            to_log.send(SysLog {msg: format!("Spawned zoneupdate handler")}).await;
            while let Some(data) = from_zones.zoneupdate.recv().await {
                let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                println!("{:?}`Manager recv: {:?}", now, data);
                
                let mut lock = house_mutex.lock().await;
                match data {
                    ZoneUpdate::Irrigation { id, moisture } => {
                        let settings = lock.get_irrigation_settings(id).expect("Settings not found");
                        let movement = settings.position.expect("Position not found");
                        let _ = lock.arm_goto(movement.arm_id, movement.x, movement.y, movement.z);  // TODO check result
                        sleep(Duration::from_secs(2)).await;  // TODO wait for arm position to be correct
                        
                        let _ = lock.pump_run(settings.pump_id);    // TODO check result
                        sleep(settings.pump_time).await;  
                        let _ = lock.pump_stop(settings.pump_id);   // TODO check result
                    }
                }
                

            }
        });
    } 
    pub async fn update_board(&mut self) {
        let mut lock = self.house.lock().await;
        let all_ds = lock.collect_display_status();
        self.board.set(all_ds);
    }

    pub fn log_enable(&self, val: bool) {
        match &self.log_enable {
            Some(sender) => {
                sender.send(val);
            }
            None => {}
        }
    }

    pub async fn blink(&mut self) -> (Result<(), Box<dyn Error>>) {
        // let board = self.board.clone();
        // tokio::spawn(async move {
            self.board.blink_all(Duration::from_millis(500), Duration::from_secs(1));
        // });
        // self.board.blink_all(Duration::from_millis(500), Duration::from_secs(1)).await;
        
        
        Ok(())
    }

    pub async fn position_from_rc(&mut self, zid: u8) -> Option<(i32, i32, i32)> {
        // let (rc_tx, mut rc_rx) = broadcast::channel::<RcInput>(8);
        let (rc_tx, mut rc_rx) = mpsc::channel::<RcInput>(8);
        let _ = self.remote.init(rc_tx).await;
        println!("Back from  remote.init()");

        let x = 0;
        let y = 0;
        let z = 0;

        let to_log = self.ops_tx.syslog.clone();
        let mutex = self.house.clone();
        let position_finder = tokio::spawn(async move {
            // println!(" =================== Spawned position finder task");
            to_log.send(SysLog {msg: format!("Spawned position finder")}).await;
            let mut arm_o: Option<&(dyn Arm + '_)> = None;

            // {
                let mut lock = mutex.lock().await;
                for z in lock.zones() {
                    match z {
                        Zone::Arm {id , settings:_, status:_, interface, runner: _} if id == &zid => {
                            // let arm = Arc::new(interface.arm.as_deref().expect("Interface not found"));
                            arm_o = Some(interface.arm.as_deref().expect("Interface not found")); 
                        }
                        _ => continue
                    }
                }

                { println!("House lock status b4 loop: {:?}", mutex.try_lock()); }

                let arm = arm_o.expect("Zone not found");
            // }
            loop {
                    tokio::select! {
                        // _ = tx.closed() => { 
                        //     println!("Managers' RC receiver dropped, exit RC feedback task");
                        //     break; 
                        // }
                
                        Some(data) = rc_rx.recv() => {
                            match data {
                                RcInput::LeftUp | RcInput::RightUp => {
                                    arm.stop_x();
                                }
                                RcInput::DownUp | RcInput::UpUp => {
                                    arm.stop_y();
                                }
                                RcInput::Left => {
                                    arm.start_x(-20);
                                }
                                RcInput::Right => {
                                    arm.start_x(20);
                                }
                                RcInput::Up => {
                                    arm.start_y(80);
                                }
                                RcInput::Down => {
                                    arm.start_y(-80);
                                }
                                RcInput::Confirm => {
                                    break RcModeExit::Confirm;
                                }
                                RcInput::Back => {
                                    break RcModeExit::Cancel;
                                }
                                RcInput::Mode => {
                                }
                                RcInput::Exit => {
                                    break RcModeExit::Cancel;   
                                }
                            
                            
                                RcInput::ConfirmUp => {
                                }
                                RcInput::DownUp | RcInput::UpUp => {
                                }
                                RcInput::BackUp => {
                                }
                                RcInput::ModeUp => {
                                }
                            }
                        }
                        else => { break RcModeExit::ElseExit; }
                    };
                }
                // { println!("House lock status after loop: {:?}", mutex.try_lock()); }

            // while let Some(data) = rc_rx.recv().await {
            //     println!("Runner RC input: {:?} ", data,);
            //     match data {
            //         RcInput::LeftUp | RcInput::RightUp => {
            //             arm.stop_x();
            //         }
            //         RcInput::DownUp | RcInput::UpUp => {
            //             arm.stop_y();
            //         }
            //         RcInput::Left => {
            //             arm.start_x(-20);
            //         }
            //         RcInput::Right => {
            //             arm.start_x(20);
            //         }
            //         RcInput::Up => {
            //             arm.start_y(80);
            //         }
            //         RcInput::Down => {
            //             arm.start_y(-80);
            //         }
            //         RcInput::Confirm => {
            //             break;
            //         }
            //         RcInput::Back => {
            //         }
            //         RcInput::Mode => {
            //         }
            //         RcInput::Exit => {
            //             break;   // Select loop instead so we can use break value?
            //         }
                   
                
            //         RcInput::ConfirmUp => {
            //         }
            //         RcInput::DownUp | RcInput::UpUp => {
            //         }
            //         RcInput::BackUp => {
            //         }
            //         RcInput::ModeUp => {
            //         }
            //     }
            // }

            // println!("End Manager RC while-loop");
            // println!("Receiever: {:?}", &rc_rx);
        });
        { println!("House lock status after spawn def: {:?}", self.house.try_lock()); }

        let exit_kind = position_finder.await.expect("Position finder error");
        
        { println!("House lock status after finder task exit: {:?}", self.house.try_lock()); }

        self.ops_tx.syslog.send(SysLog {msg: format!("Exit position finder")}).await;
        // println!("End Manager RC-task");

        // Get current position from house after exit:
        let mut house = self.house.lock().await;
        let pos = house.arm_position(1).expect("Error getting arm position from house");
        self.ops_tx.syslog.send(SysLog {msg: format!("Selected position: {:?}", &pos)}).await;
        println!("selected pos: {:?}", &pos);
        
        match exit_kind {
            RcModeExit::Confirm => {
                self.ops_tx.syslog.send(SysLog {msg: format!("Selected position: {:?}", &pos)}).await;
                Some(pos)
            },
            RcModeExit::Cancel => {
                self.ops_tx.syslog.send(SysLog {msg: format!("Position finder cancelled: {:?}", &pos)}).await;
                None
            },
            RcModeExit::SwitchFromPositionMode => {
                self.ops_tx.syslog.send(SysLog {msg: format!("Position finder mode switch")}).await;
                None
            },
            RcModeExit::SwitchFromOpsMode => {
                self.ops_tx.syslog.send(SysLog {msg: format!("Position finder unexpected exit (wrong mode)")}).await;
                None
            },
            RcModeExit::ElseExit => {
                self.ops_tx.syslog.send(SysLog {msg: format!("Position finder unexpected exit (select else)")}).await;
                None
            },
            
            _ => { None }
        }
    }

}




// // (house: House )

// impl Runner {
//     pub fn new() -> Self {
//         Self {
//             house: Vec::new(),

//         }
//     }

//     pub fn init(&mut self) {
//         // Create house from conf
//         self.house = super::conf::Conf::read_test_into_vec();
//         // Init hw
//         for z in &self.house {
//             match z {
//                 Zone::Air{id, set, status , interface} => {

//                 },
//                 _ => ()
//             }
//         }

//         // Start run
//     }

//     // Running: light scheduler, rx: zone::sensortype, ui (buttons, display, indicators)

//     pub async fn run() -> tokio::task::JoinHandle<()> {

//         tokio::spawn(async move {

//             tokio::time::sleep(Duration::from_secs(1)).await;

//         })
//     }

//     pub fn remote_set_irrigation_position() {}

//     pub async fn shutdown() -> tokio::task::JoinHandle<()> {
//         // Save: LPU position data, indicator statuses?

//         // Reset pins

//         // Disconnect LPU

//         tokio::spawn(async move {

//             tokio::time::sleep(Duration::from_secs(1)).await;

//         })

//     }
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub struct Settings {}

// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub struct Status {}

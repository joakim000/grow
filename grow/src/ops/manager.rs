use crate::zone::*;
use crate::House;
use crate::HouseMutex;
use crate::Zone;
use crate::TIME_OFFSET;

use core::error::Error;
use core::fmt::Debug;
use core::time::Duration;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::Mutex;
use time::OffsetDateTime;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
// use time::macros::offset;
use super::input::ButtonPanel;
use super::remote;
use super::remote::*;
use super::Board;
use super::OpsChannelsRx;
use super::OpsChannelsTx;
use super::SysLog;
use super::TextDisplay;
use crate::zone::water::arm::Arm;
use time::format_description::well_known::{Rfc2822, Rfc3339};

#[derive(Debug)]
enum RcModeExit {
    Confirm,
    Cancel,
    SwitchFromOpsMode,
    SwitchFromPositionMode,
    ElseExit,
}
#[derive(Debug)]
pub struct Manager {
    house: HouseMutex,
    board: Box<dyn Board>,
    display: Box<dyn TextDisplay>,
    remote: Box<dyn RemoteControl>,
    buttons: Box<dyn ButtonPanel>,
    log_enable: Option<watch::Sender<bool>>,
    status_enable: Option<watch::Sender<bool>>,
    // ops_tx: Option<OpsChannelsTx>,
    ops_tx: OpsChannelsTx,
    zone_tx: ZoneChannelsTx,
}
impl Manager {
    pub fn new(
        house: HouseMutex,
        board: Box<dyn Board>,
        display: Box<dyn TextDisplay>,
        remote: Box<dyn RemoteControl>,
        buttons: Box<dyn ButtonPanel>,
        ops_tx: OpsChannelsTx,
        zone_tx: ZoneChannelsTx,
    ) -> Self {
        Self {
            house,
            board,
            display,
            remote,
            buttons,
            log_enable: None,
            status_enable: None,
            ops_tx,
            zone_tx,
        }
    }

    pub fn init(
        &mut self,
        mut from_zones: ZoneChannelsRx,
        // ops_tx: OpsChannelsTx,
        mut ops_rx: OpsChannelsRx,
        selfmutex: crate::ManagerMutex,
    ) -> () {
        let (log_enable_tx, mut log_enable_rx) = tokio::sync::watch::channel(false);
        self.log_enable = Some(log_enable_tx);
        let (status_enable_tx, mut status_enable_rx) = tokio::sync::watch::channel(false);
        self.status_enable = Some(status_enable_tx);

        let manager_mutex = selfmutex.clone();
        let to_log = self.ops_tx.syslog.clone();

        let log_handler = tokio::spawn(async move {
            let mut log_enabled = *log_enable_rx.borrow();
            let mut status_enabled = *status_enable_rx.borrow();
            to_log
                .send(SysLog {
                    msg: format!("Spawned log handler"),
                })
                .await;
            loop {
                tokio::select! {
                    Ok(()) = log_enable_rx.changed() => {
                        log_enabled = *log_enable_rx.borrow();
                        status_enabled = *status_enable_rx.borrow();
                    }
                    Some(data) = ops_rx.syslog.recv() => {
                        if true {
                            let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                            println!("{} {:?}", now.format(&Rfc2822).expect("Time formatting error"), &data);
                        }
                    }
                    Some(data) = from_zones.zonelog.recv() => {
                        if log_enabled {
                            let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                            println!("{} {}", now.format(&Rfc2822).expect("Time formatting error"), &data);
                        }
                    }
                    Ok(data) = from_zones.zonestatus.recv() => {
                        if status_enabled {
                            let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                            println!("{} {}", now.format(&Rfc2822).expect("Time formatting error"), &data);
                        }
                        {
                            manager_mutex.lock().await.update_board().await;
                        }
                    }
                    else => { break }
                };
            }
        });

        self.display.init(self.zone_tx.zonestatus.subscribe(), 
                self.ops_tx.syslog.clone() );

        // self.display.set(self.zone_tx.zonestatus.subscribe());

        let to_log = self.ops_tx.syslog.clone();
        let house_mutex = self.house.clone();
        let zoneupdate_handler = tokio::spawn(async move {
            to_log
                .send(SysLog {
                    msg: format!("Spawned zoneupdate handler"),
                })
                .await;
            while let Some(data) = from_zones.zoneupdate.recv().await {
                let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                // println!("{:?}`Manager recv: {:?}", now, data);

                let mut lock = house_mutex.lock().await;
                match data {
                    ZoneUpdate::Water { id, moisture } => {
                        let settings = lock
                            .get_water_settings(id)
                            .expect("Settings not found");
                        let movement = settings.position; //.expect("Position not found");
                        let _ = lock.arm_goto(movement.arm_id, movement.x, movement.y, movement.z); // TODO check result
                        sleep(Duration::from_secs(2)).await; // TODO wait for arm position to be correct

                        let _ = lock.pump_run(settings.pump_id); // TODO check result
                        sleep(settings.pump_time).await;
                        let _ = lock.pump_stop(settings.pump_id); // TODO check result
                    }
                }
            }
        });
    }
    pub async fn update_board(&mut self) {
        let mut lock = self.house.lock().await;
        let all_ds = lock.collect_display_status();
        self.board.set(all_ds).await;
    }

    pub fn log_enable(&self, val: bool) {
        match &self.log_enable {
            Some(sender) => {
                sender.send(val);
            }
            None => {}
        }
    }
    pub fn status_enable(&self, val: bool) {
        match &self.status_enable {
            Some(sender) => {
                sender.send(val);
            }
            None => {}
        }
    }

    pub async fn blink(&mut self) -> (Result<(), Box<dyn Error>>) {
        self.board
            .blink_all(Duration::from_millis(500), Duration::from_secs(1));

        Ok(())
    }

    pub async fn position_from_rc(&mut self, zid: u8) -> Option<(i32, i32, i32)> {
        let (rc_tx, mut rc_rx) = mpsc::channel::<RcInput>(64);
        let cancel = CancellationToken::new();
        let _ = self.remote.init(rc_tx, cancel.clone()).await;

        // let x = 0;
        // let y = 0;
        // let z = 0;

        let to_log = self.ops_tx.syslog.clone();
        let mutex = self.house.clone();
        let position_finder = tokio::spawn(async move {
            to_log
                .send(SysLog {
                    msg: format!("Spawned position finder"),
                })
                .await;
            let mut arm_o: Option<Arc<&(dyn Arm + '_)>> = None;
            // let mut arm_o: Option<Arc<Box<(dyn Arm + '_)>>> = None;
            // let mut arm_o: Option<Arc<&Box<dyn Arm>>> = None;
            let arm: Arc<&(dyn Arm + '_)>;
            
            // let arm: Arc<&(dyn Arm + '_)>;

            // let arm_clone: Arc<&(dyn Arm + '_)>;
            // let mut arm_o: Option<&(dyn Arm + '_)> = None;
            // let arm: &dyn Arm;
            {
                let mut lock = mutex.lock().await;
                for z in lock.zones() {
                    match z {
                        Zone::Arm {
                            id,
                            settings: _,
                            status: _,
                            interface,
                            runner: _,
                        } if id == &zid => {
                            // let arm = Arc::new(interface.arm.as_deref().expect("Interface not found"));
                            // arm_o = Some(interface.arm.as_deref().expect("Interface not found"));
                            arm_o = Some(Arc::new(interface.arm.as_deref().expect("Interface not found")));
                            // arm_o = Some(Arc::new(Box::new(*interface.arm.as_deref().expect("Interface not found"))));

                            // arm_o = Some(Arc::new(Box::new(*interface.arm.as_deref().expect("Interface not found"))));
                            // arm_o = Some(Arc::new(interface.arm.as_ref().expect("Interface not found")));
                            // let foo = Some(Arc::new(interface.arm.as_ref().expect("Interface not found")));
                        }
                        _ => continue,
                    }
                }

                // {
                //     println!("House lock status b4 loop: {:?}", mutex.try_lock());
                // }

                // arm = arm_o.expect("Zone not found");
                let arm = arm_o.expect("Zone not found");
                // arm = arm_original.clone();
            

                loop {
                    tokio::select! {
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
            }  
        });
        
        let exit_kind = position_finder.await.expect("Position finder error");
        println!("RC mode exit kind: {:?}", &exit_kind);
        self.ops_tx
            .syslog
            .send(SysLog {
                msg: format!("Exit position finder"),
            })
            .await;
        
        // Get current position from house after exit:
        let pos: (i32, i32, i32);
        {
            let mut house = self.house.lock().await;
            pos = house
            .arm_position(1)
            .expect("Error getting arm position from house");
        }

        /// Exit mode from RC-loop determines what to do next
        match exit_kind {
            RcModeExit::Confirm => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!("Selected position: {:?}", &pos),
                    })
                    .await;
                // println!("Req house lock for set water pos");
                self.house.lock().await.set_water_position(zid, pos);
                // println!("Water pos NOT set");
                Some(pos)
            }
            RcModeExit::Cancel => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!("Position finder cancelled: {:?}", &pos),
                    })
                    .await;
                None
            }
            RcModeExit::SwitchFromPositionMode => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!("Position finder mode switch"),
                    })
                    .await;
                None
            }
            RcModeExit::SwitchFromOpsMode => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!("Position finder unexpected exit (wrong mode)"),
                    })
                    .await;
                None
            }
            RcModeExit::ElseExit => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!("Position finder unexpected exit (select else)"),
                    })
                    .await;
                None
            }

            _ => None,
        }
    }
}


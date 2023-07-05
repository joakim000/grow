use crate::zone::water::arm::ArmCmd;
use crate::zone::water::arm::ArmState;
use crate::zone::*;
use crate::House;
use crate::HouseMutex;
use crate::Zone;
use crate::TIME_OFFSET;
use super::display::format_time;

use super::display::Indicator;
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
use super::io::{ButtonPanel, ButtonInput, TextDisplay, Board};
use super::remote;
use super::remote::*;
use super::OpsChannelsRx;
use super::OpsChannelsTx;
use super::SysLog;
use crate::zone::water::arm::Arm;
use time::format_description::well_known::{Rfc2822, Rfc3339};
use tokio::task::spawn_blocking;

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
    zonelog_enable: Option<watch::Sender<bool>>,
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
            zonelog_enable: None,
            status_enable: None,
            ops_tx,
            zone_tx,
        }
    }

    pub async fn init(
        &mut self,
        mut from_zones: ZoneChannelsRx,
        mut ops_rx: OpsChannelsRx,
        selfmutex: crate::ManagerMutex,
    ) -> () {
        let (log_enable_tx, mut log_enable_rx) =
            tokio::sync::watch::channel(false);
        self.zonelog_enable = Some(log_enable_tx);
        let (status_enable_tx, mut status_enable_rx) =
            tokio::sync::watch::channel(false);
        self.status_enable = Some(status_enable_tx);

        /// Start log messages handler
        let manager_mutex = selfmutex.clone();
        let to_log = self.ops_tx.syslog.clone();
        let log_handler = tokio::spawn(async move {
            let mut log_enabled = *log_enable_rx.borrow();
            let mut status_enabled = *status_enable_rx.borrow();
            to_log
                .send(SysLog::new(format!("Spawned log handler")))
                .await;
            loop {
                tokio::select! {
                    Ok(()) = log_enable_rx.changed() => {
                        log_enabled = *log_enable_rx.borrow();
                    }
                    Ok(()) = status_enable_rx.changed() => {
                        status_enabled = *status_enable_rx.borrow();
                    }
                    Some(data) = ops_rx.syslog.recv() => {
                        if true {
                            let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                            // println!("{} {:?}", now.format(&Rfc2822).expect("Time formatting error"), &data);
                            println!("{} {:?}", format_time(now), &data);
                        }
                    }
                    Some(data) = from_zones.zonelog.recv() => {
                        if log_enabled {
                            let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                            // println!("{} {}", now.format(&Rfc2822).expect("Time formatting error"), &data);
                            println!("{} {}", format_time(now), &data);
                        }
                    }
                    Ok(data) = from_zones.zonestatus.recv() => {
                        if status_enabled {
                            // let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                            // println!("{} {}", now.format(&Rfc2822).expect("Time formatting error"), &data);
                            println!("{}", &data);
                        }
                        {
                            manager_mutex.lock().await.update_board().await;
                        }
                    }
                    else => { break }
                };
            }
        });

        // Calibrate arm
        {
            let mut lock = self.house.lock().await;
            let result = lock.arm_calibrate(1).await;
            self.ops_tx
                .syslog
                .send(SysLog::new(format!("Calibrated X Y from: {:?}", result)))
                .await;
        }

        // The 3 below are handled with different alternatives. 
        // Handled by log messages handler that runs an update-method from here. In contrast TextDisplay (below)
        // subscribes to zonestatus and handles updates there. Uncertain what is preferable. 

        // Init buttons
        let (buttons_tx, mut from_buttons) =
            broadcast::channel(16);
        let _ = self.buttons.init(buttons_tx.clone());
        let to_log = self.ops_tx.syslog.clone();
        let house = self.house.clone();
        let btn_handler = tokio::spawn(async move {
            to_log
                .send(SysLog::new(format!("Spawned button handler")))
                .await;
            loop {
                tokio::select! {
                    Ok(data) = from_buttons.recv() => {
                        println!("{:?}", &data);
                        match data {
                            ButtonInput::OneUp => {
                                house.lock().await.pump_stop(1);
                            }
                            ButtonInput::OneDown => {
                                house.lock().await.pump_run(1);
                            }
                            ButtonInput::TwoUp => {
                            }
                            ButtonInput::TwoDown => {
                            }
                        }
                    }
                    else => { break }
                };
            }
        });

        // Start indicator display
        

        // Start text display
        self.display.init(
            self.zone_tx.zonestatus.subscribe(),
            self.ops_tx.syslog.clone(),
        );

        /// Start action messages handler
        let to_log = self.ops_tx.syslog.clone();
        let house = self.house.clone();
        let zoneupdate_handler = tokio::spawn(async move {
            to_log
                .send(SysLog {
                    msg: format!("Spawned zoneupdate handler"),
                })
                .await;
            while let Some(data) = from_zones.zoneupdate.recv().await {
                let now = OffsetDateTime::now_utc().to_offset(TIME_OFFSET);
                // println!("{:?}`Manager recv: {:?}", now, data);

                // let mut lock = house.lock().await;
                match data {
                    ZoneUpdate::Water {
                        id: water_id,
                        settings,
                        status,
                    } => {
                        // Bryt ut nedan till funktion, task?
                        let moisture = status.read().moisture_level;
                        if moisture.is_none() {
                            to_log
                                .send(SysLog::new(format!(
                                    "Moisture level not found for {}.",
                                    water_id
                                )))
                                .await;
                            continue;
                        }
                        let moisture = moisture.unwrap();
                        if moisture > settings.moisture_limit_water {
                            // to_log.send(SysLog::new(format!("Water {}; moist {} above limit {}.", id, moisture, settings.moisture_limit_water))).await;
                            continue;
                        }
                        to_log.send(SysLog::new(format!("Water {}; moist {} below limit {}. Init watering.", water_id, moisture, settings.moisture_limit_water))).await;

                        // Check tank status
                        let tank_status = house
                            .lock()
                            .await
                            .get_displaystatus(ZoneKind::Tank, settings.tank_id)
                            .expect(&format!(
                                "Tank Zone {} not found",
                                &settings.tank_id
                            ));
                        if tank_status.indicator == Indicator::Red {
                            to_log
                                .send(SysLog::new(format!(
                                    "Water zone {} failed: Tank {} empty",
                                    water_id, settings.tank_id
                                )))
                                .await;
                            continue;
                        }

                        // Check pump status

                        // Get Arm ststus and control_rx
                        let movement = settings.position;
                        let mut arm_status: Option<
                            Arc<RwLock<crate::zone::arm::Status>>,
                        > = None;
                        let mut arm_control_rx: Option<
                            broadcast::Receiver<ArmState>,
                        > = None;
                        for z in house.lock().await.zones() {
                            match z {
                                Zone::Arm {
                                    id: arm_id , runner, status, ..
                                } if arm_id == &movement.arm_id => {
                                    arm_status = Some(status.clone());
                                    arm_control_rx = Some(
                                        runner
                                            .control_feedback_sender()
                                            .subscribe(),
                                    );
                                }
                                _ => {}
                            }
                        }
                        if arm_status.is_none() | arm_control_rx.is_none() {
                            to_log.send(SysLog::new(format!(
                                "Water zone {} failed: Arm Zone {} not found",
                                &water_id, &movement.arm_id
                            )));
                            continue;
                        }
                        let arm_status = arm_status.unwrap();
                        let mut arm_control_rx = arm_control_rx.unwrap();
                        // Check arm status

                        let mut tries = 0u8;
                        while tries < 3 {
                            let _ = house.lock().await.arm_goto(
                                movement.arm_id,
                                movement.x,
                                movement.y,
                                movement.z,
                            ); // TODO check result
                               // sleep(Duration::from_secs(2)).await; // TODO wait for arm position to be correct
                            while let Ok(arm_data) = arm_control_rx.recv().await
                            {
                                match arm_data {
                                    ArmState::Busy => {}
                                    ArmState::Idle => {
                                        break;
                                    }
                                }
                            }
                            let confirmed = house
                                .lock()
                                .await
                                .confirm_arm_position(water_id, 5)
                                .unwrap();
                            to_log
                                .send(SysLog::new(format!(
                                    "Confirm position: {:?}",
                                    confirmed
                                )))
                                .await;
                            if confirmed.0 {
                                break;
                            }
                            tries += 1;
                        }
                        if tries < 3 {
                            let _ =
                                house.lock().await.pump_run(settings.pump_id); // TODO check result
                            sleep(settings.pump_time).await;
                            let _ =
                                house.lock().await.pump_stop(settings.pump_id); // TODO check result
                            to_log
                                .send(SysLog::new(format!(
                                    "Water zone {} ok",
                                    water_id
                                )))
                                .await;
                        } else {
                            to_log.send(SysLog::new(format!("Water zone {} failed, couldn't confirm position", water_id))).await;
                        }
                    }
                    ZoneUpdate::Tank { .. } => {}
                    ZoneUpdate::Arm { .. } => {}
                }
            }
        });
    }

    pub async fn update_board(&mut self) {
        let mut lock = self.house.lock().await;
        let all_ds = lock.collect_display_status();
        self.board.set(all_ds).await;
    }

    pub async fn position_from_rc(
        &mut self,
        water_id: u8,
    ) -> Option<(i32, i32, i32)> {
        let to_log = self.ops_tx.syslog.clone();

        let settings = self.house.lock().await.get_water_settings(water_id);
        if settings.is_none() {
            to_log
                .send(SysLog::new(format!(
                    "Set position failed; Water id:{} not found",
                    &water_id
                )))
                .await;
            return None;
        }
        let arm_id = settings.unwrap().position.arm_id;

        let mut to_arm: Option<broadcast::Sender<ArmCmd>> = None;
        for z in self.house.lock().await.zones() {
            match z {
                Zone::Arm { id, runner, .. } if id == &arm_id => {
                    // } if id == &zid => {  // Calling Arm with non-existing id (like 2) leads to interesting panics, look to make that more resilient later
                    to_arm = Some(runner.cmd_sender());
                }
                _ => {}
            }
        }
        if to_arm.is_none() {
            to_log
                .send(SysLog::new(format!(
                    "Set position failed; Arm id:{} not found",
                    arm_id
                )))
                .await;
            return None;
        }
        let to_arm = to_arm.unwrap();

        /// Init remote control
        let (rc_tx, mut rc_rx) = mpsc::channel::<RcInput>(64);
        let cancel = CancellationToken::new();
        let guard = cancel.clone().drop_guard();
        let _ = self.remote.init(rc_tx, cancel.clone()).await;

        let house = self.house.clone();

        // Move: to_log, to_arm, rc_rx, house
        let position_finder = tokio::task::spawn(async move {
            to_log
                .send(SysLog::new(format!("Spawned position finder")))
                .await;
            // let mut arm_o: Option<Arc<&(dyn Arm + '_)>> = None;
            // let arm: Arc<&(dyn Arm + '_)>;
            {
                let mut lock = house.lock().await;
                // for z in lock.zones() {
                //     match z {
                //         Zone::Arm {
                //             id,
                //             interface,
                //             ..
                //         } if id == &arm_id => {
                //         // } if id == &zid => {  // Calling Arm with non-existing id (like 2) leads to interesting panics, look to make that more resilient later
                //             arm_o = Some(Arc::new(interface.arm.as_deref().expect(&format!("Arm Zone {} not found", &arm_id))));
                //         }
                //         _ => {}
                //     }
                // }
                // if arm_o.is_none() { eprintln!("arm_o was none"); return RcModeExit::ElseExit }
                // else {
                //     arm = arm_o.unwrap();
                // } //expect("Zone not found");
                loop {
                    tokio::select! {
                        Some(data) = rc_rx.recv() => {
                            match data {
                                RcInput::LeftUp | RcInput::RightUp => {
                                    // arm.stop_x();
                                    to_arm.send(ArmCmd::StopX);
                                }
                                RcInput::DownUp | RcInput::UpUp => {
                                    // arm.stop_y();
                                    to_arm.send(ArmCmd::StopY);
                                }
                                RcInput::Left => {
                                    to_arm.send(ArmCmd::StartX { speed: -20 });
                                    // arm.start_x(-20);
                                }
                                RcInput::Right => {
                                    // arm.start_x(20);
                                    to_arm.send(ArmCmd::StartX { speed: 20 });
                                }
                                RcInput::Up => {
                                    // arm.start_y(80);
                                    to_arm.send(ArmCmd::StartY { speed: 80 });
                                }
                                RcInput::Down => {
                                    // arm.start_y(-80);
                                    to_arm.send(ArmCmd::StartY { speed: -80 });
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
        let exit_kind = match position_finder.await {
            Ok(exitmode) => Some(exitmode),
            Err(e) => {
                eprintln!("Position finder error: {}", e);
                None
            }
        };
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
                .arm_position(arm_id)
                .expect("Error getting arm position from house");
        }

        /// Exit mode from RC-loop determines what to do next
        match exit_kind {
            Some(RcModeExit::Confirm) => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!("Selected position: {:?}", &pos),
                    })
                    .await;
                // println!("Req house lock for set water pos");
                self.house.lock().await.set_water_position(water_id, pos);
                // println!("Water pos NOT set");
                Some(pos)
            }
            Some(RcModeExit::Cancel) => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!("Position finder cancelled: {:?}", &pos),
                    })
                    .await;
                None
            }
            Some(RcModeExit::SwitchFromPositionMode) => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!("Position finder mode switch"),
                    })
                    .await;
                None
            }
            Some(RcModeExit::SwitchFromOpsMode) => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!(
                            "Position finder unexpected exit (wrong mode)"
                        ),
                    })
                    .await;
                None
            }
            Some(RcModeExit::ElseExit) => {
                self.ops_tx
                    .syslog
                    .send(SysLog {
                        msg: format!(
                            "Position finder unexpected exit (select else)"
                        ),
                    })
                    .await;
                None
            }
            None => None,
        }
    }

    pub fn zonelog_toggle(&self) -> Option<bool> {
        let mut r: Option<bool> = None;
        match &self.zonelog_enable {
            Some(sender) => {
                if *sender.borrow() {
                    sender.send(false);
                    r = Some(false);
                } else {
                    sender.send(true);
                    r = Some(true);
                }
            }
            None => {}
        }

        r
    }
    pub fn statuslog_toggle(&self) -> Option<bool> {
        let mut r: Option<bool> = None;
        match &self.status_enable {
            Some(sender) => {
                if *sender.borrow() {
                    sender.send(false);
                    r = Some(false);
                } else {
                    sender.send(true);
                    r = Some(true);
                }
            }
            None => {}
        }

        r
    }
    pub async fn blink(&mut self) -> (Result<(), Box<dyn Error>>) {
        self.board
            .blink_all(Duration::from_millis(500), Duration::from_secs(1));

        Ok(())
    }
}

use core::error::Error;
use core::time::Duration;
use grow::zone::arm::ArmCmd;
use grow::zone::light::LampState;
use grow::zone::pump;
use grow::zone::pump::PumpCmd;
use grow::zone::pump::Runner;
use grow::zone::Zone;
use grow::House;
use grow::HouseMutex;
use grow::ManagerMutex;
use text_io::read;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
pub enum ZoneCmd {
    // Air { id: u8, sender: broadcast::Sender<AirCmd>, },
    // Aux { id: u8, sender: broadcast::Sender<AuxCmd>, },
    Light {
        id: u8,
        sender: broadcast::Sender<(u8, bool)>,
    },
    // Water { id: u8, sender: broadcast::Sender<WaterCmd>, },
    Arm {
        id: u8,
        sender: broadcast::Sender<ArmCmd>,
    },
    Pump {
        id: u8,
        sender: broadcast::Sender<(u8, PumpCmd)>,
    },
    // Tank { id: u8, sender: broadcast::Sender<(u8, TankCmd)>, },
}

pub fn list_cmds() {
    println!("board\nupdate\nblink\nlogstart\nlogstop\nmoist\nlight1\ntemp1\nfan1\ntank1\nlamp1on\nlamp1off\nfan1dc\npump1run\npump1\nps\narm1x\narm1y\narmupdate\narmpos\ncalib\ncalibx\ncaliby\nwaterpos\n");
}

pub fn manual_cmds(
    mut house: HouseMutex,
    mut manager: ManagerMutex,
    shutdown: mpsc::UnboundedSender<bool>,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    // -> () {
    // let senders = collect_cmd_senders(house.clone()); //.await;

    Ok(tokio::spawn(async move {
        loop {
            print!("(l)ist cmds, or (q)uit\n> ");
            let line: String = read!("{}\n");
            tokio::task::yield_now().await;
            if (line.len() == 0) | line.starts_with("\r") {
                continue;
            }
            match line {
                // Operations commands
                _line if _line.contains("board") => {
                    let mut board = house.lock().await.collect_display_status();
                    board.sort();
                    // board.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    for z in board {
                        println!("{}", z);
                    }
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("update") => {
                    let _ = manager.lock().await.update_board().await;
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("blink") => {
                    println!("Calling blink");
                    let _ = manager.lock().await.blink().await;
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("logstart") => {
                    let _ = manager.lock().await.log_enable(true);
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("logstop") => {
                    let _ = manager.lock().await.log_enable(false);
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("remote") => {
                    // Connect to remote
                    print!("Set position for Water zone > ");
                    let _line: String = read!("{}\n");
                    let zid = _line.trim().parse::<u8>().unwrap();
                    let pos = manager.lock().await.position_from_rc(zid).await;
                    // if pos.is_some() {
                    //     println!("Got some pos: {:?}", &pos);
                    //     // {
                    //     //     println!("house try lock: {:?}", house.try_lock());
                    //     // }
                    //     house.lock().await.set_water_position(zid, pos.unwrap());
                    // } else {
                    //     eprintln!("Set position failure");
                    // }
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("waterpos") => {
                    print!("Show settings from Water zone > ");
                    let _line: String = read!("{}\n");
                    let zid = _line.trim().parse::<u8>().unwrap();
                    let mut lock = house.lock().await;
                    let response = lock.get_water_settings(zid);
                    println!("\tWater zone {} settings: {:#?}", &zid, &response);
                    tokio::task::yield_now().await;
                }

                // Sensor requests
                _line if _line.contains("moist") => {
                    print!("Read moisture from Water zone > ");
                    let _line: String = read!("{}\n");
                    let zid = _line.trim().parse::<u8>().unwrap();
                    let mut lock = house.lock().await;
                    let response = lock.read_moisture_value(zid);
                    println!("\tWater zone {} moisture: {:?}", &zid, &response);
                }
                _line if _line.contains("light1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_light_value(1u8);
                    println!("\tLight zone {} brightness: {:?}", 1, &response);
                }
                _line if _line.contains("temp1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_temperature_value(1u8);
                    println!("\tAir zone {} temperature: {:?}", 1, &response);
                }
                _line if _line.contains("fan1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_fan_speed(1u8);
                    println!("\tAir zone {} fan speed: {:?}", 1, &response);
                }
                _line if _line.contains("tank1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_tank_level(1u8);
                    println!("\tTank zone {} level: {:?}", 1, &response);
                }

                // General action commands
                _line if _line.contains("lamp1on") => {
                    let _ = house.lock().await.set_lamp_state(1u8, LampState::On);
                }
                _line if _line.contains("lamp1off") => {
                    let _ = house.lock().await.set_lamp_state(1u8, LampState::Off);
                }
                _line if _line.contains("fan1dc") => {
                    print!("Fan 1 duty cycle > ");
                    let _line: String = read!("{}\n");
                    let input = _line.trim().parse::<f64>().unwrap();
                    let _ = house.lock().await.set_fan_duty_cycle(1, input);
                }

                // Pump actions
                _line if _line.contains("pump1run") => {
                    let m = house.clone();
                    tokio::spawn(async move {
                        let _ = m.lock().await.pump_run(1u8);
                    });
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("pump1") => {
                    let m = house.clone();
                    tokio::spawn(async move {
                        {
                            let _ = m.lock().await.pump_run(1u8);
                        }
                        tokio::time::sleep(Duration::from_secs(4)).await;
                        {
                            let _ = m.lock().await.pump_stop(1u8);
                        }
                    });
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("ps") => {
                    {
                        let mut lock = house.lock().await;
                        let _ = lock.pump_stop(1u8);
                    }
                    tokio::task::yield_now().await;
                }

                // Arm actions
                _line if _line.contains("arm1x") => {
                    print!("Arm 1 goto X > ");
                    let _line: String = read!("{}\n");
                    let pos = _line.trim().parse::<i32>().unwrap();
                    let mut lock = house.lock().await;
                    let _ = lock.arm_goto_x(1u8, pos);
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("arm1y") => {
                    print!("Arm 1 goto Y > ");
                    let _line: String = read!("{}\n");
                    let pos = _line.trim().parse::<i32>().unwrap();
                    let mut lock = house.lock().await;
                    let _ = lock.arm_goto_y(1u8, pos);
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("armupdate") => {
                    let _ = house.lock().await.arm_update(1u8).await;
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("armpos") => {
                    {
                        let mut lock = house.lock().await;
                        let pos = lock.arm_position(1u8);
                        println!("Arm position: {:?}", pos);
                    }
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("calibx") => {
                    {
                        let mut lock = house.lock().await;
                        let result = lock.arm_calibrate_x(1).await;
                        println!("Calibrated X from: {:?}", result);
                    }
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("caliby") => {
                    {
                        let mut lock = house.lock().await;
                        let result = lock.arm_calibrate_y(1).await;
                        println!("Calibrated Y from: {:?}", result);
                    }
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("calib") => {
                    {
                        let mut lock = house.lock().await;
                        let result = lock.arm_calibrate(1).await;
                        println!("Calibrated X Y from: {:?}", result);
                    }
                    tokio::task::yield_now().await;
                }

                // Special commands
                _line if _line.contains("l") => {
                    list_cmds();
                }
                _line if _line.contains("q") => {
                    break; // Ok(())
                }
                String { .. } => (),
            }
        }
        shutdown.send(true);
    })) //;
}

// Ok(tokio::spawn(async move {
//     loop {
//         print!("(l)ist cmds, or (q)uit > ");
//         let _line: String = read!("{}\n");
//         if (_line.len() == 0) | _line.starts_with("\r") {
//             continue;
//         }
//         match _line {
//             _line if _line.contains("moist") => {
//                 print!("Read moisture from Water zone > ");
//                 let _line: String = read!("{}\n");
//                 let zid = _line.trim().parse::<u8>().unwrap();
//                 let mut lock = house.lock().await;
//                 let response = lock.read_moisture_value(zid);
//                 println!("Water zone {} moisture: {:?}", &zid, &response);
//             }
//             _line if _line.contains("pump1") => {
//                 {
//                     let mut lock = house.lock().await;
//                     // let _ = house.lock().await.run_pump(1u8, 2).await;
//                     lock.run_pump(1u8, 2).await;
//                 }
//                 println!("Pump command complete");
//             }
//             _line if _line.contains("pump1c") => {
//                 {
//                     pump_cmd.send((1, PumpCmd::RunForSec(2)));
//                 }
//                 println!("Pump command complete");
//             }
//             _line if _line.contains("light1") => {
//                 let mut lock = house.lock().await;
//                 let response = lock.read_light_value(1u8);
//                 println!("Light zone {} brightness: {:?}", 1, &response);
//             }
//             _line if _line.contains("temp1") => {
//                 let mut lock = house.lock().await;
//                 let response = lock.read_temperature_value(1u8);
//                 println!("Air zone {} temperature: {:?}", 1, &response);
//             }
//             _line if _line.contains("lamp1on") => {
//                 let _ = house.lock().await.set_lamp_state(1u8, LampState::On);
//             }
//             _line if _line.contains("lamp1off") => {
//                 let _ = house.lock().await.set_lamp_state(1u8, LampState::Off);
//             }
//             _line if _line.contains("fan1dc") => {
//                 print!("Fan 1 duty cycle > ");
//                 let _line: String = read!("{}\n");
//                 let input = _line.trim().parse::<f64>().unwrap();
//                 let _ = house.lock().await.set_duty_cycle(1, input);
//             }
//             _line if _line.contains("fan1") => {
//                 let mut lock = house.lock().await;
//                 let response = lock.read_fan_speed(1u8);
//                 println!("Air zone {} fan speed: {:?}", 1, &response);
//             }
//             _line if _line.contains("arm1x") => {
//                 print!("Arm 1 goto X > ");
//                 let _line: String = read!("{}\n");
//                 let pos = _line.trim().parse::<i32>().unwrap();
//                 let _ = house.lock().await.arm_goto_x(1u8, pos).await;
//             }
//             _line if _line.contains("arm1y") => {
//                 print!("Arm 1 goto Y > ");
//                 let _line: String = read!("{}\n");
//                 let pos = _line.trim().parse::<i32>().unwrap();
//                 let _ = house.lock().await.arm_goto_y(1u8, pos).await;
//             }
//             _line if _line.contains("board") => {
//                 let _ = house.lock().await.collect_display_status();
//             }
//             _line if _line.contains("remote") => {    // Connect to remote
//                 println!("Calling pos from rc");
//                 let _ = manager.lock().await.position_from_rc(1).await;
//             }
//             _line if _line.contains("start") => {   // Start live updates
//                 let _ = house.lock().await.collect_display_status();
//             }
//             _line if _line.contains("stop") => {   // Stop live updates
//                 let _ = house.lock().await.collect_display_status();
//             }
//             _line if _line.contains("q") => {
//                 break; // Ok(())
//             }
//             String { .. } => (),
//         }
//     }
//     println!("Cmd loop exit");
// })) //;

// let mut senders: Vec<ZoneCmd> = Vec::new();
// {
//     let mutex = house.clone();
//     let mut lock = mutex.lock().await;
//     // let mut lock = tokio::task::block_in_place(move || {
//         // house.blocking_lock_owned()
//     // });
//     for zone in lock.zones() {
//         match zone {
//             Zone::Arm{id, runner, ..} => {
//                 senders.push(ZoneCmd::Arm { id: *id, sender: runner.cmd_sender() })
//             }
//             Zone::Pump{id, runner, ..} => {
//                 senders.push(ZoneCmd::Pump { id: *id, sender: runner.cmd_sender() })
//             }
//             _ => {}
//         }
//     }
// }
// dbg!(&senders);

// _line if _line.contains("start") => {   // Start updates
//     let _ = house.lock().await.   ;
// }
// _line if _line.contains("stop") => {   // Stop updates
//     let _ = house.lock().await.   ;
// }

//     pub async fn house_cmds(mut house: HouseMutex,) //manager: ManagerMutex)
//     -> Result<JoinHandle<()>, Box<dyn Error>> {
//         // -> () {
//     let senders = collect_cmd_senders(house.clone()).await;

//     Ok(tokio::spawn(async move {
//         loop {
//             print!("(l)ist cmds, or (q)uit > ");
//             let _line: String = read!("{}\n");
//             if (_line.len() == 0) | _line.starts_with("\r") {
//                 continue;
//             }
//             match _line {
//                 // Sync commands
//                 _line if _line.contains("moist") => {
//                     print!("Read moisture from Water zone > ");
//                     let _line: String = read!("{}\n");
//                     let zid = _line.trim().parse::<u8>().unwrap();
//                     let mut lock = house.lock().await;
//                     let response = lock.read_moisture_value(zid);
//                     println!("Water zone {} moisture: {:?}", &zid, &response);
//                 }
//                 _line if _line.contains("light1") => {
//                     let mut lock = house.lock().await;
//                     let response = lock.read_light_value(1u8);
//                     println!("Light zone {} brightness: {:?}", 1, &response);
//                 }
//                 _line if _line.contains("temp1") => {
//                     let mut lock = house.lock().await;
//                     let response = lock.read_temperature_value(1u8);
//                     println!("Air zone {} temperature: {:?}", 1, &response);
//                 }

//                 _line if _line.contains("lamp1onc") => {
//                     let fake_input = 1u8;
//                     // let mut cmd: Option<broadcast::Sender<(u8,LampState)>> = None;

//                     for z in &senders {
//                         match z {
//                             ZoneCmd::Light { id, sender } if id == &fake_input => {
//                                 println!("Light zone {} found, sending cmd.", &fake_input);
//                                 let result = sender.send((fake_input, true));
//                                 println!("Lamp cmd result: {:?}", &result);
//                                 tokio::task::yield_now().await;
//                                 // println!("Pump zone {} found, getting sender.", &fake_input);
//                                 // pump_cmd = Some(sender.clone());
//                             }
//                             _ => {
//                             }
//                         }
//                     }
//                 }
//                 _line if _line.contains("lamp1on") => {
//                     let _ = house.lock().await.set_lamp_state(1u8, LampState::On);
//                 }

//                 _line if _line.contains("lamp1offc") => {
//                     let fake_input = 1u8;
//                     // let mut cmd: Option<broadcast::Sender<(u8,LampState)>> = None;

//                     for z in &senders {
//                         match z {
//                             ZoneCmd::Light { id, sender } if id == &fake_input => {
//                                 println!("Light zone {} found, sending cmd.", &fake_input);
//                                 let result = sender.send((fake_input, false));
//                                 println!("Lamp cmd result: {:?}", &result);
//                                 tokio::task::yield_now().await;
//                                 // println!("Pump zone {} found, getting sender.", &fake_input);
//                                 // pump_cmd = Some(sender.clone());
//                             }
//                             _ => {
//                             }
//                         }
//                     }
//                 }
//                 _line if _line.contains("lamp1off") => {
//                     let _ = house.lock().await.set_lamp_state(1u8, LampState::Off);
//                 }
//                 _line if _line.contains("fan1dc") => {
//                     print!("Fan 1 duty cycle > ");
//                     let _line: String = read!("{}\n");
//                     let input = _line.trim().parse::<f64>().unwrap();
//                     let _ = house.lock().await.set_duty_cycle(1, input);
//                 }
//                 _line if _line.contains("fan1") => {
//                     let mut lock = house.lock().await;
//                     let response = lock.read_fan_speed(1u8);
//                     println!("Air zone {} fan speed: {:?}", 1, &response);
//                 }
//                 _line if _line.contains("board") => {
//                     let _ = house.lock().await.collect_display_status();
//                 }

//                 // Async commands use command channels
//                 _line if _line.contains("pump1c") => {
//                     let fake_input = 1u8;
//                     // let mut pump_cmd: Option<broadcast::Sender<(u8,PumpCmd)>> = None;

//                     for z in &senders {
//                         match z {
//                             ZoneCmd::Pump { id, sender } if id == &fake_input => {
//                                 println!("Pump zone {} found, sending cmd.", &fake_input);
//                                 let result = sender.send((1, PumpCmd::RunForSec(2)));
//                                 println!("Pump cmd result: {:?}", &result);
//                                 tokio::task::yield_now().await;
//                             }
//                             _ => {
//                             }
//                         }
//                     }
//                 }
//                 _line if _line.contains("pump1secs") => {
//                     let m = house.clone();
//                     tokio::spawn(async move {
//                         let mut lock = m.lock().await;
//                         let _ = lock.run_pump_for_secs(1u8, 2).await;
//                         // tokio::task::yield_now().await;
//                     });
//                 }

//                 _line if _line.contains("pump1secsyield") => {
//                     let m = house.clone();
//                     tokio::spawn(async move {
//                         let mut lock = m.lock().await;
//                         let _ = lock.run_pump_for_secs(1u8, 2).await;
//                         tokio::task::yield_now().await;
//                     });
//                 }
//                 _line if _line.contains("pump1lockrelyield") => {
//                     let m = house.clone();
//                     tokio::spawn(async move {
//                         {
//                             let mut lock = m.lock().await;
//                             let _ = lock.run_pump(1u8).await;
//                         }
//                         tokio::time::sleep(Duration::from_secs(2)).await;
//                         {
//                             let mut lock = m.lock().await;
//                             let _ = lock.stop_pump(1u8).await;
//                         }
//                         tokio::task::yield_now().await;
//                     });
//                 }
//                 _line if _line.contains("pump1lockrel") => {
//                     let m = house.clone();
//                     tokio::spawn(async move {
//                         {
//                             let mut lock = m.lock().await;
//                             let _ = lock.run_pump(1u8).await;
//                         }
//                         tokio::time::sleep(Duration::from_secs(2)).await;
//                         {
//                             let mut lock = m.lock().await;
//                             let _ = lock.stop_pump(1u8).await;
//                         }
//                         // tokio::task::yield_now().await;
//                     });
//                 }
//                 _line if _line.contains("pump1") => {
//                     let m = house.clone();
//                     tokio::spawn(async move {
//                         let mut lock = m.lock().await;
//                         let _ = lock.run_pump(1u8).await;
//                         tokio::time::sleep(Duration::from_secs(2)).await;
//                         let _ = lock.stop_pump(1u8).await;
//                         // tokio::task::yield_now().await;
//                     });
//                 }
//                 _line if _line.contains("ps") => {
//                     let mut lock = house.lock().await;
//                     let _ = lock.stop_pump(1u8).await;
//                     tokio::task::yield_now().await;
//                 }
//                 _line if _line.contains("arm1x") => {
//                     print!("Arm 1 goto X > ");
//                     let _line: String = read!("{}\n");
//                     let pos = _line.trim().parse::<i32>().unwrap();
//                     let mut lock = house.lock().await;
//                     let _ = lock.arm_goto_x(1u8, pos).await;
//                     tokio::task::yield_now().await;
//                 }
//                 _line if _line.contains("arm1y") => {
//                     print!("Arm 1 goto Y > ");
//                     let _line: String = read!("{}\n");
//                     let pos = _line.trim().parse::<i32>().unwrap();
//                     let mut lock = house.lock().await;
//                     let _ = lock.arm_goto_y(1u8, pos).await;
//                     tokio::task::yield_now().await;
//                 }

//                 // _line if _line.contains("remote") => {    // Connect to remote
//                 //     println!("Calling pos from rc");
//                 //     let _ = manager.lock().await.position_from_rc(1).await;
//                 // }

//                 _line if _line.contains("q") => {
//                     break; // Ok(())
//                 }
//                 String { .. } => (),
//             }
//             println!("Cmd loop end");
//         }
//         println!("Cmd loop exit");
//     })) //;
// }

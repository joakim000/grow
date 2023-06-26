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
    // Air {id: u8, info: DisplayStatus},
    // Aux {id: u8, info: DisplayStatus},
    Light {
        id: u8,
        sender: broadcast::Sender<(u8, bool)>,
    },
    // Irrigation {id: u8, info: DisplayStatus},
    Arm {
        id: u8,
        sender: broadcast::Sender<ArmCmd>,
    },
    Pump {
        id: u8,
        sender: broadcast::Sender<(u8, PumpCmd)>,
    },
    // Tank {id: u8, info: DisplayStatus},
}

pub fn collect_cmd_senders(mut house: HouseMutex) -> Vec<ZoneCmd> {
    let mut r: Vec<ZoneCmd> = Vec::new();
    // let mut lock = house.lock().await;
    let mut lock = tokio::task::block_in_place(move || house.blocking_lock_owned());
    for zone in lock.zones() {
        match zone {
            Zone::Arm { id, runner, .. } => r.push(ZoneCmd::Arm {
                id: *id,
                sender: runner.cmd_sender(),
            }),
            Zone::Pump { id, runner, .. } => r.push(ZoneCmd::Pump {
                id: *id,
                sender: runner.cmd_sender(),
            }),
            Zone::Light { id, runner, .. } => r.push(ZoneCmd::Light {
                id: *id,
                sender: runner.lamp_cmd_sender(),
            }),
            _ => {}
        }
    }
    dbg!(&r);
    r
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
            print!("(l)ist cmds, or (q)uit > ");
            let line: String = read!("{}\n");
            tokio::task::yield_now().await;
            if (line.len() == 0) | line.starts_with("\r") {
                continue;
            }
            match line {
                // Operations commands
                line if line.contains("board") => {
                    let board = house.lock().await.collect_display_status();
                    println!("{:#?}", board);
                    tokio::task::yield_now().await;
                }
                line if line.contains("update") => {
                    let _ = manager.lock().await.update_board().await;
                    tokio::task::yield_now().await;
                }
                line if line.contains("blink") => {
                    println!("Calling blink");
                    let _ = manager.lock().await.blink().await;
                    tokio::task::yield_now().await;
                }
                line if line.contains("logstart") => {
                    let _ = manager.lock().await.log_enable(true);
                    tokio::task::yield_now().await;
                }
                line if line.contains("logstop") => {
                    let _ = manager.lock().await.log_enable(false);
                    tokio::task::yield_now().await;
                }

                // Sensor requests
                line if line.contains("moist") => {
                    print!("Read moisture from Irrigation zone > ");
                    let line: String = read!("{}\n");
                    let zid = line.trim().parse::<u8>().unwrap();
                    let mut lock = house.lock().await;
                    let response = lock.read_moisture_value(zid);
                    println!("\tIrrigation zone {} moisture: {:?}", &zid, &response);
                }
                line if line.contains("light1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_light_value(1u8);
                    println!("\tLight zone {} brightness: {:?}", 1, &response);
                }
                line if line.contains("temp1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_temperature_value(1u8);
                    println!("\tAir zone {} temperature: {:?}", 1, &response);
                }
                line if line.contains("fan1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_fan_speed(1u8);
                    println!("\tAir zone {} fan speed: {:?}", 1, &response);
                }
                line if line.contains("tank1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_tank_level(1u8);
                    println!("\tTank zone {} level: {:?}", 1, &response);
                }

                // General action commands
                line if line.contains("lamp1on") => {
                    let _ = house.lock().await.set_lamp_state(1u8, LampState::On);
                }
                line if line.contains("lamp1off") => {
                    let _ = house.lock().await.set_lamp_state(1u8, LampState::Off);
                }
                line if line.contains("fan1dc") => {
                    print!("Fan 1 duty cycle > ");
                    let line: String = read!("{}\n");
                    let input = line.trim().parse::<f64>().unwrap();
                    let _ = house.lock().await.set_fan_duty_cycle(1, input);
                }
                
                // Pump actions                
                line if line.contains("pump1run") => {
                    let m = house.clone();
                    tokio::spawn(async move {
                        let _ = m.lock().await.pump_run(1u8);
                    });
                    tokio::task::yield_now().await;
                }
                line if line.contains("pump1") => {
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
                line if line.contains("ps") => {
                    {
                        let mut lock = house.lock().await;
                        let _ = lock.pump_stop(1u8);
                    }
                    tokio::task::yield_now().await;
                }

                // Arm actions
                line if line.contains("arm1x") => {
                    print!("Arm 1 goto X > ");
                    let line: String = read!("{}\n");
                    let pos = line.trim().parse::<i32>().unwrap();
                    let mut lock = house.lock().await;
                    let _ = lock.arm_goto_x(1u8, pos);
                    tokio::task::yield_now().await;
                }
                line if line.contains("arm1y") => {
                    print!("Arm 1 goto Y > ");
                    let line: String = read!("{}\n");
                    let pos = line.trim().parse::<i32>().unwrap();
                    let mut lock = house.lock().await;
                    let _ = lock.arm_goto_y(1u8, pos);
                    tokio::task::yield_now().await;
                }
                line if line.contains("armupdate") => {
                    let _ = house.lock().await.arm_update(1u8).await;
                    tokio::task::yield_now().await;
                }
                line if line.contains("armpos") => {
                    {
                        let mut lock = house.lock().await;
                        let pos = lock.arm_position(1u8);
                        println!("Arm position: {:?}", pos);
                    }
                    tokio::task::yield_now().await;
                }
                line if line.contains("remote") => {
                    // Connect to remote
                    let _ = manager.lock().await.position_from_rc(1).await;
                    tokio::task::yield_now().await;
                }
                
                // Special commands
                line if line.contains("l") => {
                   // TODO list commands
                }
                line if line.contains("q") => {
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
//         let line: String = read!("{}\n");
//         if (line.len() == 0) | line.starts_with("\r") {
//             continue;
//         }
//         match line {
//             line if line.contains("moist") => {
//                 print!("Read moisture from Irrigation zone > ");
//                 let line: String = read!("{}\n");
//                 let zid = line.trim().parse::<u8>().unwrap();
//                 let mut lock = house.lock().await;
//                 let response = lock.read_moisture_value(zid);
//                 println!("Irrigation zone {} moisture: {:?}", &zid, &response);
//             }
//             line if line.contains("pump1") => {
//                 {
//                     let mut lock = house.lock().await;
//                     // let _ = house.lock().await.run_pump(1u8, 2).await;
//                     lock.run_pump(1u8, 2).await;
//                 }
//                 println!("Pump command complete");
//             }
//             line if line.contains("pump1c") => {
//                 {
//                     pump_cmd.send((1, PumpCmd::RunForSec(2)));
//                 }
//                 println!("Pump command complete");
//             }
//             line if line.contains("light1") => {
//                 let mut lock = house.lock().await;
//                 let response = lock.read_light_value(1u8);
//                 println!("Light zone {} brightness: {:?}", 1, &response);
//             }
//             line if line.contains("temp1") => {
//                 let mut lock = house.lock().await;
//                 let response = lock.read_temperature_value(1u8);
//                 println!("Air zone {} temperature: {:?}", 1, &response);
//             }
//             line if line.contains("lamp1on") => {
//                 let _ = house.lock().await.set_lamp_state(1u8, LampState::On);
//             }
//             line if line.contains("lamp1off") => {
//                 let _ = house.lock().await.set_lamp_state(1u8, LampState::Off);
//             }
//             line if line.contains("fan1dc") => {
//                 print!("Fan 1 duty cycle > ");
//                 let line: String = read!("{}\n");
//                 let input = line.trim().parse::<f64>().unwrap();
//                 let _ = house.lock().await.set_duty_cycle(1, input);
//             }
//             line if line.contains("fan1") => {
//                 let mut lock = house.lock().await;
//                 let response = lock.read_fan_speed(1u8);
//                 println!("Air zone {} fan speed: {:?}", 1, &response);
//             }
//             line if line.contains("arm1x") => {
//                 print!("Arm 1 goto X > ");
//                 let line: String = read!("{}\n");
//                 let pos = line.trim().parse::<i32>().unwrap();
//                 let _ = house.lock().await.arm_goto_x(1u8, pos).await;
//             }
//             line if line.contains("arm1y") => {
//                 print!("Arm 1 goto Y > ");
//                 let line: String = read!("{}\n");
//                 let pos = line.trim().parse::<i32>().unwrap();
//                 let _ = house.lock().await.arm_goto_y(1u8, pos).await;
//             }
//             line if line.contains("board") => {
//                 let _ = house.lock().await.collect_display_status();
//             }
//             line if line.contains("remote") => {    // Connect to remote
//                 println!("Calling pos from rc");
//                 let _ = manager.lock().await.position_from_rc(1).await;
//             }
//             line if line.contains("start") => {   // Start live updates
//                 let _ = house.lock().await.collect_display_status();
//             }
//             line if line.contains("stop") => {   // Stop live updates
//                 let _ = house.lock().await.collect_display_status();
//             }
//             line if line.contains("q") => {
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

// line if line.contains("start") => {   // Start updates
//     let _ = house.lock().await.   ;
// }
// line if line.contains("stop") => {   // Stop updates
//     let _ = house.lock().await.   ;
// }

//     pub async fn house_cmds(mut house: HouseMutex,) //manager: ManagerMutex)
//     -> Result<JoinHandle<()>, Box<dyn Error>> {
//         // -> () {
//     let senders = collect_cmd_senders(house.clone()).await;

//     Ok(tokio::spawn(async move {
//         loop {
//             print!("(l)ist cmds, or (q)uit > ");
//             let line: String = read!("{}\n");
//             if (line.len() == 0) | line.starts_with("\r") {
//                 continue;
//             }
//             match line {
//                 // Sync commands
//                 line if line.contains("moist") => {
//                     print!("Read moisture from Irrigation zone > ");
//                     let line: String = read!("{}\n");
//                     let zid = line.trim().parse::<u8>().unwrap();
//                     let mut lock = house.lock().await;
//                     let response = lock.read_moisture_value(zid);
//                     println!("Irrigation zone {} moisture: {:?}", &zid, &response);
//                 }
//                 line if line.contains("light1") => {
//                     let mut lock = house.lock().await;
//                     let response = lock.read_light_value(1u8);
//                     println!("Light zone {} brightness: {:?}", 1, &response);
//                 }
//                 line if line.contains("temp1") => {
//                     let mut lock = house.lock().await;
//                     let response = lock.read_temperature_value(1u8);
//                     println!("Air zone {} temperature: {:?}", 1, &response);
//                 }

//                 line if line.contains("lamp1onc") => {
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
//                 line if line.contains("lamp1on") => {
//                     let _ = house.lock().await.set_lamp_state(1u8, LampState::On);
//                 }

//                 line if line.contains("lamp1offc") => {
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
//                 line if line.contains("lamp1off") => {
//                     let _ = house.lock().await.set_lamp_state(1u8, LampState::Off);
//                 }
//                 line if line.contains("fan1dc") => {
//                     print!("Fan 1 duty cycle > ");
//                     let line: String = read!("{}\n");
//                     let input = line.trim().parse::<f64>().unwrap();
//                     let _ = house.lock().await.set_duty_cycle(1, input);
//                 }
//                 line if line.contains("fan1") => {
//                     let mut lock = house.lock().await;
//                     let response = lock.read_fan_speed(1u8);
//                     println!("Air zone {} fan speed: {:?}", 1, &response);
//                 }
//                 line if line.contains("board") => {
//                     let _ = house.lock().await.collect_display_status();
//                 }

//                 // Async commands use command channels
//                 line if line.contains("pump1c") => {
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
//                 line if line.contains("pump1secs") => {
//                     let m = house.clone();
//                     tokio::spawn(async move {
//                         let mut lock = m.lock().await;
//                         let _ = lock.run_pump_for_secs(1u8, 2).await;
//                         // tokio::task::yield_now().await;
//                     });
//                 }

//                 line if line.contains("pump1secsyield") => {
//                     let m = house.clone();
//                     tokio::spawn(async move {
//                         let mut lock = m.lock().await;
//                         let _ = lock.run_pump_for_secs(1u8, 2).await;
//                         tokio::task::yield_now().await;
//                     });
//                 }
//                 line if line.contains("pump1lockrelyield") => {
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
//                 line if line.contains("pump1lockrel") => {
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
//                 line if line.contains("pump1") => {
//                     let m = house.clone();
//                     tokio::spawn(async move {
//                         let mut lock = m.lock().await;
//                         let _ = lock.run_pump(1u8).await;
//                         tokio::time::sleep(Duration::from_secs(2)).await;
//                         let _ = lock.stop_pump(1u8).await;
//                         // tokio::task::yield_now().await;
//                     });
//                 }
//                 line if line.contains("ps") => {
//                     let mut lock = house.lock().await;
//                     let _ = lock.stop_pump(1u8).await;
//                     tokio::task::yield_now().await;
//                 }
//                 line if line.contains("arm1x") => {
//                     print!("Arm 1 goto X > ");
//                     let line: String = read!("{}\n");
//                     let pos = line.trim().parse::<i32>().unwrap();
//                     let mut lock = house.lock().await;
//                     let _ = lock.arm_goto_x(1u8, pos).await;
//                     tokio::task::yield_now().await;
//                 }
//                 line if line.contains("arm1y") => {
//                     print!("Arm 1 goto Y > ");
//                     let line: String = read!("{}\n");
//                     let pos = line.trim().parse::<i32>().unwrap();
//                     let mut lock = house.lock().await;
//                     let _ = lock.arm_goto_y(1u8, pos).await;
//                     tokio::task::yield_now().await;
//                 }

//                 // line if line.contains("remote") => {    // Connect to remote
//                 //     println!("Calling pos from rc");
//                 //     let _ = manager.lock().await.position_from_rc(1).await;
//                 // }

//                 line if line.contains("q") => {
//                     break; // Ok(())
//                 }
//                 String { .. } => (),
//             }
//             println!("Cmd loop end");
//         }
//         println!("Cmd loop exit");
//     })) //;
// }

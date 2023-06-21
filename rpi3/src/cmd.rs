use grow::HouseMutex;
use grow::ManagerMutex;
use grow::zone::arm::ArmCmd;
use grow::zone::pump;
use core::error::Error;
use grow::zone::light::LampState;
use grow::House;
use text_io::read;
use tokio::task::JoinHandle;
use tokio::sync::broadcast;
use grow::zone::pump::PumpCmd;
use grow::zone::pump::Runner;
use grow::zone::Zone;


#[derive(Clone, Debug)]
pub enum ZoneCmd {
    // Air {id: u8, info: DisplayStatus},
    // Aux {id: u8, info: DisplayStatus},
    Light {id: u8, sender: broadcast::Sender<LampState>},
    // Irrigation {id: u8, info: DisplayStatus},
    Arm {id: u8, sender: broadcast::Sender<ArmCmd>},
    Pump {id: u8, sender: broadcast::Sender<(u8, PumpCmd)>},
    // Tank {id: u8, info: DisplayStatus},
}



pub async fn collect_cmd_senders(mut house: HouseMutex) -> Vec<ZoneCmd> {
    let mut r: Vec<ZoneCmd> = Vec::new();
    let mut lock = house.lock().await;
    // let mut lock = tokio::task::block_in_place(move || {
        // house.blocking_lock_owned()
    // });
    for zone in lock.zones() {
        match zone {
            Zone::Arm{id, runner, ..} => {
                r.push(ZoneCmd::Arm { id: *id, sender: runner.cmd_sender() })
            }
            Zone::Pump{id, runner, ..} => {
                r.push(ZoneCmd::Pump { id: *id, sender: runner.cmd_sender() })
            }
            _ => {}
        }    
    }
    dbg!(&r);
    r
}    


pub async fn house_cmds(mut house: HouseMutex,) //manager: ManagerMutex) 
    -> Result<JoinHandle<()>, Box<dyn Error>> {
    let senders = collect_cmd_senders(house.clone()).await;

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


    // let zid = 1u8;
    // let mut pump_cmd: Option<broadcast::Sender<(u8,PumpCmd)>> = None;
    // {
    //     let mutex = house.clone();
    //     // let mut lock = mutex.lock().await;
    //     let mut lock = tokio::task::block_in_place(move || {
    //         mutex.blocking_lock_owned()
    //     });
    //     // dbg!(lock.zones());
    //     for z in lock.zones() {
    //         match z {
    //             Zone::Pump {id, settings:_, status:_, interface:_, runner} if id == &zid => {
    //                 // let pump_cmd_sender = Some(runner.tx_pumpcmd.clone()); 
    //                 // println!("Matched ")
    //                 pump_cmd = Some(runner.cmd_sender()); 
    //             }
    //             _ => continue
    //         }
    //     }
    // }    
    // let pump_cmd = pump_cmd.expect("Pump zone not found");

    Ok(tokio::spawn(async move {
        loop {
            print!("(l)ist cmds, or (q)uit > ");
            let line: String = read!("{}\n");
            if (line.len() == 0) | line.starts_with("\r") {
                continue;
            }
            match line {
                // Sync commands
                line if line.contains("moist") => {
                    print!("Read moisture from Irrigation zone > ");
                    let line: String = read!("{}\n");
                    let zid = line.trim().parse::<u8>().unwrap();
                    let mut lock = house.lock().await;
                    let response = lock.read_moisture_value(zid);
                    println!("Irrigation zone {} moisture: {:?}", &zid, &response);
                }
                line if line.contains("light1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_light_value(1u8);
                    println!("Light zone {} brightness: {:?}", 1, &response);
                }
                line if line.contains("temp1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_temperature_value(1u8);
                    println!("Air zone {} temperature: {:?}", 1, &response);
                }
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
                    let _ = house.lock().await.set_duty_cycle(1, input);
                }
                line if line.contains("fan1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_fan_speed(1u8);
                    println!("Air zone {} fan speed: {:?}", 1, &response);
                }
                line if line.contains("board") => {
                    let _ = house.lock().await.collect_display_status();
                }
                // line if line.contains("start") => {   // Start live updates
                //     let _ = house.lock().await.   ;
                // }
                // line if line.contains("stop") => {   // Stop live updates
                //     let _ = house.lock().await.   ;
                // }

                // Async commands use command channels
                line if line.contains("pump1c") => {
                    // let mut pump_cmd: Option<broadcast::Sender<(u8,PumpCmd)>> = None;
                    for z in &senders {
                        match z {
                            ZoneCmd::Pump { id, sender } if id == &1 => {
                                println!("Pump zone {} found, sending cmd.", &1);
                                let result = sender.send((1, PumpCmd::RunForSec(2)));
                                println!("Pump cmd result: {:?}", &result);

                                // println!("Pump zone {} found, getting sender.", &1);
                                // pump_cmd = Some(sender.clone());
                            }
                            _ => {
                                // continue;
                                // println!("Pump zone {} not found.", &fake_input);
                            }
                        }
                    }
                    // println!("Pump zone {} found, sending cmd.", &fake_input);
                    // let pump_cmd = pump_cmd.expect("Pump zone not found");

                    // {
                    //     pump_cmd.send((1, PumpCmd::RunForSec(2)));
                    // }
                    // println!("Pump command complete");
                }
                line if line.contains("arm1x") => {
                    print!("Arm 1 goto X > ");
                    let line: String = read!("{}\n");
                    let pos = line.trim().parse::<i32>().unwrap();
                    let _ = house.lock().await.arm_goto_x(1u8, pos).await;
                }
                line if line.contains("arm1y") => {
                    print!("Arm 1 goto Y > ");
                    let line: String = read!("{}\n");
                    let pos = line.trim().parse::<i32>().unwrap();
                    let _ = house.lock().await.arm_goto_y(1u8, pos).await;
                }
               
                line if line.contains("remote") => {    // Connect to remote
                    println!("Calling pos from rc");
                    let _ = manager.lock().await.position_from_rc(1).await;
                }
              
                line if line.contains("q") => {
                    break; // Ok(())
                }
                String { .. } => (),
            }
        }
        println!("Cmd loop exit");
    })) //;
    // Ok(())
    // println!("Cmd loop exit");
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
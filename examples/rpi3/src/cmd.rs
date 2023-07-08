use core::error::Error;
use core::time::Duration;

use grow::zone::light::LampState;



use grow::HouseMutex;
use grow::ManagerMutex;
use text_io::read;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;



pub fn list_cmds() {
    let general_list = vec![
        ("board", "Show status board"),
        ("status", "Toggle output on status change"),
        ("log", "Toggle output on zone log event"),
        ("pset", "Set water position with RC"),
        ("pshow", "Show settings for Water zone"),
        ("pconfirm", "Confirm arm positioned for Water zone"),
        ("pgoto", "Go to position for Water zone"),
        ("calib", "Calibrate Arm zero-position"),
    ];
    let debug_list = vec![
        ("armpos", "Show current Arm position"),
        ("arm1", "Move Arm 1"),
        ("arm1x", "Move Arm 1 x-axis"),
        ("arm1y", "Move Arm 1 y-axis"),
        ("pump1", "Run Pump 1 for 3 seconds"),
        ("pump1run", "Run Pump 1 until stopped"),
        ("ps", "Stop Pump 1"),
        ("fan1dc", "Set fan duty cycle for Air zone 1"),
    ];
    let sensor_list = vec![
        ("moist", "Take moisture reading from Water zone"),
        ("light1", "Take brightness reading from Light zone 1"),
        ("temp1", "Take temp reading from Air zone 1"),
        ("tank1", "Take level reading from Tank zone 1"),
        ("fan1", "Take fan speed reading from Air zone 1"),
    ];
    for cmd in general_list {
        println!("{:>10}\t{}", cmd.0, cmd.1);
    }
    for cmd in sensor_list {
        println!("{:>10}\t{}", cmd.0, cmd.1);
    }
    for cmd in debug_list {
        println!("{:>10}\t{}", cmd.0, cmd.1);
    }
    println!("\tAlso:\nupdate\nblink\nlamp1on\nlamp1off\narmupdate\nload\nsave\n");
}

#[rustfmt::skip]
pub fn manual_cmds(
    house: HouseMutex,
    manager: ManagerMutex,
    shutdown: mpsc::UnboundedSender<bool>,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    let getnum_u8 = || -> ( bool, u8 ) {
        let _line: String = read!("{}\n"); 
        let num = _line.trim().parse::<u8>();  
        match num {
            Ok(num) => (true, num),
            Err(e) => {
                eprintln!("{:?}. Try again.", e);
                (false, 0)
            }
        }
    };
    let getnum_i32 = || -> ( bool, i32 ) {
        let _line: String = read!("{}\n"); 
        let num = _line.trim().parse::<i32>();  
        match num {
            Ok(num) => (true, num),
            Err(e) => {
                eprintln!("{:?}. Try again.", e);
                (false, 0)
            }
        }
    };
    Ok(tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        loop {
            print!("(l)ist cmds, or (q)uit >");
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
                    for z in board {
                        println!("{}", &z);
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
                _line if _line.contains("log") => {
                    let set_to = manager.lock().await.zonelog_toggle().unwrap();
                    println!("Output zone log: {:?}", set_to);
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("status") => {
                    let set_to = manager.lock().await.statuslog_toggle().unwrap();
                    println!("Output status log: {:?}", set_to);
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("pset") => {
                    // Connect to remote
                    print!("Set position for Water zone > ");
                    let zid = getnum_u8();
                    if !zid.0 {continue;}
                    let pos = manager.lock().await.position_from_rc(zid.1).await;
                    println!("Set position for Water zone {}: {:?}", &zid.1, &pos);
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("pshow") => {
                    print!("Show settings from Water zone > ");
                    let zid = getnum_u8();
                    if !zid.0 {continue;}
                    {
                        let mut lock = house.lock().await;
                        let response = lock.get_water_settings(zid.1);
                        println!("\tWater zone {} settings: {:#?}", &zid.1, &response);
                    }
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("pgoto") => {
                    print!("Move Arm to Water position > ");
                    let zid = getnum_u8();
                    if !zid.0 {continue;}
                    {
                        let mut lock = house.lock().await;
                        let response = lock.get_water_settings(zid.1);
                        println!("\tWater zone {} settings: {:#?}", &zid.1, &response);
                        let movement = response.unwrap().position;
                        let _ = lock.arm_goto(
                            movement.arm_id,
                            movement.x,
                            movement.y,
                            movement.z,
                        );
                    }
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("pconfirm") => {
                    print!("Confirm arm positioned for Water zone > ");
                    let zid = getnum_u8();
                    if !zid.0 {continue;}
                    {
                        let mut lock = house.lock().await;
                        let response = lock.confirm_arm_position(zid.1, 5);
                        println!("\tWater zone {} arm positioned: {:#?}", &zid.1, &response);
                    }
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("load") => {
                    println!("Load settings...");
                    let _ = house.lock().await.load_settings();
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("save") => {
                    println!("Save settings...");
                    let _ = house.lock().await.save_settings();
                    tokio::task::yield_now().await;
                }



                // Sensor requests
                _line if _line.contains("moist") => {
                    print!("Read moisture from Water zone > ");
                    let zid = getnum_u8();
                    if !zid.0 {continue}
                    let mut lock = house.lock().await;
                    let response = lock.read_moisture_value(zid.1);
                    println!("\tWater zone {} moisture: {:?}", &zid.1, &response);
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
                        let _ = m.lock().await.pump_run(1u8).await;
                    });
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("pump1") => {
                    let m = house.clone();
                    tokio::spawn(async move {
                        {
                            let _ = m.lock().await.pump_run(1u8).await;
                        }
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        {
                            let _ = m.lock().await.pump_stop(1u8).await;
                        }
                    });
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("ps") => {
                    {
                        let _ = house.lock().await.pump_stop(1u8).await;
                    }
                    tokio::task::yield_now().await;
                }

                // Arm actions
                _line if _line.contains("arm1x") => {
                    print!("Arm 1 goto X > ");
                    let pos_x = getnum_i32();
                    if !pos_x.0 {continue;}
                    let _ = house.lock().await.arm_goto_x(1u8, pos_x.1).await;
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("arm1y") => {
                    print!("Arm 1 goto Y > ");
                    let pos_y = getnum_i32();
                    if !pos_y.0 {continue;}
                    let _ = house.lock().await.arm_goto_y(1u8, pos_y.1).await;
                    tokio::task::yield_now().await;
                }
                _line if _line.contains("arm1") => {
                    print!("Arm 1 goto X > ");
                    let pos_x = getnum_i32();
                    if !pos_x.0 {continue;}
                    print!("Arm 1 goto Y > ");
                    let pos_y = getnum_i32();
                    if !pos_y.0 {continue;}
                    let _ = house.lock().await.arm_goto(1u8, pos_x.1, pos_y.1, 0).await;
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
        let _ = shutdown.send(true);
    })) //;
}


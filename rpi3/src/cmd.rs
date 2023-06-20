use super::HouseMutex;
use core::error::Error;
use grow::zone::light::LampState;
use grow::House;
use text_io::read;

pub fn house_cmds(mut house: HouseMutex) -> Result<(), Box<dyn Error>> {
    tokio::spawn(async move {
        loop {
            print!("(l)ist cmds, or (q)uit > ");
            let line: String = read!("{}\n");
            if (line.len() == 0) | line.starts_with("\r") {
                continue;
            }
            match line {
                line if line.contains("moist") => {
                    print!("Read moisture from Irrigation zone > ");
                    let line: String = read!("{}\n");
                    let zid = line.trim().parse::<u8>().unwrap();
                    let mut lock = house.lock().await;
                    let response = lock.read_moisture_value(zid);
                    println!("Irrigation zone {} moisture: {:?}", &zid, &response);
                }
                line if line.contains("pump1") => {
                    {
                        let mut lock = house.lock().await;
                        // let _ = house.lock().await.run_pump(1u8, 2).await;
                        lock.run_pump(1u8, 2).await;
                    }
                    println!("Pump command complete");
                }
                line if line.contains("light1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_light_value(1u8);
                    println!("Light zone {} brightness: {:?}", &1, &response);
                }
                line if line.contains("temp1") => {
                    let mut lock = house.lock().await;
                    let response = lock.read_temperature_value(1u8);
                    println!("Air zone {} temperature: {:?}", &1, &response);
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
                line if line.contains("board") => {
                    let _ = house.lock().await.collect_display_status();
                }
                line if line.contains("q") => {
                    break; // Ok(())
                }
                String { .. } => (),
            }
        }
    });
    Ok(())
}

// pub fn house_cmds2(mut house: HouseMutex) -> Result<(), Box<dyn Error>> {
//     tokio::spawn(async move {
//     // loop {
//         print!("(l)ist cmds, or (q)uit > ");
//         let line: String = read!("{}\n");
//         if (line.len() == 0) | line.starts_with("\r") {
//             // continue;
//         } else {
//         match line {
//             line if line.contains("moist") => {
//                 print!("Read moisture from Irrigation zone > ");
//                 let line: String = read!("{}\n");
//                 let zid = line.trim().parse::<u8>().unwrap();
//                 let mut lock = house.lock().await;
//                 let response = lock.read_moisture_value(&zid);
//                 println!("Irrigation zone {} moisture: {:?}", &zid, &response);
//             }
//             line if line.contains("pump1") => {
//                     let mut lock = house.lock().await;
//                     // let _ = house.lock().await.run_pump(1u8, 2).await;
//                     lock.run_pump(1u8, 2).await;
//                     println!("Pump command complete");
//                 }
//             line if line.contains("q") => {
//                 // break // Ok(())
//             }
//             String {..} => ()
//         }
// else if line.contains("moist") {
//     print!("Read moisture from Irrigation zone > ");
//     let line: String = read!("{}\n");
//     let zid = line.trim().parse::<u8>().unwrap();
//     let mut lock = house.lock().await;
//     let response = lock.read_moisture_value(&zid);
//     println!("Irrigation zone {} moisture: {:?}", &zid, &response);

// }
// else if line.contains("light1") {
//     let mut lock = house.lock().await;
//     let response = lock.read_light_value(1u8);
//     println!("Light zone {} brightness: {:?}", &1, &response);
// }
// else if line.contains("temp1") {
//     let mut lock = house.lock().await;
//     let response = lock.read_temperature_value(1u8);
//     println!("Air zone {} temperature: {:?}", &1, &response);
// }
// else if line.contains("lamp1on") {
//     let _ = house.lock().await.set_lamp_state(1u8, LampState::On);
// }
// else if line.contains("lamp1off") {
//     let _ = house.lock().await.set_lamp_state(1u8, LampState::Off);
// }
// else if line.contains("pump1") {
//     // let mut lock = house.lock().await;
//     let _ = house.lock().await.run_pump(1u8, 2).await;
//     // let _ = lock.run_pump(1u8, 3).await;
//     println!("Pump command complete");
//     continue;
// }
// else if line.contains("arm1x") {
//     print!("Arm 1 goto X > ");
//     let line: String = read!("{}\n");
//     let pos = line.trim().parse::<i32>().unwrap();
//     let _ = house.lock().await.arm_goto_x(1u8, pos).await;
// }
// else if line.contains("arm1y") {
//     print!("Arm 1 goto Y > ");
//     let line: String = read!("{}\n");
//     let pos = line.trim().parse::<i32>().unwrap();
//     let _ = house.lock().await.arm_goto_y(1u8, pos).await;
// }

// else if line.contains("q") {
//     break Ok(())
// }
//         }
//         println!("Cmd loop end");
//     // }
// });
// Ok(())
// }

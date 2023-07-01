use super::conf::*;

use async_trait::async_trait;
use core::time::Duration;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
// use std::sync::Mutex;
use core::error::Error;
use parking_lot::RwLock;
use time::OffsetDateTime;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use lego_powered_up::consts::HubPropertyOperation;
use lego_powered_up::consts::HubPropertyRef;
use lego_powered_up::consts::MotorSensorMode;
use lego_powered_up::consts::{named_port, HubType};
use lego_powered_up::error::{Error as LpuError, OptionContext, Result as LpuResult};
use lego_powered_up::hubs::HubNotification;
use lego_powered_up::iodevice::basic::Basic;
use lego_powered_up::iodevice::modes;
use lego_powered_up::iodevice::remote::{RcButtonState, RcDevice};
use lego_powered_up::iodevice::visionsensor::DetectedColor;
use lego_powered_up::iodevice::visionsensor::VisionSensor;
use lego_powered_up::iodevice::{hubled::*, motor::*, sensor::*};
use lego_powered_up::notifications::HubPropertyValue;
use lego_powered_up::notifications::Power;
use lego_powered_up::notifications::*;
use lego_powered_up::HubMutex;
use lego_powered_up::{ConnectedHub, IoDevice, IoTypeId, PoweredUp};
use lego_powered_up::{Hub, HubFilter};
use lego_powered_up::iodevice::motor::BufferState;

use crate::hardware::lpu::broadcast::Receiver;
use grow::ops::display::Indicator;
use grow::zone;
use grow::zone::arm::ArmCmd;
use grow::zone::pump::PumpCmd;
use grow::zone::tank::TankLevel;
use grow::TIME_OFFSET;
use grow::zone::arm::ArmState;
use grow::ops::display::DisplayStatus;

pub async fn init(
    pu: Arc<TokioMutex<PoweredUp>>,
    cancel: CancellationToken,
) -> Result<HubMutex, Box<dyn Error>> {
    let mut lock = pu.lock().await;
    println!("Waiting for hub...");
    let hub = lock
        .wait_for_hub_filter(HubFilter::Kind(HubType::TechnicMediumHub))
        .await?;
    println!("Connecting to hub...");
    let created_hub = lock.create_hub(&hub).await;
    match created_hub {
        Ok(_) => {}
        Err(e) => { { return Err(Box::new(e)) } } 
    }
    let hub = ConnectedHub::setup_hub(created_hub.unwrap()).await;
    match hub {
        Ok(_) => {}
        Err(e) => { { return Err(Box::new(e)) } } 
    }
    // let hub = ConnectedHub::setup_hub(lock.create_hub(&hub).await).expect("Error creating hub") // thread 'main' panicked at 'Error creating hub: BluetoothError(Other(DbusError(D-Bus error: Operation already in progress (org.bluez.Error.Failed))))', src/hardware/lpu.rs:54:67
    //     .await
    //     .expect("Error setting up hub");
    println!("Connectedhub  created");  


    Ok(hub.unwrap().mutex.clone())
}

pub struct Vsensor {
    id: u8,
    device: IoDevice,
    level: Arc<RwLock<TankLevel>>,
    // hub: HubMutex,
    feedback_task: Option<JoinHandle<()>>,
    // color_task: JoinHandle<()>,
    // rx_color: broadcast::Receiver<DetectedColor>,
}
#[async_trait]
impl zone::water::tank::TankSensor for Vsensor {
    fn id(&self) -> u8 {
        self.id
    }

    async fn init(
        &mut self,
        tx_tanklevel: tokio::sync::broadcast::Sender<(u8, Option<TankLevel>)>,
    ) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.tank_feedback(tx_tanklevel)
                .await
                .expect("Error initializing feedback task"),
        );
        Ok(())
    }
    fn read(&self) -> Result<(TankLevel), Box<dyn Error>> {
        Ok(*self.level.read())
    }
}
impl Vsensor {
    pub fn new(id: u8, hub: HubMutex) -> Self {
        let device: IoDevice;
        {
            // let lock = hub.lock().await;
            let lock = tokio::task::block_in_place(move || {
                // let lock = hub.blocking_lock();
                hub.blocking_lock_owned()
            });
            device = lock
                .io_from_kind(IoTypeId::VisionSensor)
                .expect("Error accessing LPU device");
        }
        Self {
            id,
            // hub,
            level: Arc::new(RwLock::new(TankLevel::Red)), // get from save
            device,
            feedback_task: None,
        }
    }

    async fn tank_feedback(
        &self,
        tx: broadcast::Sender<(u8, Option<TankLevel>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let level = self.level.clone();
        let (mut rx_color, _color_task) = self.device.visionsensor_color().unwrap();
        Ok(tokio::spawn(async move {
            println!("Spawned tank feedback");
            while let Ok(data) = rx_color.recv().await {
                // println!("Tank color: {:?} ", data,);
                match data {
                    DetectedColor::NoObject => {
                        let _ = tx.send((id, Some(TankLevel::Blue)));
                        *level.write() = TankLevel::Blue;
                    }
                    DetectedColor::Blue => {
                        let _ = tx.send((id, Some(TankLevel::Green)));
                        *level.write() = TankLevel::Green;
                    }
                    DetectedColor::Green => {
                        let _ = tx.send((id, Some(TankLevel::Green)));
                        *level.write() = TankLevel::Green;
                    }
                    DetectedColor::Yellow => {
                        let _ = tx.send((id, Some(TankLevel::Yellow)));
                        *level.write() = TankLevel::Yellow;
                    }
                    DetectedColor::Red => {
                        let _ = tx.send((id, Some(TankLevel::Red)));
                        *level.write() = TankLevel::Red;
                    }
                    _ => {
                        tx.send((id, None));
                    }
                }
            }
        }))
    }
}

pub struct BrickPump {
    id: u8,
    device: IoDevice,
    // hub: HubMutex,
    control_task: Option<JoinHandle<()>>,
    feedback_task: Option<JoinHandle<()>>,
}
#[async_trait]
impl zone::water::pump::Pump for BrickPump {
    fn id(&self) -> u8 {
        self.id
    }
    async fn run_for_secs(&self, secs: u16) -> Result<(), Box<dyn Error>> {
        // println!("LPU got cmd: run_for_secs({}", &secs);
        self.device.start_speed(50, 100)?;
        sleep(Duration::from_secs(secs as u64)).await;
        self.device.start_power(Power::Float)?;
        Ok(())
    }
    fn run(&self) -> Result<(), Box<dyn Error>> {
        // println!("LPU got cmd: RUN");
        self.device.start_speed(50, 100)?;
        Ok(())
    }
    fn stop(&self) -> Result<(), Box<dyn Error>> {
        // println!("LPU got cmd: STOP");
        self.device.start_power(Power::Brake);
        Ok(())
    }
    fn float(&self) -> Result<(), Box<dyn Error>> {
        self.device.start_power(Power::Float);
        Ok(())
    }
    async fn init(
        &mut self,
        rx_pumpcmd: tokio::sync::broadcast::Receiver<(u8, PumpCmd)>,
        tx_pump: tokio::sync::broadcast::Sender<(u8, (i8, i32))>,
    ) -> Result<(), Box<dyn Error>> {
        self.control_task = Some(
            self.pump_control(rx_pumpcmd)
                .await
                .expect("Error initializing control task"),
        );
        self.feedback_task = Some(
            self.pump_feedback(tx_pump)
                .await
                .expect("Error initializing feedback task"),
        );
        Ok(())
    }
}
impl BrickPump {
    pub async fn new(id: u8, hub: HubMutex) -> Self {
        let device: IoDevice;
        {
            let lock = hub.lock().await;
            device = lock
                .io_from_port(PUMP_ADDR)
                .expect("Error accessing LPU device");
        }
        Self {
            id,
            // hub,
            device,
            control_task: None,
            feedback_task: None,
        }
    }

    async fn pump_control(
        &self,
        mut rx_cmd: broadcast::Receiver<(u8, PumpCmd)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let device = self.device.clone();
        Ok(tokio::spawn(async move {
            println!("Spawned pump control");
            while let Ok(data) = rx_cmd.recv().await {
                // println!("Pump recv cmd: {:?}", &data);
                match data {
                    (_id, PumpCmd::RunForSec(secs)) => {
                        let _ = device.start_speed(50, 100);
                        sleep(Duration::from_secs(secs as u64));
                        let _ = device.start_power(Power::Float);
                    }
                    (_id, PumpCmd::Stop) => {
                        device.start_power(Power::Brake);
                    }
                }
            }
        }))
    }
    async fn pump_feedback(
        &self,
        tx: broadcast::Sender<(u8, (i8, i32))>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let (mut rx_motor, _motor_sensor_task) = self
            .device
            .enable_8bit_sensor(modes::InternalMotorTacho::SPEED, 1)
            // .await
            .unwrap();
        Ok(tokio::spawn(async move {
            println!("Spawned pump feedback");
            while let Ok(data) = rx_motor.recv().await {
                // println!("Pump feedback: {:?} ", data,);
                tx.send((id, (data[0], 0)));
            }
        }))
    }
}

pub struct BrickArm {
    id: u8,
    device_x: IoDevice,
    device_y: IoDevice,
    pos_x: Arc<RwLock<i32>>,
    pos_y: Arc<RwLock<i32>>,
    feedback_task: Option<JoinHandle<()>>,
    cmd_task: Option<JoinHandle<()>>,
    cancel: CancellationToken,
    // hub_channels: lego_powered_up::hubs::Channels,
}
#[async_trait]
impl zone::water::arm::Arm for BrickArm {
    fn id(&self) -> u8 {
        self.id
    }
    async fn init(
        &mut self,
        tx_axis_x: tokio::sync::broadcast::Sender<((i8, i32))>,
        tx_axis_y: tokio::sync::broadcast::Sender<((i8, i32))>,
        _tx_axis_z: tokio::sync::broadcast::Sender<((i8, i32))>,
        tx_control: grow::zone::arm::ControlFeedbackTx,
        rx_cmd: tokio::sync::broadcast::Receiver<ArmCmd>,
    ) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.arm_feedback(tx_axis_x, tx_axis_y, tx_control, self.cancel.clone())
                .await
                .expect("Error initializing feedback task"),
        );
        self.cmd_task = Some(
            self.arm_cmd(rx_cmd, self.cancel.clone())
                .await
                .expect("Error initializing cmd task"),
        );
        Ok(())
    }
    fn goto(&self, x: i32, y: i32, _z: i32) -> Result<(), Box<dyn Error>> {
        self.device_x
            .goto_absolute_position(x, 20, 20, EndState::Brake)?;
        // .await?;
        self.device_y
            .goto_absolute_position(y, 50, 20, EndState::Brake)?;
        // .await?;
        Ok(())
    }
    fn stop(&self) -> Result<(), Box<dyn Error>> {
        self.device_x.start_power(Power::Brake);
        self.device_y.start_power(Power::Brake);
        Ok(())
    }

    async fn update_pos(&self) -> Result<(), Box<dyn Error>> {
        self.device_x
            .device_mode(modes::TechnicLargeLinearMotorTechnicHub::SPEED, 1, true);
        // sleep(Duration::from_millis(100)).await;
        self.device_y
            .device_mode(modes::TechnicLargeLinearMotorTechnicHub::SPEED, 1, true);
        sleep(Duration::from_millis(100)).await;
        self.device_x
            .device_mode(modes::TechnicLargeLinearMotorTechnicHub::POS, 1, true);
        // sleep(Duration::from_millis(100)).await;
        self.device_y
            .device_mode(modes::TechnicLargeLinearMotorTechnicHub::POS, 1, true);

        Ok(()) 
    }
    fn goto_x(&self, x: i32) -> Result<(), Box<dyn Error>> {
        self.device_x
            .goto_absolute_position(x, 50, 20, EndState::Brake)?;
        // .await?;
        Ok(())
    }
    fn goto_y(&self, y: i32) -> Result<(), Box<dyn Error>> {
        self.device_y
            .goto_absolute_position(y, 100, 20, EndState::Brake)?;
        // .await?;
        Ok(())
    }
    fn start_x(&self, speed: i8) -> Result<(), Box<dyn Error>> {
        self.device_x.start_speed(-speed, 15)?;
        Ok(())
    }
    fn stop_x(&self) -> Result<(), Box<dyn Error>> {
        self.device_x.start_power(Power::Brake)?;
        Ok(())
    }
    fn start_y(&self, speed: i8) -> Result<(), Box<dyn Error>> {
        self.device_y.start_speed(speed, 60)?;
        Ok(())
    }
    fn stop_y(&self) -> Result<(), Box<dyn Error>> {
        self.device_y.start_power(Power::Brake)?;
        Ok(())
    }
    fn position(&self) -> Result<((i32, i32, i32)), Box<dyn Error>> {
        Ok((*self.pos_x.read(), *self.pos_y.read(), 0))
    }
    async fn calibrate(&self) -> Result<(i32, i32, i32), Box<dyn Error>> {
        let cancel = CancellationToken::new();
        let guard = cancel.clone().drop_guard();
        let device_x = self.device_x.clone();
        let device_y = self.device_y.clone();
        let (tx_axis_x, mut rx_axis_x) = broadcast::channel::<(i8, i32)>(16);
        let (tx_axis_y, mut rx_axis_y) = broadcast::channel::<(i8, i32)>(16);
        let (tx_control, mut rx_control) = broadcast::channel::<ArmState>(16);


        let feedback_task = Some(
            self.arm_feedback(tx_axis_x, tx_axis_y, tx_control, cancel.clone())
                .await
                .expect("Error initializing feedback task"),
        );

        let calibration_task_x = tokio::spawn(async move {
            let mut started = false;
            device_x.start_speed(-20, 20);
            loop {
                tokio::select! {
                    Ok(data) = rx_axis_x.recv() => {
                        if started & (data.0 >= 0) {
                            device_x.start_power(Power::Float);
                            // println!("Calib X stopped");
                            break;
                        }
                        if !started & (data.0 < 0) {
                            started = true;
                        }
                    }
                    else => { break }
                };
            }
        });
        let calibration_task_y = tokio::spawn(async move {
            let mut started = false;
            device_y.start_speed(-30, 20);
            loop {
                tokio::select! {
                    Ok(data) = rx_axis_y.recv() => {
                        if started & (data.0 >= 0) {
                            device_y.start_power(Power::Float);
                            // println!("Calib Y stopped");
                            break;
                        }
                        if !started & (data.0 < 0) {
                            started = true;
                        }
                    }
                    else => { break }
                };
            }
        });
        tokio::join!(calibration_task_x, calibration_task_y);
        // println!("Calib tasks joined");
        let before = (*self.pos_x.read(), *self.pos_y.read(), 0);
        self.device_x.preset_encoder(0);
        self.device_y.preset_encoder(0);
        // println!("Calibrated X Y from: {:?}", &before);
        Ok(before)
    }

  

    /// Calibrate zero-point and range
    /// Needed if we want to use relative position settings, using absolute values for now
    async fn calibrate_with_range(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
impl BrickArm {
    pub async fn new(id: u8, hub: HubMutex) -> Self {
        let device_x: IoDevice;
        let device_y: IoDevice;
        // let hub_channels: lego_powered_up::hubs::Channels;
        {
            let mut lock = hub.lock().await;
            device_x = lock
                .io_from_port(ARM_ROT_ADDR)
                .expect("Error accessing LPU device");
            device_y = lock
                .io_from_port(ARM_EXTENSION_ADDR)
                .expect("Error accessing LPU device");
            // hub_channels = lock.channels().clone();
        }
        Self {
            id,
            device_x,
            device_y,
            pos_x: Arc::new(RwLock::new(0)), // get from save
            pos_y: Arc::new(RwLock::new(0)), // get from save
            feedback_task: None,
            cmd_task: None,
            cancel: CancellationToken::new(),
            // hub_channels,
        }
    }
    async fn arm_cmd(
        &self,
        mut rx_cmd: broadcast::Receiver<ArmCmd>,
        cancel: CancellationToken,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let device_x = self.device_x.clone();
        let device_y = self.device_y.clone();
        Ok(tokio::spawn(async move {
            println!("Spawned arm cmd");
            while let Ok(data) = rx_cmd.recv().await {
                match data {
                    ArmCmd::Stop => {
                        let _ = device_x.start_power(Power::Brake);
                        let _ = device_y.start_power(Power::Brake);
                    }
                    ArmCmd::StopX => {
                        let _ = device_x.start_power(Power::Brake);
                    }
                    ArmCmd::StopY => {
                        let _ = device_y.start_power(Power::Brake);
                    }
                    ArmCmd::Confirm => {}
                    ArmCmd::StartX { speed } => {
                        // Sign reversal on speed because gearing inverts expected movement direction
                        let _ = device_x.start_speed(-speed, 20);
                    }
                    ArmCmd::StartY { speed } => {
                        let _ = device_y.start_speed(speed, 20);
                    }
                    ArmCmd::Goto { x, y } => {
                        let _ = device_x.goto_absolute_position(x, 20, 20, EndState::Brake);
                        // .await;
                        let _ = device_y.goto_absolute_position(y, 20, 20, EndState::Brake);
                        // .await;
                    }
                    ArmCmd::GotoX { x } => {
                        let _ = device_x.goto_absolute_position(x, 20, 20, EndState::Brake);
                        // .await;
                    }
                    ArmCmd::GotoY { y } => {
                        let _ = device_y.goto_absolute_position(y, 20, 20, EndState::Brake);
                        // .await;
                    }
                }
            }
        }))
    }

    async fn arm_feedback(
        &self,
        tx_axis_x: tokio::sync::broadcast::Sender<((i8, i32))>,
        tx_axis_y: tokio::sync::broadcast::Sender<((i8, i32))>,
        tx_control: grow::zone::arm::ControlFeedbackTx,
        cancel: CancellationToken,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let pos_x = self.pos_x.clone();
        let pos_y = self.pos_y.clone();
        let (mut rx_axis_x, _axis_x_task) = self
            .device_x
            .motor_combined_sensor_enable(1, 2)
            .await
            .unwrap();
        let (mut rx_axis_y, _axis_y_task) = self
            .device_y
            .motor_combined_sensor_enable(1, 2)
            .await
            .unwrap();
        // let mut rx_cmdfb = 
        //     self.hub_channels.commandfeedback_sender.as_ref().unwrap().subscribe();
        let (mut rx_control_x, _control_x_task) = self
            .device_x
            .cmd_feedback_handler()
            .expect("Failed to get command feedback receiver");
        let (mut rx_control_y, _control_y_task) = self
            .device_y
            .cmd_feedback_handler()
            .expect("Failed to get command feedback receiver");
        let mut state_x: BufferState = Default::default();
        let mut state_y: BufferState = Default::default();
        // let state_arm = grow::zone::arm::ArmState::Idle;

        Ok(tokio::spawn(async move {
            println!("Spawned arm feedback");
            loop {
                tokio::select! {
                    Ok(data) = rx_axis_x.recv() => {
                            tx_axis_x.send( data );
                            *pos_x.write() = data.1;
                    }
                    Ok(data) = rx_axis_y.recv() => {
                            tx_axis_y.send( data );
                            *pos_y.write() = data.1;
                    }
                    Ok(data) = rx_control_x.recv() => {
                        state_x = data.state;
                        if (state_x == BufferState::Idle) & (state_y == BufferState::Idle) {
                            tx_control.send(ArmState::Idle);
                        }
                        else {
                            tx_control.send(ArmState::Busy);
                        }    
                        // println!("Cmdfb X: {:?} States:{:?}{:?}", &data, &state_x, &state_y);
                    }
                    Ok(data) = rx_control_y.recv() => {
                        state_y = data.state;
                        if (state_x == BufferState::Idle) & (state_y == BufferState::Idle) {
                            tx_control.send(ArmState::Idle);
                        }
                        else {
                            tx_control.send(ArmState::Busy);
                        }
                        // println!("Cmdfb Y: {:?} State X:{:?} Y:{:?}", &data, &state_x, &state_y);
                    }
                    _ = cancel.cancelled() => {
                        println!("Arm feedback task canceled");
                        break;
                    }
                    else => { break }
                };
            }
        }))
    }
}

pub struct LpuHub {
    id: u8,
    hub: HubMutex,
    feedback_task: Option<JoinHandle<()>>,
    cancel: CancellationToken,
}
#[async_trait]
impl zone::auxiliary::AuxDevice for LpuHub {
    fn id(&self) -> u8 {
        self.id
    }
    async fn init(
        &mut self,
        tx_status: tokio::sync::broadcast::Sender<(u8, DisplayStatus)>,
    ) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.hub_feedback(tx_status)
                .await
                .expect("Error initializing feedback task"),
        );
        Ok(())
    }
    fn read(&self) -> Result<String, Box<dyn Error>> {
        Ok(String::from("Placeholder"))
    }
}
impl LpuHub {
    pub fn new(id: u8, hub: HubMutex, cancel: CancellationToken) -> Self {
        Self {
            id,
            hub,
            feedback_task: None,
            cancel,
        }
    }

    async fn hub_feedback(
        &self,
        tx: broadcast::Sender<(u8, DisplayStatus)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let hub = self.hub.clone();
        let mut rx_hub: broadcast::Receiver<HubNotification>;
        {
            //     let mut lock = tokio::task::block_in_place(move || {
            //         hub.blocking_lock_owned()
            //     });
            let mut lock = hub.lock().await;
            rx_hub = lock
                .channels()
                .hubnotification_sender
                .as_ref()
                .unwrap()
                .subscribe();
            // These will send current status when enabling updates
            let _ = lock.hub_props(
                HubPropertyRef::Button,
                HubPropertyOperation::EnableUpdatesDownstream,
            )?;
            let _ = lock.hub_props(
                HubPropertyRef::BatteryType,
                HubPropertyOperation::EnableUpdatesDownstream,
            )?;
            let _ = lock.hub_props(
                HubPropertyRef::Rssi,
                HubPropertyOperation::EnableUpdatesDownstream,
            )?;
            let _ = lock.hub_props(
                HubPropertyRef::BatteryVoltage,
                HubPropertyOperation::EnableUpdatesDownstream,
            )?;

            // These will not send current status when enabling updates; request single update first
            let _ = lock.hub_alerts(AlertType::LowVoltage, AlertOperation::RequestUpdate)?;
            let _ = lock.hub_alerts(AlertType::LowVoltage, AlertOperation::EnableUpdates)?;

            let _ = lock.hub_alerts(AlertType::HighCurrent, AlertOperation::RequestUpdate)?;
            let _ = lock.hub_alerts(AlertType::HighCurrent, AlertOperation::EnableUpdates)?;

            let _ = lock.hub_alerts(AlertType::LowSignalStrength, AlertOperation::RequestUpdate)?;
            let _ = lock.hub_alerts(AlertType::LowSignalStrength, AlertOperation::EnableUpdates)?;

            let _ =
                lock.hub_alerts(AlertType::OverPowerCondition, AlertOperation::RequestUpdate)?;
            let _ =
                lock.hub_alerts(AlertType::OverPowerCondition, AlertOperation::EnableUpdates)?;
        }

        let cancel_clone = self.cancel.clone();
        Ok(tokio::spawn(async move {
            println!("Spawned hub feedback");
            loop {
                tokio::select! {
                    Ok(data) = rx_hub.recv() => {
                        // println!("Hub {:?} sent: {:?}", id, data,);
                        match data {
                            HubNotification {
                                hub_alert:
                                    Some(HubAlert {
                                        alert_type,
                                        payload,
                                        ..
                                    }),
                                    ..
                            } if payload == AlertPayload::Alert => {
                                tx.send(( id, DisplayStatus::new(Indicator::Red, Some(alert_type.to_string())) ));
                            },

                            HubNotification {
                                hub_property:
                                    Some(HubProperty {
                                        property,
                                        operation,
                                        ..
                                    }),
                                    ..
                            } if operation == HubPropertyOperation::UpdateUpstream => {
                                match property {
                                    HubPropertyValue::Button(state) =>  {

                                    }
                                    HubPropertyValue::BatteryType(t) => {

                                    }
                                    HubPropertyValue::BatteryVoltage(v) => {
                                        if v < 15 {
                                            tx.send(( id, DisplayStatus::new(Indicator::Red, Some( format!("Battery: {}%", v) )) ));
                                        }
                                        else if v < 30 {
                                            tx.send(( id, DisplayStatus::new(Indicator::Yellow, Some( format!("Battery: {}%", v) )) ));
                                        }
                                        else {
                                            tx.send(( id, DisplayStatus::new(Indicator::Green, Some( format!("Battery: {}%", v) )) ));
                                        }

                                    }
                                    HubPropertyValue::Rssi(signal) => {

                                    }
                                    _ => {}
                                }

                            }
                            _ => {}
                        }
                    }
                    _ = cancel_clone.cancelled() => {
                        match hub.lock().await.shutdown().await {
                            Ok(_) => { println!("LPU hub off"); }
                            Err(e) => { println!("LPU hub shutdown error: {:?}", e); }
                        }
                        match hub.lock().await.disconnect().await {
                            Ok(_) => { println!("LPU hub disconnected"); }
                            Err(e) => { println!("LPU hub disconnect error: {:?}", e); }
                        }
                        break;
                    }
                    else => { break }
                };
            }
        }))
    }
}


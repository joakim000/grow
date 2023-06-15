#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move {
    pub arm_id: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
pub struct Interface {
    arm: Box<dyn Arm>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Status {}

trait Arm {
    fn goto(&self, x: i32, y: i32) -> anyhow::Result<()>;
    fn confirm(&self, x: i32, y: i32) -> anyhow::Result<()>;
}

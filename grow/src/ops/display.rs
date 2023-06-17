

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Indicator {
    #[default] Blue,
    Green,
    Yellow,
    Red,
}


#[derive(Clone, Debug, PartialEq)]
pub struct DisplayStatus {
    pub indicator: Indicator,
    pub msg: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ZoneDisplay {
    Air {id: u8, info: DisplayStatus},
    Light {id: u8, info: DisplayStatus},
    Irrigation {id: u8, info: DisplayStatus},
    Arm {id: u8, info: DisplayStatus},
    Pump {id: u8, info: DisplayStatus},
    Tank {id: u8, info: DisplayStatus},
}


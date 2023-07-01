use core::fmt;
use time::OffsetDateTime;
use time::format_description::well_known::{Rfc3339, Rfc2822};
use crate::zone::*;
use crate::TIME_OFFSET;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Default)]
pub enum Indicator {
    #[default] Blue,
    Green,
    Yellow,
    Red,
}
#[rustfmt::skip]
impl fmt::Display for Indicator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant = match self  {
            Indicator::Blue =>   "\x1b[94m  Blue\x1b[0m",
            Indicator::Green =>  "\x1b[92m Green\x1b[0m",
            Indicator::Yellow => "\x1b[93mYellow\x1b[0m",
            Indicator::Red =>    "\x1b[91m   Red\x1b[0m",
        };
        
        write!(
            f, "{}", variant
        )
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct DisplayStatus {
    pub indicator: Indicator,
    pub msg: Option<String>,
    pub changed: OffsetDateTime,
}
impl DisplayStatus {
    pub fn new(indicator: Indicator, msg: Option<String>) -> Self {
        Self { indicator, msg, changed:OffsetDateTime::now_utc().to_offset(TIME_OFFSET)  }
    }
}
impl fmt::Display for DisplayStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.msg {
            None => {
                write!(
                    f,
                    "{:>5} No message",
                    self.indicator,
                )
            }, 
            Some(inner) => {
                write!(
                    f,
                    "{:>5} {}",
                    self.indicator,
                    inner,
                )
            }
        }
    }
}

#[rustfmt::skip]
impl fmt::Display for ZoneDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoneDisplay::Air { id, info } =>        { write!(f, "   Air {} {} \t{}", id, info, info.changed ) },
            ZoneDisplay::Light { id, info } =>      { write!(f, " Light {} {} \t{}", id, info, info.changed ) },
            ZoneDisplay::Water { id, info } =>      { write!(f, " Water {} {} \t{}", id, info, info.changed ) },
            ZoneDisplay::Aux { id, info } =>        { write!(f, "   Aux {} {} \t{}", id, info, info.changed ) },
            ZoneDisplay::Arm { id, info } =>        { write!(f, "   Arm {} {} \t{}", id, info, info.changed ) },
            ZoneDisplay::Pump { id, info } =>       { write!(f, "  Pump {} {} \t{}", id, info, info.changed ) },
            ZoneDisplay::Tank { id, info } =>       { write!(f, "  Tank {} {} \t{}", id, info, info.changed ) },
        }
    }
}
impl fmt::Display for ZoneLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoneLog::Air { id, temp, fan_rpm, changed_status } => {
                let temp_text = match temp {
                    None => {format!("None" )},
                    Some(temp) => {
                        format!("{:.1}", temp)
                    },
                };  
                let fan_text = match fan_rpm {
                    None => {format!("None" )},
                    Some(rpm) => {
                        format!("{:.0}", rpm)
                    },
                };      
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Air {} {{Temp {}°C, Fan {} rpm, Status change: {} }}",
                        id,temp_text,fan_text,status_text
                )    
            },
            ZoneLog::Light { id, lamp_on, light_level, changed_status } => {
                let lamp_text = match lamp_on {
                    None => {format!("None" )},
                    Some(x) => {
                        format!("{:?}", x)
                    },
                };  
                let light_text = match light_level {
                    None => {format!("None" )},
                    Some(x) => {
                        format!("{:.0}", x)
                    },
                };      
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Light {} {{Lamp {}, Light level {}, Status change: {} }}",
                        id,lamp_text,light_text,status_text
                )    
            },
            ZoneLog::Water { id, moisture, changed_status } => {
                let moist_text = match moisture {
                    None => {format!("None" )},
                    Some(x) => {
                        format!("{:?}", x)
                    },
                };  
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Water {} {{Moisture {}, Status change: {} }}",
                        id,moist_text,status_text
                )    
            },
            ZoneLog::Aux { id,  changed_status } => {
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Aux {} {{Status change: {} }}",
                        id, status_text
                )    
            }
            ZoneLog::Arm { id, x:_, y:_, z:_, changed_status } => {
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Arm {} {{Status change: {} }}",
                        id, status_text
                )    
            }
            ZoneLog::Pump { id,  changed_status } => {
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Pump {} {{Status change: {} }}",
                        id, status_text
                )    
            }
            ZoneLog::Tank { id,  changed_status } => {
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Tank {} {{Status change: {} }}",
                        id, status_text
                )    
            }
        }
    }
}
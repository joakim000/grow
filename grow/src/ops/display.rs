use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Indicator {
    #[default] Blue,
    Green,
    Yellow,
    Red,
}
impl fmt::Display for Indicator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant = match self  {
            Indicator::Blue => "Blue",
            Indicator::Green => "Green",
            Indicator::Yellow => "Yellow",
            Indicator::Red => "Red",
        };
        
        write!(
            f, "{}", variant
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayStatus {
    pub indicator: Indicator,
    pub msg: Option<String>,
}
impl fmt::Display for DisplayStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.msg {
            None => {
                write!(
                    f,
                    "[{}] No message",
                    self.indicator,
                )
            }, 
            Some(inner) => {
                write!(
                    f,
                    "[{}] {}",
                    self.indicator,
                    inner,
                )
            }
        }
    }
}



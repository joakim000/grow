use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Indicator {
    #[default] Blue,
    Green,
    Yellow,
    Red,
}
// impl fmt::Display for Indicator {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let variant = match self  {
            
//         };
        
//         write!(
//             f,
//             "{:#?} on port {} ({:#x}) with {} modes: {:#?}",
//             self.kind,
          
          
//         )
//     }
// }

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayStatus {
    pub indicator: Indicator,
    pub msg: Option<String>,
}
// impl DisplayStatus {
//     pub fn indicator(&self) -> Indicator {
//         self.indicator.clone()
//     }
// }

// impl fmt::Display for DisplayStatus {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "{:#?} on port {} ({:#x}) with {} modes: {:#?}",
//             self.kind,
          
          
//         )
//     }
// }

// impl fmt::Display for DisplayStatus {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "{:#?} on port {} ({:#x}) with {} modes: {:#?}",
//             self.kind,
//             self.port,
//             self.port,
//             self.mode_count,
//             self.modes
//                 .values()
//                 .map(|mode| &mode.name[..])
//                 .collect::<Vec<_>>()
//         )
//     }
// }


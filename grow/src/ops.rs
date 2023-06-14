#![allow(unused)]

extern crate alloc;
use super::House;
// use super::HouseMapped;
use crate::zone;
use crate::zone::Zone;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use anyhow::*;

pub mod conf;
pub mod running;
// mod warning;

pub mod warning {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {}

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}
}

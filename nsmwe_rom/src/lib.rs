#![allow(clippy::identity_op)]

extern crate nom;
extern crate num_enum;
extern crate polyerror;

pub mod addr;
pub mod compression;
pub mod error;
pub mod graphics;
pub mod internal_header;
pub mod level;
pub mod rom;

pub use crate::{
    internal_header::RomInternalHeader,
    rom::{
        Rom,
        SMC_HEADER_SIZE,
    },
};

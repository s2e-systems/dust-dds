#![no_std]

use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType)]
pub struct ColorState {
    pub on: bool,
}

#[derive(DdsType)]
pub struct ButtonState {
    pub pressed: bool,
}

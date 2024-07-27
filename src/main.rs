use std::{fs::ReadDir, ops::Not};

use ftdi_mpsse::MpsseCmdBuilder;
use ftdi_spi::{
    flash_cmds::{common::*, SpiSession},
    spi::*,
};

fn main() {
    let mut spi = SPIComm::new(ftdi::Interface::A).expect("Device Open Failed");
    let spi_cmd = ReadID {
        mode: ReadIDMode::ManufacturerDevice,
    };

    let ret = spi.issue_commands(&[spi_cmd]).expect("SPI Comm Error");
    println!(
        "{}",
        ret.iter()
            .map(|x: &u8| format!("{:#x}", x))
            .collect::<Vec<String>>()
            .concat()
    )
}

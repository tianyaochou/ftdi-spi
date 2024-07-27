use derive_more::From;
use ftdi::*;
use ftdi_mpsse::*;

use crate::flash_cmds::SpiSession;

pub struct SPIComm {
    freq: u32,
    device: Device,
}

#[derive(From, Debug)]
pub enum Error {
    FDTIErr(ftdi::Error),
    IOError(std::io::Error),
    MpsseInitErr,
    BufferLengthExceeded,
}

type Result<T> = std::result::Result<T, Error>;

const MAX_LENGTH: usize = 1usize + u16::MAX as usize;
pub const SPI_CS_ENABLE: u8 = 0x00;
pub const SPI_CS_DISABLE: u8 = 0x08;
pub const SPI_DIRECTION: u8 = 0xFB;

impl SPIComm {
    pub fn new(interface: Interface) -> Result<SPIComm> {
        // 0x6010 for FT2232X 0x6011 for FT4232H and 0x6014 for FT232H
        // TODO: multi device support
        let opener = find_by_vid_pid(0x0403, 0x6014).interface(interface);
        let mut device = opener.open()?;
        let mut mpsse_setting = MpsseSettings::default();
        mpsse_setting.reset = true;
        device.init(&mpsse_setting)?;
        let freq = 20_000;
        device.set_mpsse_clock(freq)?;
        let cmds = MpsseCmdBuilder::new()
            // HACK: Setting mask when using init has no effect, have to send command mannually here
            // Setting DBUS0,1,3-7 as output and DBUS1 as input, and set CS to high
            .set_gpio_lower(SPI_CS_DISABLE, SPI_DIRECTION)
            .send_immediate();
        device.send(cmds.as_slice())?;
        Ok(SPIComm { freq, device })
    }

    pub fn issue_commands(&mut self, cmds: &[impl SpiSession]) -> Result<Vec<u8>> {
        let mut mpsse_cmds = MpsseCmdBuilder::new();
        let mut len = 0;
        for cmd in cmds {
            mpsse_cmds = cmd.issue(mpsse_cmds);
            len = len + cmd.read_length();
        }
        let mut buffer: Vec<u8> = Vec::new();
        buffer.reserve_exact(len);
        self.write_and_read(mpsse_cmds.send_immediate().as_slice(), &mut buffer, len)?;
        Ok(buffer)
    }

    pub fn write_and_read(&mut self, cmd: &[u8], data_out: &mut [u8], len: usize) -> Result<()> {
        self.device.send(cmd)?;
        if len > 0 {
            self.device.recv(data_out)?;
        }
        Ok(())
    }

    pub fn read_write(&mut self, data: &[u8], buffer: &mut [u8]) -> Result<()> {
        if data.len() > MAX_LENGTH {
            return Err(Error::BufferLengthExceeded);
        }
        let cmds = MpsseCmdBuilder::new()
            .clock_data(ClockData::MsbPosIn, data)
            .send_immediate();
        self.device.send(cmds.as_slice())?;
        self.device.recv(buffer)?;
        Ok(())
    }
}

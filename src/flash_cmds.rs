use ftdi_mpsse::MpsseCmdBuilder;

use crate::spi::{SPI_CS_DISABLE, SPI_CS_ENABLE, SPI_DIRECTION};

pub trait SpiSession {
    fn command(&self) -> impl Iterator<Item = u8>;
    fn read_length(&self) -> usize;
    fn issue(&self, cmds: MpsseCmdBuilder) -> MpsseCmdBuilder {
        let buffer: Vec<u8> = self.command().collect();
        let mut new_cmds = cmds
            .set_gpio_lower(SPI_CS_ENABLE, SPI_DIRECTION)
            .clock_data_out(ftdi_mpsse::ClockDataOut::MsbNeg, &buffer);
        if self.read_length() > 0 {
            new_cmds = new_cmds.clock_data_in(ftdi_mpsse::ClockDataIn::MsbPos, self.read_length())
        }
        new_cmds.set_gpio_lower(SPI_CS_DISABLE, SPI_DIRECTION)
    }
}

// trait SpiIssue<T: SpiSession> {
//     fn issue(self, session: T) -> Self;
// }

// impl SpiIssue<impl SpiSession> for MpsseCmdBuilder {
//     fn issue(self, session: impl SpiSession) -> Self {
//         session.issue(self)
//     }
// }

pub trait OpCode<Op> {
    fn op(&self) -> Op;
}

pub mod common {

    use std::iter::once;

    use super::{OpCode, SpiSession};

    type Addr = [u8; 3];
    type Op = u8;

    // Read
    pub struct Read {
        pub addr: Addr,
        pub read_length: usize,
    }

    impl OpCode<Op> for Read {
        fn op(&self) -> Op {
            0x03
        }
    }

    impl SpiSession for Read {
        fn command(&self) -> impl Iterator<Item = u8> {
            std::iter::once(self.op()).chain(self.addr.iter().cloned())
        }
        fn read_length(&self) -> usize {
            self.read_length
        }
    }

    pub struct FastRead {
        pub addr: Addr,
        pub read_length: usize,
    }

    impl OpCode<Op> for FastRead {
        fn op(&self) -> Op {
            0x0B
        }
    }

    impl SpiSession for FastRead {
        fn command(&self) -> impl Iterator<Item = u8> {
            once(self.op())
                .chain(self.addr.iter().cloned())
                .chain(once(0x00))
        }
        fn read_length(&self) -> usize {
            self.read_length
        }
    }

    pub struct SectorErase {
        addr: Addr,
    }

    impl OpCode<Op> for SectorErase {
        fn op(&self) -> Op {
            0x20
        }
    }

    impl SpiSession for SectorErase {
        fn command(&self) -> impl Iterator<Item = u8> {
            once(self.op()).chain(self.addr.iter().cloned())
        }
        fn read_length(&self) -> usize {
            0
        }
    }

    pub struct BlockErase32K {
        addr: Addr,
    }

    impl OpCode<Op> for BlockErase32K {
        fn op(&self) -> Op {
            0x52
        }
    }

    impl SpiSession for BlockErase32K {
        fn command(&self) -> impl Iterator<Item = u8> {
            once(self.op()).chain(self.addr.iter().cloned())
        }
        fn read_length(&self) -> usize {
            0
        }
    }

    pub struct ChipErase;

    impl OpCode<Op> for ChipErase {
        fn op(&self) -> Op {
            0x60 // Alterative: 0xC7
        }
    }

    impl SpiSession for ChipErase {
        fn command(&self) -> impl Iterator<Item = u8> {
            once(self.op())
        }
        fn read_length(&self) -> usize {
            0
        }
    }

    pub struct ReadStatus;

    impl OpCode<Op> for ReadStatus {
        fn op(&self) -> Op {
            0x05
        }
    }

    impl SpiSession for ReadStatus {
        fn command(&self) -> impl Iterator<Item = u8> {
            once(self.op())
        }
        fn read_length(&self) -> usize {
            1
        }
    }

    pub struct WriteEnable;

    impl OpCode<Op> for WriteEnable {
        fn op(&self) -> Op {
            0x06
        }
    }

    impl SpiSession for WriteEnable {
        fn command(&self) -> impl Iterator<Item = u8> {
            once(self.op())
        }
        fn read_length(&self) -> usize {
            0
        }
    }

    pub struct WriteDisable;

    impl OpCode<Op> for WriteDisable {
        fn op(&self) -> Op {
            0x04
        }
    }

    impl SpiSession for WriteDisable {
        fn command(&self) -> impl Iterator<Item = u8> {
            once(self.op())
        }
        fn read_length(&self) -> usize {
            0
        }
    }

    pub struct ByteProgram {
        addr: Addr,
        data: u8,
    }

    impl OpCode<Op> for ByteProgram {
        fn op(&self) -> Op {
            0x02
        }
    }

    impl SpiSession for ByteProgram {
        fn command(&self) -> impl Iterator<Item = u8> {
            once(self.op())
                .chain(self.addr.iter().cloned())
                .chain(once(self.data))
        }
        fn read_length(&self) -> usize {
            0
        }
    }

    #[derive(Clone, Copy)]
    pub enum ReadIDMode {
        ManufacturerDevice = 0x00,
        DeviceManufacturer = 0x01,
    }

    pub struct ReadID {
        pub mode: ReadIDMode,
    }

    impl OpCode<Op> for ReadID {
        fn op(&self) -> Op {
            0x90
        }
    }

    impl SpiSession for ReadID {
        fn command(&self) -> impl Iterator<Item = u8> {
            once(self.op())
                .chain([0x00u8, 0x00u8].iter().cloned())
                .chain(once(self.mode as u8))
        }
        fn read_length(&self) -> usize {
            2
        }
    }
}

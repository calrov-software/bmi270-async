use crate::types::Error;
use embedded_hal::{i2c, spi};
use embedded_hal::digital::OutputPin;

const BMI270_I2C_ADDR: u8 = 0x68;

pub struct I2cInterface<I2C> {
    pub i2c: I2C,
    pub address: u8,
}

pub struct SpiInterface<SPI, CS> {
    pub spi: SPI,
    pub cs: CS,
}

/// I2c address.
pub enum I2cAddr {
    /// Use the default i2c address, 0x68.
    Default,
    /// Use alternative 0x69 as the i2c address. (selected when SDO is pulled high).
    Alternative,
}

impl Default for I2cAddr {
    fn default() -> Self {
        I2cAddr::Default
    }
}

impl I2cAddr {
    pub fn addr(self) -> u8 {
        match self {
            I2cAddr::Default => BMI270_I2C_ADDR,
            I2cAddr::Alternative => BMI270_I2C_ADDR | 1,
        }
    }
}

pub trait WriteData {
    type Error;
    fn write(&mut self, payload: &mut [u8]) -> Result<(), Self::Error>;
    fn write_reg(&mut self, register: u8, data: u8) -> Result<(), Self::Error>;
}

pub trait ReadData {
    type Error;
    fn read(&mut self, payload: &mut [u8]) -> Result<(), Self::Error>;
    fn read_reg(&mut self, register: u8) -> Result<u8, Self::Error>;
}

impl<I2C, E> WriteData for I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    type Error = Error<E, ()>;
    fn write(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c
            .write(self.address, payload)
            .map_err(Error::Comm)
    }

    fn write_reg(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [register, data];
        self.i2c
            .write(self.address, &payload)
            .map_err(Error::Comm)
    }
}

impl<SPI, CS, CommE, CsE> WriteData for SpiInterface<SPI, CS>
where
    SPI: spi::SpiDevice<u8, Error = CommE>,
    CS: OutputPin<Error = CsE>,
{
    type Error = Error<CommE, CsE>;
    fn write(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        payload[0] += 0x80;

        self.cs.set_low().map_err(Error::Cs)?;
        let res = self.spi.write(&payload).map_err(Error::Comm);
        self.cs.set_high().map_err(Error::Cs)?;

        res
    }

    fn write_reg(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [register + 0x80, data];

        self.cs.set_low().map_err(Error::Cs)?;
        let res = self.spi.write(&payload).map_err(Error::Comm);
        self.cs.set_high().map_err(Error::Cs)?;

        res
    }
}

impl<I2C, E> ReadData for I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    type Error = Error<E, ()>;
    fn read(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c
            .write_read(self.address, &[payload[0]], &mut payload[1..])
            .map_err(Error::Comm)
    }

    fn read_reg(&mut self, register: u8) -> Result<u8, Self::Error> {
        let mut data = [0];
        self.i2c
            .write_read(self.address, &[register], &mut data)
            .map_err(Error::Comm)
            .and(Ok(data[0]))
    }
}

impl<SPI, CS, CommE, CsE> ReadData for SpiInterface<SPI, CS>
where
    SPI: spi::SpiDevice<u8, Error = CommE>,
    CS: OutputPin<Error = CsE>,
{
    type Error = Error<CommE, CsE>;
    fn read(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Cs)?;
        let res = self.spi.read(payload).map_err(Error::Comm);
        self.cs.set_high().map_err(Error::Cs)?;

        res?;
        Ok(())
    }

    fn read_reg(&mut self, register: u8) -> Result<u8, Self::Error> {
        let mut payload = [register, 0];
        let mut rxbuf = [0u8; 2];

        self.cs.set_low().map_err(Error::Cs)?;
        self.spi.transfer(&mut payload, &mut rxbuf).map_err(Error::Comm)?;
        self.cs.set_high().map_err(Error::Cs)?;

        Ok(rxbuf[1])
    }
}

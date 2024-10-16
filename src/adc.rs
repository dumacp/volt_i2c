extern crate i2c_linux;
use std::result;

use i2c_linux::I2c;

// use i2cdev::core::*;
// use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError, LinuxI2CMessage};

const SLAVE_ADDR: u16 = 0x54;

pub struct ADC {
    dev: I2c
}



pub enum FlagRegister {
    AlertHold = 0x10,
    AlertFlagEnable = 0x08,
    AlertPINEnable = 0x04,
    Polarity = 0x01,
    Tx32=0x20,
}

impl ADC {

    //New ADC
    pub fn new() -> Result<ADC> {
        let dev: I2c<std::fs::File> = I2c::from_path("/dev/i2c-2")?;
    
        Ok(ADC{dev: dev})
    }

    //set conf in ADC. Flags type -> FlagRegister, example (FlagRegister::AlertHold | FlagRegister::AlertPINEnable)
    pub fn set_conf_register(&mut self, flags: u8) -> Result<()> {
        self.dev.smbus_write_byte_data(0x02, flags)?;
        println!("register conf: {:#X}", flags);
        self.dev.smbus_write_byte_data(0x01, 0x00)?;
        Ok(())
    }

    pub fn set_alert_under_range(&mut self, value: f32) -> Result<()> {

        let value_u = (value/0.016).round() as u16 & 0x0FFF;    
        self.dev.smbus_write_word_data(0x03, value_u.to_be())?;
        Ok(())
    }

    pub fn set_alert_over_range(&mut self, value: f32) -> Result<()> {

        let value_u = (value/0.016).round() as u16 & 0x0FFF;      
        self.dev.smbus_write_word_data(0x04, value_u.to_be())?;
        Ok(())
    }

    pub fn set_alert_hysteresis(&mut self, value: f32) -> Result<()> {

        let value_u = (value/0.016).round() as u16 & 0x0FFF;
        self.dev.smbus_write_word_data(0x05, value_u.to_be())?;
        Ok(())
    }

    pub fn read_register_word(&mut self, addr: u8) -> Result<u16> {

        // let mut read_data: [u8; 2] = [0; 2];
        // let mut msgs = [
        //     LinuxI2CMessage::write(&[addr]),
        //     LinuxI2CMessage::read(&mut read_data)
        // ];
        // self.dev.transfer(&mut msgs)?;  


        // let result = read_data.iter().rev().enumerate().fold(0, |acc: u16, (i, x)| acc + (((*x as u16) & 0x00FF)  << i*8 ));

        let result = self.dev.smbus_read_word_data(addr)?;
        // println!("Reading: {:?}", read_data);
        println!("Reading: {:#X}", result);
        Ok(result)
    }

    pub fn read_register_byte(&mut self, addr: u8) -> Result<u8> {

        // let mut read_data: [u8; 1] = [0; 1];
        // let mut msgs = [
        //     LinuxI2CMessage::write(&[addr]),
        //     LinuxI2CMessage::read(&mut read_data)
        // ];
        // self.dev.transfer(&mut msgs)?;  

        // let result = read_data[0];

        let result = self.dev.smbus_read_byte_data(addr)?;
        // println!("Reading: {:?}", read_data);
        println!("Reading: {:#X}", result);
        Ok(result)
    }
 
    pub fn read_value(&mut self) -> Result<(f32, bool)> {
        let result = self.read_register_word(0x00)?;
        // println!("read_value: {:#X}", result);
        let alert = (result & 0x8000) == 0x8000;
        let value = result & 0x0FFF;
        Ok(((value as f32) * 0.016, alert))
    }

    pub fn read_min_value(&mut self) -> Result<f32> {

        let result = self.read_register_word(0x06)?;
        Ok(((result & 0x0FFF) as f32) * 0.016)
    }
    pub fn write_min_value(&mut self, value: f32) -> Result<()> {
        let value_u = (value/0.016).round() as u16 & 0x0FFF;
        self.dev.smbus_write_word_data(0x06, value_u.to_be())?;
        Ok(())
    }

    pub fn read_max_value(&mut self) -> Result<f32> {

        let result = self.read_register_word(0x07)?;
        Ok(((result & 0x0FFF) as f32) * 0.016)
    }
    pub fn write_max_value(&mut self, value: f32) -> Result<()> {
        let value_u = (value/0.016).round() as u16 & 0x0FFF;
        self.dev.smbus_write_word_data(0x07, value_u.to_be())?;
        Ok(())
    }

    //Result -> (bool, bool) = (over range, under range)
    pub fn read_alert(&mut self) -> Result<(bool, bool)> {

        let result = self.read_register_byte(0x01)?;
        Ok((result & 0x02 == 0x02, result & 0x01 == 0x01))
    }

    pub fn clear_alert_over(&mut self) -> Result<()> {
        self.dev.smbus_write_byte_data(0x01, 0x02)?;
        Ok(())
    }
    pub fn clear_alert_under(&mut self) -> Result<()> {
        self.dev.smbus_write_byte_data(0x01, 0x01)?;
        Ok(())
    }
    pub fn clear_alerts(&mut self) -> Result<()> {

        self.dev.smbus_write_byte_data(0x01, 0x03)?;
        Ok(())
    }
}

use std::{error::Error, thread::sleep, time::Duration};
use bmi270::{Bmi270, Burst, I2cAddr, PwrCtrl};
// use types::Data;
extern crate linux_embedded_hal as hal;

fn main() -> Result<(), Box<dyn Error>> {
  let update_freq: f32 = 40.; // [Hz]
  let update_interval = Duration::from_micros((1e6 / update_freq) as u64);

  let mut bmi = Bmi270::new_i2c(hal::I2cdev::new("/dev/i2c-2")?,
                                I2cAddr::Alternative,
                                Burst::Other(255));

  // Get the chip id. Should be 0x24 or 36 in decimal
  let chip_id = bmi.get_chip_id().unwrap();
  println!("chip_id: {}", chip_id);

  // Initialize the senor.
  // During this process a configuration of > 8kB is uploaded to the sensor.
  bmi.init().unwrap();

  // Enable power for the accelerometer and the gyroscope.
  let pwr_ctrl = PwrCtrl{ aux_en: false, gyr_en: true, acc_en: true, temp_en: false };
  bmi.set_pwr_ctrl(pwr_ctrl).unwrap();

  loop {
    // Read the raw data
    let data = bmi.get_data().unwrap();
    println!("{} {} {} {} {} {} {}",
             data.acc.x, data.acc.y, data.acc.z,
             data.gyr.x, data.gyr.y, data.gyr.z,
             data.time
             );
    sleep(update_interval);
  }

  Ok(())
}

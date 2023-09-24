use bme280::i2c::BME280;
use embassy_time::Delay;
use embedded_hal::i2c::I2c;

pub struct Thermometer<T: I2c> {
    device: BME280<T>,
    delay: Delay,
}

pub struct Measurement {
    /// temperature in degrees celsius
    pub temperature: f32,
    /// pressure in pascals
    pub pressure: f32,
    /// percent relative humidity
    pub humidity: f32,
}

impl<T: I2c> Thermometer<T> {
    pub fn new(bus: T) -> Self
    where
        T: I2c,
    {
        let mut device = BME280::new_primary(bus);
        let mut delay = Delay;
        device.init(&mut delay).unwrap();
        Self { device, delay }
    }

    pub fn measure(&mut self) -> Result<Measurement, ()> {
        let resp = self.device.measure(&mut self.delay).map_err(|_| ())?;
        Ok(Measurement {
            temperature: resp.temperature,
            humidity: resp.humidity,
            pressure: resp.pressure,
        })
    }
}

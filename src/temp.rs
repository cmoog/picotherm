use bme280::i2c::BME280;
use embassy_time::Delay;
use embedded_hal::i2c::I2c;

pub struct Thermometer<T: I2c> {
    device: BME280<T>,
    delay: Delay,
}

pub struct Measurement {
    /// temperature in degrees celsius
    pub temperature: Temperature,
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
            temperature: Temperature::from_celsius(resp.temperature),
            humidity: resp.humidity,
            pressure: resp.pressure,
        })
    }
}

pub struct Temperature(f32);

#[allow(dead_code)]
impl Temperature {
    pub fn celsius(&self) -> f32 {
        self.0
    }
    pub fn fahrenheit(&self) -> f32 {
        (self.0 * 9.0 / 5.0) + 32.0
    }

    pub fn from_fahrenheit(temp: f32) -> Self {
        Self((temp - 32.0) * 5.0 / 9.0)
    }

    pub fn from_celsius(temp: f32) -> Self {
        Self(temp)
    }
}

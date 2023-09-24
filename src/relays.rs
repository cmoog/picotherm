#![allow(unused)]

use embedded_hal::digital::OutputPin;

pub struct Relay<T: OutputPin> {
    pin: T,
}

impl<T: OutputPin> Relay<T> {
    pub fn new(pin: T) -> Self {
        Self { pin }
    }
    pub fn connect(&mut self) {
        self.pin.set_high();
    }

    pub fn disconnect(&mut self) {
        self.pin.set_low();
    }
}

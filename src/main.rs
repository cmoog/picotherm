#![no_std]
#![no_main]
#![feature(async_fn_in_trait, type_alias_impl_trait, never_type)]
#![allow(incomplete_features)]

use core::sync::atomic::{AtomicI32, Ordering};

use cyw43::Control;
use cyw43_pio::PioSpi;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    peripherals::*,
    pio::{self, Pio},
};
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_ncm::embassy_net::Device;
use embedded_hal::digital::{OutputPin, PinState};
use embedded_hal_async::i2c::I2c;
use panic_probe as _;
use relays::Relay;
use static_cell::make_static;

mod relays;
mod sevensegment;
mod temp;

use temp::Thermometer;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let (_net_device, control) = setup(
        &spawner, p.PIN_23, p.PIN_25, p.PIO0, p.PIN_24, p.PIN_29, p.DMA_CH0,
    )
    .await;

    unwrap!(spawner.spawn(display_task(
        p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, p.PIN_6, p.PIN_7, p.PIN_8, p.PIN_9, p.PIN_10,
    )));
    unwrap!(spawner.spawn(temp_task(p.I2C0, p.PIN_0, p.PIN_1)));
    unwrap!(spawner.spawn(control_task(p.PIN_15, p.PIN_16, p.PIN_17)));

    blink(control).await;
}

// TOOD: remove
async fn blink<'a>(mut c: Control<'a>) {
    let delay = Duration::from_secs(1);
    loop {
        info!("led on!");
        c.gpio_set(0, true).await;
        Timer::after(delay).await;

        info!("led off!");
        c.gpio_set(0, false).await;
        Timer::after(delay).await;
    }
}

// setup the cyw43 wifi chip by injecting the embedded firmware blob
// and initializing the respective pins
async fn setup(
    spawner: &Spawner,
    p23: PIN_23,
    p25: PIN_25,
    pio0: PIO0,
    p24: PIN_24,
    p29: PIN_29,
    dmacho: DMA_CH0,
) -> (Device<'_, 1514>, Control<'_>) {
    let fw = include_bytes!("../embassy/cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../embassy/cyw43-firmware/43439A0_clm.bin");

    let pwr = Output::new(p23, Level::Low);
    let cs = Output::new(p25, Level::High);
    let mut pio = Pio::new(pio0, Irqs);
    let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, p24, p29, dmacho);

    let state = make_static!(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(wifi_task(runner)));
    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    (net_device, control)
}

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

// primary bang-bang control loop for maintaining target temp
#[embassy_executor::task]
async fn control_task(pin_ac: PIN_15, pin_heat: PIN_16, pin_fan: PIN_17) {
    use relays::Relay;
    let ac = Relay::new(Output::new(pin_ac, Level::Low));
    let heater = Relay::new(Output::new(pin_heat, Level::Low));
    let fan = Relay::new(Output::new(pin_fan, Level::Low));
    run_control_loop(ac, heater, fan).await;
}

// a background task for measuring the temp of the room and updating the global state
#[embassy_executor::task]
async fn temp_task(i2c0: I2C0, sda: PIN_0, scl: PIN_1) {
    use embassy_rp::i2c::{self, I2c};
    bind_interrupts!(struct I2c0Irqs {
        I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    });
    let device = I2c::new_async(i2c0, scl, sda, I2c0Irqs, Default::default());
    temp_reader(Thermometer::new(device).await).await;
}

#[embassy_executor::task]
async fn display_task(
    a: PIN_2,
    b: PIN_3,
    c: PIN_4,
    d: PIN_5,
    e: PIN_6,
    f: PIN_7,
    g: PIN_8,
    display1: PIN_9,
    display2: PIN_10,
) -> ! {
    let delay_dur = Duration::from_millis(1);

    let mut display1 = Output::new(display1, Level::Low);
    let mut display2 = Output::new(display2, Level::Low);
    let mut common = sevensegment::SevenSeg::new(
        PinState::High,
        Output::new(a, Level::Low),
        Output::new(b, Level::Low),
        Output::new(c, Level::Low),
        Output::new(d, Level::Low),
        Output::new(e, Level::Low),
        Output::new(f, Level::Low),
        Output::new(g, Level::Low),
    );
    common.clear().unwrap();

    loop {
        let temp = THERMOMTER_READING.load(Ordering::Relaxed);
        let (digit1, digit2) = digits(temp as u8);
        display1.set_low();
        display2.set_high();
        common.display(digit1).unwrap();
        Timer::after(delay_dur).await;

        display1.set_high();
        display2.set_low();
        common.display(digit2).unwrap();
        Timer::after(delay_dur).await;
    }
}

async fn temp_reader<T: I2c>(mut dev: Thermometer<T>) {
    loop {
        let resp = dev.measure().await.unwrap();
        info!("measured the temperature = {} deg C", resp.temperature);
        THERMOMTER_READING.store(resp.temperature.fahrenheit() as i32, Ordering::Relaxed);
        Timer::after(Duration::from_secs(5)).await;
    }
}

// Separate a two digit number into each digit,
// any input >= 99 returns (9, 9).
fn digits(mut n: u8) -> (u8, u8) {
    if n > 99 {
        n = 99
    }
    (n / 10, n % 10)
}

static TARGET_TEMP: AtomicI32 = AtomicI32::new(20);
static THERMOMTER_READING: AtomicI32 = AtomicI32::new(0);

#[derive(PartialEq, Copy, Debug, Clone)]
enum ControlMode {
    /// Use both the AC and heater to bring the measured temperature within
    /// range of the target temperature.
    Auto,
    /// Only use the AC to bring the temperature down to the target value.
    Cool,
    /// Only use the heater to bring the temperature up to the target value.
    Heat,
    /// Disable AC and heater.
    #[allow(dead_code)]
    Off,
}

#[derive(PartialEq, Copy, Debug, Clone)]
enum ControlState {
    Heating,
    Cooling,
    Nothing,
}

#[derive(PartialEq, Copy, Debug, Clone)]
enum FanMode {
    #[allow(dead_code)]
    On,
    Off,
}

fn new_state(current_temp: i32, target_temp: i32, mode: ControlMode) -> ControlState {
    use ControlMode::{Auto, Cool, Heat};
    use ControlState::{Cooling, Heating, Nothing};
    const ALLOWANCE: i32 = 1;
    if current_temp < target_temp - ALLOWANCE && (mode == Heat || mode == Auto) {
        Heating
    } else if current_temp > target_temp + ALLOWANCE && (mode == Cool || mode == Auto) {
        Cooling
    } else {
        Nothing
    }
}

async fn run_control_loop<AC, HE, FAN>(
    mut ac: Relay<AC>,
    mut heater: Relay<HE>,
    mut fan: Relay<FAN>,
) where
    AC: OutputPin,
    HE: OutputPin,
    FAN: OutputPin,
{
    // hard coded for now, should be runtime configurable by Matter API
    let mode = ControlMode::Auto;
    let fan_state = FanMode::Off;

    loop {
        let current_temp = THERMOMTER_READING.load(Ordering::Relaxed);
        let target_temp = TARGET_TEMP.load(Ordering::Relaxed);

        match new_state(current_temp, target_temp, mode) {
            ControlState::Heating => {
                ac.disconnect();
                heater.connect();
            }
            ControlState::Cooling => {
                ac.connect();
                heater.disconnect();
            }
            ControlState::Nothing => {
                ac.disconnect();
                heater.disconnect();
            }
        };
        match fan_state {
            FanMode::On => {
                fan.connect();
            }
            FanMode::Off => {
                fan.disconnect();
            }
        };
        Timer::after(Duration::from_secs(2)).await;
    }
}

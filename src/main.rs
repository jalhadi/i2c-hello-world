#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m;
use cortex_m_rt::entry;

use stm32f3xx_hal::{self, prelude::*, pac, i2c, stm32};

mod lib;

use lib::enums::*;

const LCD_ADDRESS: u8 = 0x27;



// figure out cortex_m::asm::delay
fn delay(_i: u32) {
    let mut i = _i;
    while i > 0 {
        i -= 1;
    }
}

type Pins = (stm32f3xx_hal::gpio::gpiob::PB6<stm32f3xx_hal::gpio::AF4>, stm32f3xx_hal::gpio::gpiob::PB7<stm32f3xx_hal::gpio::AF4>);

fn write_data(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8) {
    i2c.write(LCD_ADDRESS, &[data | 0x08]);
}

fn pulse_enable(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8) {
    i2c.write(LCD_ADDRESS, &[data | En | 0x08]);
    delay(10_000);
    i2c.write(LCD_ADDRESS, &[(data & !En) | 0x08]);
    delay(50_000);
}

fn write4bits(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8) {
    write_data(i2c, data);
    pulse_enable(i2c, data);
}

fn send(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8, mode: u8) {
    let high_bits: u8 = data & 0xf0;
    let low_bits: u8 = (data << 4) & 0xf0;
    write4bits(i2c, high_bits | mode);
    write4bits(i2c, low_bits | mode);
}

// What is mode?
fn write(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8) {
    send(i2c, data, 0x01);
}

fn command(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8) {
    send(i2c, data, 0x00);
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let mut i2c = i2c::I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks, &mut rcc.apb1);

    // Adapted from https://github.com/fdebrabander/Arduino-LiquidCrystal-I2C-library/blob/master/LiquidCrystal_I2C.cpp
    write4bits(&mut i2c, 0x03 << 4);
    delay(10_000);
    write4bits(&mut i2c, 0x03 << 4);
    delay(10_000);
    write4bits(&mut i2c, 0x03 << 4);
    delay(10_000);
    write4bits(&mut i2c, 0x02 << 4);

    command(
        &mut i2c,
        Command::FunctionSet as u8 |
        FunctionMode::Bit4 as u8 |
        FunctionDots::Dots5x8 as u8 |
        FunctionLine::Line2 as u8
    );


    command(
        &mut i2c,
        Command::DisplayControl as u8 |
        DisplayMode::DisplayOn as u8 |
        DisplayCursor::CursorOn as u8 |
        DisplayBlink::BlinkOn as u8
    );

    command(
        &mut i2c,
        Command::ClearDisplay as u8
    );

    command(
        &mut i2c,
        Command::EntryModeSet as u8 |
        EntryModeDirection::EntryRight as u8
    );

    let hello: &'static str = "Hello, world!";
    for c in hello.chars() {
        write(&mut i2c, c as u8);
    }

    loop {
    }
}

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m;
use cortex_m_rt::entry;

use stm32f3xx_hal::prelude::*;
use stm32f3xx_hal::{self, delay, pac, i2c, stm32};

const LCD_ADDRESS: u8 = 0x27;
const En: u8 = 0x04;
const Backlight: u8 = 0x08;

// A low effort and not really accurate
// delay for n milliseconds function.
// I mainly just want the delay to be proportional
// to the input. Seems to work fine.
fn delay_ms(ms: u32) {
    let mut i = ms * 7_200;
    while i > 0 {
        i -= 1;
    }
}

type Pins = (stm32f3xx_hal::gpio::gpiob::PB6<stm32f3xx_hal::gpio::AF4>, stm32f3xx_hal::gpio::gpiob::PB7<stm32f3xx_hal::gpio::AF4>);

// All the I2C implementations usually have a write4bits function
// which does the following
// 1. Write the data to the line (in this case call i2c.write(LCD_ADDRESS, &[data | 0x08]))
// 2. Pulse the Enable (En) bit
//   2.1 i2c.write(LCD_ADDRESS, &[data | En | 0x08])
//   2.2 i2c.write(LCD_ADDRESS, &[(data | ~En) | 0x08])
// To be honest, I don't really know the purpose of step 1 is
// as step 2 is also sending the data over (the most significant 4 bits)
// of 2.1 is sending the same data over. Similarly, I'm not really sure
// what the purpose of (data | ~En) is, besides sedding the En bit to 0.
// I've been able to succesfully write "Hello, World!" with the stripped
// down instructions. I've tried to find why these other writes are necessary,
// but I haven't been able to find anyting (I also haven't been able to find
// much information about the specific I2C protocol for these Hitachi LCDs;
// it seems like it's some weird tribal knowledge or I'm just missing something
// that's obvious (which is also entirely possible)).
fn write4bits(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8) {
    // En seems to say "hey, you can now read this data"
    i2c.write(LCD_ADDRESS, &[data | En | Backlight]);
    delay_ms(1);
    // Seems that all we really want to do is clear the En bit
    // and keep the light on (or off)
    i2c.write(LCD_ADDRESS, &[Backlight]);
    delay_ms(5);
}

fn send(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8, mode: u8) {
    let high_bits: u8 = data & 0xf0;
    let low_bits: u8 = (data << 4) & 0xf0;
    write4bits(i2c, high_bits | mode);
    write4bits(i2c, low_bits | mode);
}

fn write(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8) {
    send(i2c, data, 0x01);
}

fn command(i2c: &mut i2c::I2c<stm32::I2C1, Pins>, data: u8) {
    send(i2c, data, 0x00);
}

#[entry]
fn main() -> ! {
    // Three different documents that are useful
    // 1. User manual
    // 2. Reference manual
    // 3. Data sheet
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain(); // "Consumes" the FLASH peripheral
    let mut rcc = dp.RCC.constrain(); // "Consumes" the RCC peripheral

    // Flash Access Control Register
    // Initializes the clocks. Can't change clock rates after freeze.
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // dp.GPIOB is just a pointer, split breaks (consumes the peripheral) it into a struct
    // with each register as a member. Need to enable the clocks on GPIOB
    // before we can use it.
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    // My microcontroller, STM32F303DISCOVERY, has two I2C interfaces,
    // names I2C1 and I2C2. Using I2C1 here. PB6 can act as SCL for I2C1,
    // PB7 can act as SDA for I2C1, therefore we will configure these
    // pins to work with I2C1.
    // pg. 231 "The specific alternate function assignments for each pin are detailed in the device datasheet."
    // Alternate function table on pg. 47 of datasheet, says we use Alternate Function 4.
    // moder -> Mode Register, will be configured for mode Alternate Function
    // afrl -> Alternate function low register (configures alternate function mode
    //   for pins for 0-7, while afhr congigures alternate function for 8-15)
    //   alternate function 4 will be configured
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let mut i2c = i2c::I2c::i2c1(
        dp.I2C1, // Using I2C1, therefore consume the I2C1 peripheral pointer
        (scl, sda), // The pins we just configured for I2C1 alternate functions
        400.khz(), // Choosing the frequency the line runs at. 400 khz is "Fast-mode"
        // Not completely sure of the specifics, but given that I2C1 is part
        // of APB1, we need to enable the clock for I2C1 (configured through the APB1 peripheral clock enable register),
        // and we need the clock rate of ABP1 to correctly set the SCL for I2C1. I'm still
        // fuzzy on the details of what's going on.
        clocks, // Need to get the values of the Advanded Peripheral Bus 1 clock for syncronization
        &mut rcc.apb1 // Advanced Peripheral Bus 1, we need to enable the clock for I2C1
    );

    write4bits(&mut i2c, 0x03 << 4);
    delay_ms(5);
    write4bits(&mut i2c, 0x03 << 4);
    delay_ms(5);
    write4bits(&mut i2c, 0x03 << 4);
    delay_ms(5);
    write4bits(&mut i2c, 0x02 << 4); // Set to 4-bit mode, while in 8-bit mode

    command(
        &mut i2c,
        0x20 as u8 | // Function set command
        0x00 as u8 | // 5x8 display
        0x08 as u8   // Two line display
    );


    command(
        &mut i2c,
        0x08 as u8 | // Display control command
        0x04 as u8 | // Display on
        0x02 as u8 | // Cursor on
        0x01 as u8   // Blink on
    );

    command(
        &mut i2c,
        0x01 as u8 // Clear display
    );

    command(
        &mut i2c,
        0x04 as u8 | // Entry mode command
        0x02 as u8   // Entry right
    );

    let hello: &'static str = "Hello, World!";
    for c in hello.chars() {
        write(&mut i2c, c as u8);
    }

    loop {
    }
}

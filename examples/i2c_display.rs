#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig, InterruptHandler};
use embassy_rp::i2c::{Config as I2c_config, I2c};
use embassy_time::Timer;
use embassy_rp::gpio;
use gpio::{Level, Output, Pull};
use embassy_rp::bind_interrupts;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use heapless::String;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    defmt::info!("Temperature Display");

    // Configure I2C
    let sda = p.PIN_20;
    let scl = p.PIN_21;
    
    let mut i2c_config = I2c_config::default();
    i2c_config.frequency = 100_000;
    let i2c = I2c::new_blocking(p.I2C0, scl, sda, i2c_config);

    // Initialize display
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();

    if let Err(_) = display.init() {
        defmt::error!("Display init failed");
        panic!("Display init failed");
    }

    // Configure ADC
    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let mut temp_channel = Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);
    let mut adc_1 = Channel::new_pin(p.PIN_27, Pull::Down);

    //Configure GPIO
    let mut led = Output::new(p.PIN_0, Level::Low);
    // Create text styles
    let header_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let temp_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // Static header text
    let header_text = "Temp Interna";
    let header_width = header_text.len() as i32 * 6;
    let header_x = (128 - header_width) / 2;
    let header_y = 22; // Always choose in y a number between 22 to 63

    // Temperature position
    let temp_y = 30;
    let temp_x = 30;

    // Static header text ADC 1
    let header_text_adc_1 = "Value of ADC 1";
    let header_width_adc_1 = header_text_adc_1.len() as i32 * 6;
    let header_x_adc_1 = (128 - header_width_adc_1) / 2;
    let header_y_adc_1 = 42; // Always choose in y a number between 22 to 63

    // ADC 1 position
    let temp_y_adc_1 = 50;
    let temp_x_adc_1 = 50;

    loop {

        info!("led on!");
        led.set_high();
        //Timer::after_secs(1).await;

        // Read temperature
        let raw_temp = match adc.read(&mut temp_channel).await {
            Ok(v) => v,
            Err(_) => {
                defmt::error!("ADC read failed");
                continue;
            }
        };

        // Read temperature
        let raw_temp_adc_1 = match adc.read(&mut adc_1).await {
            Ok(v) => v,
            Err(_) => {
                defmt::error!("ADC read failed");
                continue;

            }
        };

        // Convert to Celsius
        let voltage = (raw_temp as f32) * 3.3 / 4095.0;
        let temp_c = 27.0 - (voltage - 0.706) / 0.001721;

        // Format temperature string
        let mut buffer = String::<10>::new();
        write!(&mut buffer, "{:.1} C", temp_c).unwrap();

        // Format temperature string
        let mut buffer_adc_1 = String::<10>::new();
        write!(&mut buffer_adc_1, "{:.1} ", raw_temp_adc_1).unwrap();

        // Update display
        if let Err(_) = display.clear(BinaryColor::Off) {
            defmt::error!("Clear failed");
        }

   

        // Draw header
        Text::new(header_text, Point::new(header_x, header_y), header_style)
            .draw(&mut display)
            .unwrap();

        // Draw temperature internal
        Text::new(&buffer, Point::new(temp_x, temp_y), temp_style)
            .draw(&mut display)
            .unwrap();

        // Draw header of ADC 1
        Text::new(header_text_adc_1, Point::new(header_x_adc_1, header_y_adc_1), header_style)
            .draw(&mut display)
            .unwrap();

        // Draw ADC 1
        Text::new(&buffer_adc_1, Point::new(temp_x_adc_1, temp_y_adc_1), temp_style)
            .draw(&mut display)
            .unwrap();

        if let Err(_) = display.flush() {
            defmt::error!("Flush failed");
        }
        

        info!("led off!");
        led.set_low();
        Timer::after_secs(1).await;

        Timer::after_secs(1).await;
    }
}
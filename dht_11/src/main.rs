#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c;
use embassy_time::{Delay, Timer};

use {defmt_rtt as _, panic_probe as _};

// For in the graphics drawing utilities like the font
// and the drawing routines:
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{Baseline, Text},
};
use u8g2_fonts::fonts::u8g2_font_wqy12_t_gb2312;
use u8g2_fonts::U8g2TextStyle;
// The display driver:
use ssd1306::{prelude::*, Ssd1306};
pub mod fmtbuf;
use fmtbuf::FmtBuf;

use core::ptr::addr_of_mut;
use embassy_dht::prelude::*;
use embedded_alloc::LlffHeap as Heap;
#[global_allocator]
static HEAP: Heap = Heap::empty();

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Pico2 Display"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example use ssd1306 to display the temp && humi  info from dht22/dht11 sensor"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());

    info!("set up i2c ");
    let sda = p.PIN_2; //i2c1 SDA
    let scl = p.PIN_3; //I2C1 SCL
    let i2c = i2c::I2c::new_blocking(p.I2C1, scl, sda, i2c::Config::default());

    // Create the I²C display interface:
    let interface = ssd1306::I2CDisplayInterface::new(i2c);

    // Create a driver instance and initialize:
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    // Create a text style for drawing the font:
    let character_style = U8g2TextStyle::new(u8g2_font_wqy12_t_gb2312, BinaryColor::On);

    info!("set up dhtxx pin");
    let mut line0_p2 = FmtBuf::new();

    cfg_if::cfg_if! {
        if  #[cfg(feature = "dht11")] {
            write!(&mut line0_p2, "{}", "DHT11").unwrap();
            let mut dht_pin = DHT11::new(p.PIN_17, Delay);

        } else if #[cfg(feature = "dht22")] {
            write!(&mut line0_p2, "{}", "DHT22").unwrap();
            let mut dht_pin = DHT22::new(p.PIN_17, Delay);
        }
    }


    let mut led = Output::new(p.PIN_25, Level::Low);

    // Perform a sensor reading
    let mut line1 = FmtBuf::new();
    let mut line2 = FmtBuf::new();

    loop {
        Timer::after_secs(1).await;
        // Empty the display:
        // Draw 3 lines of text:
        //reset before loop
        let _ = display.clear(BinaryColor::Off);
        line1.reset();
        line2.reset();

        let dht_reading = dht_pin.read().unwrap();

        // Perform a sensor reading
        //  let measurement = Reading::read(&mut Delay, &mut dht_pin).unwrap();
        //  let (temp, humi) = get(measurement).value();

        let (temp, humi) = (dht_reading.get_temp(), dht_reading.get_hum());
        Text::with_baseline(
            "SensorType",
            Point::new(3, 2),
            character_style.clone(),
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        Text::with_baseline(
            line0_p2.as_str(),
            Point::new(74, 2),
            character_style.clone(),
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        write!(&mut line1, "温度： {}℃", temp).unwrap(); // ℃ ,°C
        Text::with_baseline(
            line1.as_str(),
            Point::new(32, 22),
            character_style.clone(),
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        write!(&mut line2, "湿度： {}%", humi).unwrap();
        Text::with_baseline(
            line2.as_str(),
            Point::new(32, 38),
            character_style.clone(),
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        Line::new(Point::new(0, 0), Point::new(127, 0))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        Line::new(Point::new(0, 0), Point::new(0, 63))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        Line::new(Point::new(0, 63), Point::new(127, 63))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        Line::new(Point::new(127, 0), Point::new(127, 63))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        Line::new(Point::new(70, 0), Point::new(70, 16))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        Line::new(Point::new(0, 16), Point::new(127, 16))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        Line::new(Point::new(0, 15), Point::new(127, 15))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();

        //Timer::after_secs(1).await;

        info!("led on!");
        led.set_high();
        Timer::after_millis(250).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(250).await;
    }
}
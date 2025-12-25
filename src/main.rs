use pio_pi5_rs::*;
use std::{
    error::Error,
    thread::{self},
    time::Duration,
};

const PIXEL_COUNT: usize = 8;
const FRAME_DELAY_MS: u64 = 35;
const COLOR_SPREAD: u16 = 24;
const COLOR_SPEED: u16 = 3;
const SPARKLE_SPACING: u16 = 56;
const BREATH_PERIOD: u16 = 640;
const SPARKLE_PERIOD: u16 = 256;
const BASE_BRIGHTNESS: u16 = 45;

fn main() -> Result<(), Box<dyn Error>> {
    let pio = Rp1PIO::new(0)?;
    let state_machine = pio.sm_claim_unused()?;
    let instructions = [0x6221, 0x1223, 0x1300, 0xa342];
    let offset = pio.add_program(&PioProgram::new(&instructions, None))?;

    let gpio = 4;

    println!("ws2812 raspberry pi tesing, running on gpio {}", gpio);

    let rgbw = true;
    //init
    //pio.gpio_init(gpio)?;
    pio.pio_gpio_init(gpio)?;
    state_machine.set_consecutive_pindirs(gpio as u32, 1, true)?;
    let div = 200_000_000 as f64 / (800_000 * (3 + 4 + 3)) as f64;
    let sm_config = SmConfig::default()
        .set_wrap(offset as u32 + 0, offset as u32 + 3)?
        .set_sideset(1, false, false)?
        .set_sideset_pins(gpio as u32)?
        .set_out_shift(false, true, if rgbw { 32 } else { 24 })?
        .set_fifo_join(PioFifoJoin::Tx)?
        .set_clkdiv(div)?;
    state_machine.init(offset, &sm_config)?;
    state_machine.set_enabled(true)?;

    println!("init success");

    // Build a smooth rainbow chase with a gentle breathing effect by advancing a shared frame counter.
    let mut frame: u16 = 0;
    loop {
        let breathe = triangle_wave(frame, BREATH_PERIOD);

        for led in 0..PIXEL_COUNT {
            let color_index = frame.wrapping_add((led as u16) * COLOR_SPREAD) as u8;
            let sparkle_phase = frame.wrapping_add((led as u16) * SPARKLE_SPACING);
            let sparkle = triangle_wave(sparkle_phase, SPARKLE_PERIOD);

            let mut brightness = BASE_BRIGHTNESS + mul_u8(breathe, sparkle) as u16;
            if brightness > 255 {
                brightness = 255;
            }

            let (r, g, b) = apply_brightness(color_wheel(color_index), brightness as u8);
            let packed = pack_grb(r, g, b);
            state_machine.put(packed, true)?;
        }

        frame = frame.wrapping_add(COLOR_SPEED);
        thread::sleep(Duration::from_millis(FRAME_DELAY_MS));
    }
}

fn pack_grb(r: u8, g: u8, b: u8) -> u32 {
    (((g as u32) << 16) | ((r as u32) << 8) | b as u32) << 8
}

fn apply_brightness((r, g, b): (u8, u8, u8), brightness: u8) -> (u8, u8, u8) {
    let scale = |channel: u8| -> u8 { ((channel as u16 * brightness as u16) / 255) as u8 };
    (scale(r), scale(g), scale(b))
}

fn triangle_wave(tick: u16, period: u16) -> u8 {
    let half = period / 2;
    if half == 0 {
        return 0;
    }

    let phase = tick % period;
    let rising = if phase < half { phase } else { period - phase };
    ((rising as u32 * 255) / half as u32) as u8
}

fn mul_u8(a: u8, b: u8) -> u8 {
    ((a as u16 * b as u16) / 255) as u8
}

fn color_wheel(pos: u8) -> (u8, u8, u8) {
    let pos = 255 - pos;
    match pos {
        0..=84 => {
            let p = pos as u16;
            (((255 - p * 3) as u8), 0, (p * 3) as u8)
        }
        85..=169 => {
            let p = (pos - 85) as u16;
            (0, (p * 3) as u8, (255 - p * 3) as u8)
        }
        _ => {
            let p = (pos - 170) as u16;
            ((p * 3) as u8, (255 - p * 3) as u8, 0)
        }
    }
}
use std::{error::Error, process::Stdio, thread::sleep_ms};
use pio_pi5_rs::*;


fn main() -> Result<(), Box<dyn Error>> {
    let pio = Rp1PIO::new(0)?;
    let state_machine = pio.sm_claim_unused()?;
    let instructions = [0x6221, 0x1223, 0x1300, 0xa342];
    let offset = pio.add_program(&PioProgram::new(&instructions, None))?;

    let gpio = 4;

    println!("ws2812 raspberry pi tesing, running on gpio {}", gpio);

    let rgbw = true;
    //init
    pio.gpio_init(gpio)?;
    state_machine.set_consecutive_pindirs(gpio as u32, 1, true)?;
    let div = 500_000_000 as f64 / (800_000 * (3 + 4 + 3)) as f64;
    let sm_config = SmConfig::default()
        .set_sideset_pins(gpio as u32)?
        .set_out_shift(false, true, if rgbw {32} else {24})?
        .set_fifo_join(PioFifoJoin::Tx)?
        .set_clkdiv(div)?;
    state_machine.init(offset, &sm_config)?;
    state_machine.set_enabled(true)?;

    loop {
        state_machine.put(rand::random(), true)?;
        sleep_ms(10);
    }
    
    Ok(())
}
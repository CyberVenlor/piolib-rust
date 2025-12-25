use std::{error::Error, process::Stdio};
use pio_pi5_rs::*;


fn main() -> Result<(), Box<dyn Error>> {
    let pio = Rp1PIO::new(0)?;
    let state_machine = pio.sm_claim_unused()?;
    let instructions = [0x6221, 0x1223, 0x1300, 0xa342];
    let offset = pio.add_program(&PioProgram::new(&instructions, None));

    Ok(())
}
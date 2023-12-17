#![no_std]
use ili9325_lcd;

fn write_command(value: u16) {
}

fn write_data(value: u16) {
}

fn read_data() -> u16 {
    0x1234
}

fn reset() {
}

#[derive(Copy, Clone)]
pub struct Interface {

}

impl Interface {
	pub fn new() -> Interface {
		Interface {
		}
	}
}

impl ili9325_lcd::Interface for Interface {
    fn write_command(&self, command: u16) {
		write_command(command);
	}

	fn write_data(&self, data: u16) {
	    write_data(data);
	}

    fn read_data(&self, data: &mut u16) {
        *data = read_data();
	}

	fn reset(&self) {
	    reset();
	}
}

pub type Controller = ili9325_lcd::Controller<Interface>;

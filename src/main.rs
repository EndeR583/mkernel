#![no_std]
#![no_main]

use lazy_static::lazy_static;
use spin::Mutex;
use core::panic::PanicInfo;
use cpuio::outb;
use core::fmt::Write;
use crate::vga_buffer::{Color, ColorCode, Writer};
use core::fmt;

mod vga_buffer;

//Function for panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	loop {}
}

/*This is the global WRITER that we use for printing.
Accessible through Mutex to make it secure.
Use it with WRITER.lock().<function_name(<parameters>)>*/
lazy_static!{
	pub static ref WRITER: Mutex<vga_buffer::Writer> = Mutex::new( Writer {
    		column_position: 0,
    		color_code: ColorCode::new(Color::LightGray, Color::Black),
    		buffer: unsafe { &mut *(0xb8000 as *mut vga_buffer::Buffer) },
    		row_position: 0,
		});
}

/*Macro for _print. Accepts arguments for core::fmt::Writer.
For more info, check out the _print function.*/
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

/*Macro for _println. Accepts arguments for core::fmt::Writer.
After printing the given data, it moves the cursor to the next line.
For more info, check out the _print function.*/
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/*Function for printing to the WRITER.
Breaks line after each 80th character.
Don't call it directly, please use the macro print! or println! instead.*/
#[doc(hidden)]
fn _print(args: fmt::Arguments) {
	WRITER.lock().write_fmt(args).unwrap();
}

/*Macro for _set_color_code. Accepts 2 Color enum parameters.
The first parameter sets the foreground, and the second sets the background color.
For more info, check out the _set_color_code function.*/
#[macro_export]
macro_rules! set_color_code {
	($c1:path, $c2:path) => ($crate::_set_color_code($c1, $c2));
}

/*Function for changing the foreground and background color of the WRITER
Accepts a ColorCode parameter which is a byte containing the 2 colors (4 bit for each).
It can be casted using ColorCode::new(<foregroundColor>, <backgroundColor>).
Only text printed after calling this function will have the given colors.
Don't call it directly, please use the macro set_color_code! instead.*/
#[doc(hidden)]
fn _set_color_code(c1: Color, c2:Color){
	WRITER.lock().set_color_code(ColorCode::new(c1, c2));
}

/*Macro for setting the cursor position.
It accepts an x: u16 and an y: u16 parameter.
0 <= x <= 80
0 <= y <= 25
For more info, check out the _set_cursor_position function.*/
#[macro_export]
macro_rules! set_cursor_position {
	($x:expr, $y:expr) => ($crate::_set_cursor_position($x, $y))
}

/*Function for changing the cursor position.
It is using direct interrupts to the CPU (UNSAFE).
Don't call it directly, please use the macro set_cursor_position! instead.*/
#[doc(hidden)]
fn _set_cursor_position(x: u16, y: u16){
	if x <= 80 && y <= 25{
		let pos: u16 = &y * 80 + &x;
		unsafe{
			outb(0x0F, 0x3D4);
			outb((pos & 0xFF) as u8, 0x3D5);
			outb(0x0E, 0x3D4);
			outb(((pos >> 8) & 0xFF) as u8, 0x3D5);
		}
	}
}

/*The main entry point of the kernel.
It prints out a welcome message along with a lipsum for macro/color testing,
then moves the cursor to the beginning of the last line (that will be the input area,
as this kernel is going to be a shell in the first phase).
TODO: make a functional shell*/
#[no_mangle]
pub extern "C" fn _start() -> ! {
	println!("Welcome to mCore!");
	set_color_code!(Color::LightRed, Color::LightGray);
	println!("Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.");
	set_color_code!(Color::LightGray, Color::Black);
    set_cursor_position!(0, 24);
	//TODO macro for write_string_in_row function.
	WRITER.lock().write_string_in_row("By Mattee, 2019", 23, true);
 	loop {}
}

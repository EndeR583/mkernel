use volatile::Volatile;
use core::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
//Enum for colors.
pub enum Color {
	Black = 0,
	Blue = 1,
	Green = 2,
	Cyan = 3,
	Red = 4,
	Magenta = 5,
	Brown = 6,
	LightGray = 7,
	DarkGray = 8,
	LightBlue = 9,
	LightGreen = 10,
	LightCyan = 11,
	LightRed = 12,
	Pink = 13,
	Yellow = 14,
	White = 15, 
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
/*A byte containing the values of 2 Color enums.
The first 4 bits are for the background, and the other 4 bits are for the foreground color.
It is used by the WRITER to print colored text.*/
pub struct ColorCode(u8);

impl ColorCode {
	/*Makes the byte value by taking the 2 color values and pushing
	the bits of the background color to the left by 4 bits,
	and filling the other 4 bits with the bits of the foreground color.*/
	pub fn new(foreground: Color, background: Color) -> ColorCode{
		ColorCode((background as u8) << 4 | (foreground as u8))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
//A struct used storing a byte-sized ASCII character, and it's ColorCode.
struct ScreenChar {
	ascii_character: u8,
	color_code: ColorCode,
}

//The maximum number of lines.
const BUFFER_HEIGHT: usize = 25;
//The maximum number of characters in one line.
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
/*A 2 dimensional array for storing the characters to display.
In VGA Text mode, writing to this array at the location 0xb8000
will make the given characters appear on the screen.*/
pub struct Buffer {
	chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/*The struct for the WRITER object.
It stores the current cursor position (usize), ColorCode and the buffer
(which is pointing to 0xb8000, so we can print out text).*/
pub struct Writer {
	pub column_position: usize,
	pub row_position: usize,
	pub color_code: ColorCode,
	pub buffer: &'static mut Buffer,
}

impl Writer {
	//Function for changing ColorCode.
	pub fn set_color_code(&mut self, c: ColorCode){
		self.color_code = c;
	}
	pub fn write_string_in_row(&mut self, s: &str, r: usize, clear: bool){
		if r <= BUFFER_HEIGHT {
			if clear {
				self.clear_row(r);
			}
			self.column_position = 0;
			self.row_position = r;
			self.write_string(&s);
		}
	}
	pub fn write_byte(&mut self, byte: u8) {
		match byte{
			b'\n' => self.new_line(),
			byte => {
				if self.column_position >= BUFFER_WIDTH {
					self.new_line();
				}

				let row: usize = self.row_position;
				let col = self.column_position;

				let color_code = self.color_code;
				self.buffer.chars[row][col].write(ScreenChar {
					ascii_character: byte,
					color_code,
				});
				self.column_position += 1;
			}
		}
	}
	fn new_line(&mut self){
		if self.row_position >= BUFFER_HEIGHT - 2{
			for row in 1..BUFFER_HEIGHT {
				for col in 0..BUFFER_WIDTH {
					let character = self.buffer.chars[row][col].read();
					self.buffer.chars[row - 1][col].write(character);
				}
			}
			self.row_position = BUFFER_HEIGHT - 2;
		} else {
			self.row_position += 1;
		}
        self.column_position = 0;
    }
	fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }

        }
    }
}

impl fmt::Write for Writer {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.write_string(s);
		Ok(())
	}
}
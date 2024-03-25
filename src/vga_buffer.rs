use core::fmt;
use core::fmt::{Result, Write};
use volatile::Volatile;
use lazy_static::lazy_static; //惰性初始化静态数据，其中值仅在第一次线程安全访问时初始化
use spin::Mutex; //使用自旋锁，不使用标准库提供的互斥锁类 Mutex

/*

//标准库中 println! 宏的实现源码
#[macro_export] 
macro_rules! println {
    () => (print!("\n")); // println!() 换行
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*))); // println!("任意") 输出任意值
}
//print! 宏的实现源码
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}
*/

#[macro_export] //让整个包（crate）和基于它的包都能访问这个宏，而不仅限于定义它的模块（module）
macro_rules! print { //macro_rules! 声明宏的方式
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {  
    () => ($crate::print!("\n")); //修改println!宏，在其前添加 $crate，使其使用 println! 时，不必也编写代码导入 print! 宏
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    //use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)] //使用 #[allow(dead_code)]，可以禁用编译器对每个未使用的变量发出警告
#[derive(Debug, Clone, Copy, PartialEq, Eq)] //生成（derive了Copy、Clone、Debug、PartialEq 和 Eq 这几个trait
                                             //Trait是Rust中的一种抽象机制,类似于其他编程语言中的接口或抽象类
#[repr(u8)] //限制枚举类型为u8
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
struct ColorCode(u8);

impl ColorCode { //ColorCode 类型包装了一个完整的颜色代码字节，包含前景色(字体颜色)和背景色信息(字体外的填充颜色)
    //impl 用以调用类型( struct )或特性( trait )
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar{ //成员变量
    ascii_character: u8,
    color_code: ColorCode
}

const BUFFER_HEIGHT: usize = 25; //定义整块区域的行数为25
const BUFFER_WIDTH: usize = 80;  //定义整块区域的列数为80

#[repr(transparent)] //用以确保类型和它的单个成员有相同的内存布局
struct Buffer{
    //volatile是一个泛型，其包含所有类型，其主要作用为在多线程环境下对变量的可见性
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT], //相当于先输出一行数据，再输出下一行((a,b),c)
}

pub struct Writer { //输出字符到屏幕
    column_position: usize, //此变量将跟踪光标在最后一行的位置
    color_code: ColorCode, //字符的前景和背景色
    buffer: &'static mut Buffer, //存入一个 VGA 字符缓冲区的可变借用( &mut )到buffer变量中, 'static为生命周期，意味着这个借用应该在整个程序的运行期间有效
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new( //使用自旋的互斥锁，为 WRITER 类实现安全的内部可变性
        Writer { 
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer: unsafe { 
                &mut *(0xb8000 as *mut Buffer) 
            },
        }
    );
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) { //输出ascii字符
        match byte {
            b'\n' => self.new_line(), //输入字符 '\n' ，调用new_line()方法
            byte => {
                if self.column_position >= BUFFER_WIDTH { //当前行的列数等于最大值时，创建新的一行
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1; //当前列数+1
            }
        }
    }
    fn write_string(&mut self, s: &str) { //输出字符串
        for byte in s.bytes() {
            match byte {
                // 可以是能打印的 ASCII 码字节，也可以是换行符
                0x20..=0x7e | b'\n' => self.write_byte(byte), // ' a ..= b ' 相当于 从 a 到 b 的值
                // 不包含在上述范围之内的字节
                _ => self.write_byte(0xfe), //打印的 '_' 在ga编码中为16进制的 (0xfe)
            }


        }
    }
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
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
    
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        self.write_string(s);
        Ok(())
    }
}

/*#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0 .. 200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i,c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    } 
}*/
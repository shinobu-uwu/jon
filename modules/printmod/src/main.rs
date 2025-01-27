#![no_std]
#![no_main]

use jon_common::println;

#[no_mangle]
fn _start() -> ! {
    loop {
        println!("Hello, world from module!");
    }
}

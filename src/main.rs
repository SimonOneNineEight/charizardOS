#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(charizard::test_runner)]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use charizard::{memory::BootInfoFrameAllocator, println, serial_println};
use core::panic::PanicInfo;

// Tells Rust what the entry function of the OS
entry_point!(kernel_main);

#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use charizard::{
        allocator,
        devices::{console::CONSOLE, keyboard},
        memory,
    };
    use x86_64::{instructions, VirtAddr};

    println!("Welcome to Charizard!");

    charizard::init();

    let phys_mem_offset: VirtAddr = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    println!("Kernel initialized! Waiting for commands...");

    #[cfg(test)]
    test_main();

    // let mut console = CONSOLE.lock();
    //
    // console.print_char_and_move_cursor('h');
    // console.print_char_and_move_cursor('e');
    // console.print_char_and_move_cursor('l');
    // console.print_char_and_move_cursor('l');
    // console.print_char_and_move_cursor('o');

    loop {
        let command = keyboard::read_line();
        println!("Command received: {}", command);
        instructions::hlt();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    charizard::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    charizard::test_panic_handler(info);
}

#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(charizard::test_runner)]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use charizard::{memory::BootInfoFrameAllocator, println};
use core::panic::PanicInfo;

// Tells Rust what the entry function of the OS
entry_point!(kernel_main);

#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use charizard::{allocator, command, devices::keyboard, file_system::FileSystem, memory};
    use x86_64::{instructions, VirtAddr};

    println!("Welcome to Charizard!");

    charizard::init();

    let phys_mem_offset: VirtAddr = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let mut fs = FileSystem::new();

    println!("Kernel initialized! Waiting for commands...");

    #[cfg(test)]
    test_main();

    loop {
        let command = keyboard::read_line();

        match command::parse_and_execute_command(&command, &mut fs) {
            Ok(response) => println!("{}", response),
            Err(err) => println!("Error: {}", err),
        }
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

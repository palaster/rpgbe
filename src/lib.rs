#![allow(dead_code, unused_variables, unused_macros)]
#![feature(lang_items, core_intrinsics, start, allocator_api)]
#![no_std]

#[macro_use]
extern crate alloc;

use core::intrinsics;
use core::panic::PanicInfo;

use psp2_sys::ctrl::*;
use psp2_sys::kernel::processmgr::*;
use vitallocator::Vitallocator;

mod debug;
mod gameboy;
mod bit_logic;

#[global_allocator]
static GLOBAL: Vitallocator = Vitallocator::new();

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    intrinsics::abort()
}

#[no_mangle]
pub unsafe fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut gameboy = gameboy::Gameboy::new(debug::screen::DebugScreen::new());
    
    let data = include_bytes!("../../tetris.gb");

    gameboy.memory.load_cartridge(data.to_vec());
    
    if sceCtrlSetSamplingMode(SceCtrlPadInputMode::SCE_CTRL_MODE_ANALOG) < 0 {
        sceKernelExitProcess(0);
        return 0;
    }

    // id, button, current_state, was_released
    let mut button_array = [
        (4, SceCtrlButtons::SCE_CTRL_CIRCLE, &mut false, &mut false),
        (5, SceCtrlButtons::SCE_CTRL_CROSS, &mut false, &mut false),
        (2, SceCtrlButtons::SCE_CTRL_UP, &mut false, &mut false),
        (3, SceCtrlButtons::SCE_CTRL_DOWN, &mut false, &mut false),
        (1, SceCtrlButtons::SCE_CTRL_LEFT, &mut false, &mut false),
        (0, SceCtrlButtons::SCE_CTRL_RIGHT, &mut false, &mut false),
        (7, SceCtrlButtons::SCE_CTRL_START, &mut false, &mut false),
        (6, SceCtrlButtons::SCE_CTRL_SELECT, &mut false, &mut false)
    ];

    loop {
    
        let mut ctrl: SceCtrlData = Default::default();
        if sceCtrlReadBufferPositive(0, &mut ctrl, 1) < 0 {
            break;
        }

        if ctrl.buttons == (SceCtrlButtons::SCE_CTRL_LTRIGGER as u32 | SceCtrlButtons::SCE_CTRL_RTRIGGER as u32) {
            break;
        }

        for button in &mut button_array {
            if ctrl.buttons == button.1 as u32 {
                *button.2 = true;
            } else {
                if *button.2 {
                    *button.3 = true;
                }
                *button.2 = false;
            }
        }

        for button in &mut button_array {
            if *button.2 {
                //gameboy.key_pressed(button.0);
            }
            if !*button.2 && *button.3 {
                *button.3 = false;
                //gameboy.key_released(button.0);
            }
        }

        //let start = Instant::now();
        gameboy.next_frame();
        /*
        texture.update(None, &gameboy.screen_data, WIDTH.wrapping_mul(3) as usize).expect("Couldn't update texture from main");
        canvas.clear();
        canvas.copy(&texture, None, None).expect("Couldn't copy canvas");
        canvas.present();

        device.queue(&gameboy.spu.audio_data);
        gameboy.spu.audio_data.clear();

        let elapsed_time = start.elapsed();
        if elapsed_time <= DURATION_BETWEEN_FRAMES {
            let time_remaining = DURATION_BETWEEN_FRAMES - elapsed_time;
            thread::sleep(time_remaining);
        }
         */
    }

    sceKernelExitProcess(0);
    return 0;
}

#[start]
#[no_mangle]
pub unsafe fn _start(argc: isize, argv: *const *const u8) -> isize {
    main(argc, argv)
}
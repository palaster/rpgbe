#![allow(dead_code, unused_variables, unused_macros)]
#![feature(lang_items, core_intrinsics, start, allocator_api)]
#![no_std]

#[macro_use]
extern crate alloc;

use core::intrinsics;
use core::mem::size_of;
use core::panic::PanicInfo;

use psp2_sys::ctrl::*;
use psp2_sys::display::*;
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
    let debug_screen = debug::screen::DebugScreen::new();
    let mut gameboy = gameboy::Gameboy::new();
    
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

    const FORMATED_SCREEN_DATA_SIZE: usize = (gameboy::WIDTH * gameboy::HEIGHT) as usize;
    static mut FORMATED_FRAME_DATA: [u32; FORMATED_SCREEN_DATA_SIZE] = [0; FORMATED_SCREEN_DATA_SIZE];

    const VITA_WIDTH: u32 = 960;
    const VITA_HEIGHT: u32 = 544;
    const SCALED_SCREEN_DATA_SIZE: usize = (VITA_WIDTH * VITA_HEIGHT) as usize;
    static mut SCALED_FRAME_DATA: [u32; SCALED_SCREEN_DATA_SIZE] = [0; SCALED_SCREEN_DATA_SIZE];

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
            if *button.2 {
                gameboy.key_pressed(button.0);
            }
            if !*button.2 && *button.3 {
                *button.3 = false;
                gameboy.key_released(button.0);
            }
        }

        //let start = Instant::now();
        gameboy.next_frame();

        for i in (0..(FORMATED_SCREEN_DATA_SIZE * 3)).step_by(3) {
            let (r, g, b) = (gameboy.screen_data[i], gameboy.screen_data[i + 1], gameboy.screen_data[i + 2]);
            let value = ((255u8 as u32) << 24)
                + ((b as u32) << 16)
                + ((g as u32) << 8)
                + ((r as u32) << 0);
            FORMATED_FRAME_DATA[i / 3] = value;
        }

        for y in 0..gameboy::HEIGHT {
            for x in 0..gameboy::WIDTH {
                SCALED_FRAME_DATA[(x as u32 + (VITA_WIDTH * y as u32)) as usize] = FORMATED_FRAME_DATA[(x + (gameboy::WIDTH * y)) as usize];
            }
        }

        let sce_display_frame_buf = SceDisplayFrameBuf {
            size: size_of::<SceDisplayFrameBuf>() as u32,
            base: SCALED_FRAME_DATA.as_mut_ptr() as *mut core::ffi::c_void,
            pitch: VITA_WIDTH,
            pixelformat: SceDisplayPixelFormat::SCE_DISPLAY_PIXELFORMAT_A8B8G8R8 as u32,
            width: VITA_WIDTH,
            height: VITA_HEIGHT,
        };
        sceDisplaySetFrameBuf(&sce_display_frame_buf, SceDisplaySetBufSync::SCE_DISPLAY_SETBUF_NEXTFRAME);
        
        gameboy.spu.audio_data.clear();

        /*
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
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
    //let mut gameboy = gameboy::Gameboy::new();
    //gameboy.memory.load_cartridge(Vec::new());
    
    if sceCtrlSetSamplingMode(SceCtrlPadInputMode::SCE_CTRL_MODE_ANALOG) < 0 {
        sceKernelExitProcess(0);
        return 0;
    }

    loop {
    
        let ctrl: *mut SceCtrlData = core::ptr::null_mut::<SceCtrlData>();
        let ret = sceCtrlReadBufferPositive(0, ctrl, 1);

        if ret >= 0 {
            break;
        }

        if !ctrl.is_null() {
            break;
        }

        /*
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(key_down), repeat: false, .. } => {
                    let key_code: i8 = match key_down {
                        Keycode::W => 2, // UP
                        Keycode::A => 1, // LEFT
                        Keycode::S => 3, // DOWN
                        Keycode::D => 0, // RIGHT
                        Keycode::H => 5, // B
                        Keycode::U => 4, // A
                        Keycode::B => 6, // SELECT
                        Keycode::N => 7, // START
                        _ => -1,
                    };
                    if key_code >= 0 {
                        gameboy.key_pressed(key_code as u8);
                    }
                },
                Event::KeyUp { keycode: Some(key_up), repeat: false, .. } => {
                    let key_code: i8 = match key_up {
                        Keycode::W => 2, // UP
                        Keycode::A => 1, // LEFT
                        Keycode::S => 3, // DOWN
                        Keycode::D => 0, // RIGHT
                        Keycode::H => 5, // B
                        Keycode::U => 4, // A
                        Keycode::B => 6, // SELECT
                        Keycode::N => 7, // START
                        _ => -1,
                    };
                    if key_code >= 0 {
                        gameboy.key_released(key_code as u8);
                    }
                },
                _ => (),
            }
        }
        */

        //let start = Instant::now();
        //gameboy.next_frame();
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
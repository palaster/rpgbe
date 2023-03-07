#![allow(dead_code, unused_variables, unused_macros)]
#![feature(lang_items, core_intrinsics, start, allocator_api)]
#![no_std]

#[macro_use]
extern crate alloc;

use core::intrinsics;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::slice::from_raw_parts_mut;

use psp2_sys::ctrl::*;
use psp2_sys::display::*;
use psp2_sys::kernel::processmgr::*;
use psp2_sys::kernel::sysmem::SceKernelAllocMemBlockOpt;
use psp2_sys::kernel::sysmem::SceKernelMemBlockType;
use psp2_sys::kernel::sysmem::SceKernelMemoryAccessType;
use psp2_sys::kernel::sysmem::sceKernelAllocMemBlock;
use psp2_sys::kernel::sysmem::sceKernelGetMemBlockBase;
use psp2_sys::kernel::threadmgr::sceKernelCreateMutex;
use psp2_sys::kernel::threadmgr::sceKernelLockMutex;
use psp2_sys::kernel::threadmgr::sceKernelUnlockMutex;
use psp2_sys::types::SceUID;
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

    const VITA_WIDTH: u32 = 960;
    const VITA_HEIGHT: u32 = 544;

    let mutex: SceUID = sceKernelCreateMutex(b"display_mutex\0".as_ptr(), 0, 0, core::ptr::null_mut());

    let mut display_block_options: SceKernelAllocMemBlockOpt = SceKernelAllocMemBlockOpt {
        size: size_of::<SceKernelAllocMemBlockOpt>() as u32,
        attr: SceKernelMemoryAccessType::SCE_KERNEL_MEMORY_ACCESS_R as u32,
        alignment: 256 * 1024,
        uidBaseBlock: 0,
        strBaseBlockName: core::ptr::null(),
        flags: 0,
        reserved: [0; 10],
    };

    let display_block = sceKernelAllocMemBlock(b"display\0".as_ptr(), SceKernelMemBlockType::SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
        2 * 1024 * 1024, &mut display_block_options);

    let mut framebuffer_pointer: *mut core::ffi::c_void = core::ptr::null_mut();
    sceKernelGetMemBlockBase(display_block, &mut framebuffer_pointer);

    let sce_display_frame_buf = SceDisplayFrameBuf {
        size: size_of::<SceDisplayFrameBuf>() as u32,
        base: framebuffer_pointer,
        pitch: VITA_WIDTH,
        pixelformat: SceDisplayPixelFormat::SCE_DISPLAY_PIXELFORMAT_A8B8G8R8 as u32,
        width: VITA_WIDTH,
        height: VITA_HEIGHT,
    };
    sceDisplaySetFrameBuf(&sce_display_frame_buf, SceDisplaySetBufSync::SCE_DISPLAY_SETBUF_NEXTFRAME);

    let vram: &mut [u32] = from_raw_parts_mut(framebuffer_pointer as *mut u32, (2 * 1024 * 1024) / 4);

    let mut ctrl: SceCtrlData = Default::default();

    loop {

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

        sceKernelLockMutex(mutex, 1, core::ptr::null_mut());
        for y in 0..gameboy::HEIGHT {
            for x in 0..gameboy::WIDTH {
                vram[(x as u32 + (VITA_WIDTH * y as u32)) as usize] = gameboy.screen_data[(x + (gameboy::WIDTH * y)) as usize];
            }
        }
        sceKernelUnlockMutex(mutex, 1);
        
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
#![allow(dead_code, unused_imports, unused_variables, unused_macros, unused_parens)]
#![feature(lang_items, core_intrinsics, start, allocator_api)]
#![no_std]

#[macro_use]
extern crate alloc;

use core::alloc::Allocator;
use core::alloc::GlobalAlloc;
use core::fmt::Write;
use core::intrinsics;
use core::panic::PanicInfo;
use core::alloc::AllocError;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::cmp::max;
use core::mem::MaybeUninit;
use core::mem::size_of;
use core::ptr::NonNull;
use core::ptr::null;
use core::ptr::null_mut;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

use alloc::vec::Vec;
use psp2_sys::ctrl::*;
use psp2_sys::kernel::sysmem::*;
use psp2_sys::kernel::sysmem::SceKernelMemBlockType::SCE_KERNEL_MEMBLOCK_TYPE_USER_RW;
use psp2_sys::kernel::sysmem::SceKernelMemoryAccessType::SCE_KERNEL_MEMORY_ACCESS_R;
use psp2_sys::kernel::threadmgr::*;
use psp2_sys::kernel::processmgr::*;
use psp2_sys::types::SceUID;

mod gameboy;
mod bit_logic;

fn write_hex(number: usize, buf: &mut [u8]) {
    let length = ::core::mem::size_of::<usize>() / 4;
    for i in 0..length {
        buf[buf.len() - (i + 2)] = match (number & 0xF) as u8 {
            x @ 0x0u8..=0x9u8 => x as u8 + b'0',
            y @ 0xAu8..=0xFu8 => y as u8 + b'A',
            _ => unreachable!(),
        };
    }
}

pub struct Vitallocator {
    block_count: AtomicUsize,
}

impl Default for Vitallocator {
    fn default() -> Self {
        Vitallocator::new()
    }
}

impl Vitallocator {
    /// Create a new kernel allocator.
    pub const fn new() -> Self {
        Vitallocator {
            block_count: AtomicUsize::new(0)
        }
    }
}

unsafe impl GlobalAlloc for Vitallocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Prepare the options to pass to SceKernelAllocMemBlock
        let mut options = SceKernelAllocMemBlockOpt {
            size: size_of::<SceKernelAllocMemBlockOpt>() as u32,
            attr: SCE_KERNEL_MEMORY_ACCESS_R as u32,
            alignment: layout.align() as u32,
            uidBaseBlock: 0,
            strBaseBlockName: ::core::ptr::null(),
            flags: 0,
            reserved: [0; 10],
        };

        // Prepare the pointer
        let mut basep: *mut core::ffi::c_void = ::core::ptr::null_mut::<u8>() as *mut _;

        // Define a new name for the block (writing the block count as hex)
        let mut name: [u8; 18] = *b"__rust_0x00000000\0";
        write_hex(self.block_count.load(Ordering::SeqCst), &mut name[9..16]);

        // Allocate the memory block
        let uid: SceUID = sceKernelAllocMemBlock(
            (&name).as_ptr(),
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW,
            max(layout.size() as i32, 4096),
            &mut options as *mut _,
        );
        if uid < 0 {
            return null_mut();
        }

        // Imcrease the block count: to the kernel, we allocated a new block.
        // `wrapping_add` avoids a panic when the total number of allocated blocks
        // exceeds `usize::max_value()`. An undefined behaviour is still expected
        // from the kernel since some block could possibly be named the same.
        self.block_count.fetch_add(1, Ordering::SeqCst);

        // Get the adress of the allocated location
        if sceKernelGetMemBlockBase(uid, &mut basep as *mut *mut core::ffi::c_void) < 0 {
            sceKernelFreeMemBlock(uid); // avoid memory leak if the block cannot be used
            return null_mut();
        }

        // Return the obtained non-null, opaque pointer
        basep as *mut _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Get the size of the pointer memory block
        let info: *mut SceKernelMemBlockInfo = ::core::ptr::null_mut::<SceKernelMemBlockInfo>();
        sceKernelGetMemBlockInfoByAddr(ptr as *mut core::ffi::c_void, info);

        // Find the SceUID
        let uid = sceKernelFindMemBlockByAddr(ptr as *mut core::ffi::c_void, (*info).size);

        // Free the memory block
        sceKernelFreeMemBlock(uid);
    }
}

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
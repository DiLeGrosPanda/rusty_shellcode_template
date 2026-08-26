#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unsafe_op_in_unsafe_fn)]

use core::arch::global_asm;
use core::ffi::c_void;
use core::panic::PanicInfo;
use core::ptr;

mod intrinsics;
mod utils;

use crate::utils::obf_str;

type FnGetStdHandle = unsafe extern "system" fn(u32) -> *mut c_void;
type FnWriteFile =
    unsafe extern "system" fn(*mut c_void, *const u8, u32, *mut u32, *mut c_void) -> i32;
type FnLoadLibraryA = unsafe extern "system" fn(*const u8) -> *mut c_void;
type FnMessageBoxA = unsafe extern "system" fn(*mut c_void, *const u8, *const u8, u32) -> i32;

const KERNEL32_HASH: u32 = jenkins_hash_str(b"KERNEL32.DLL");
const MESSAGEBOXAHASH: u32 = jenkins_hash_str(b"MessageBoxA");
const GETSTDHANDLE_HASH: u32 = jenkins_hash_str(b"GetStdHandle");
const WRITEFILE_HASH: u32 = jenkins_hash_str(b"WriteFile");
const LOADLIBRARYA_HASH: u32 = jenkins_hash_str(b"LoadLibraryA");

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

global_asm!(
    ".global _start",
    "_start:",
    "push rbp",
    "mov rbp, rsp",
    "and rsp, -16",
    "sub rsp, 32",
    "call _main",
    "mov rsp, rbp",
    "pop rbp",
    "mov rax, 0",
    "ret"
);

const fn jenkins_hash_str(s: &[u8]) -> u32 {
    let mut hash: u32 = 0;
    let mut i = 0;
    while i < s.len() {
        hash = hash.wrapping_add(s[i] as u32);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
        i += 1;
    }
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    hash
}

fn jenkins_hash(data: *const u8, uppercase: bool) -> u32 {
    let mut hash: u32 = 0;
    let mut i = 0;
    unsafe {
        while *data.add(i) != 0 {
            let mut c = *data.add(i);
            if uppercase && c >= b'a' && c <= b'z' {
                c -= 0x20;
            }
            hash = hash.wrapping_add(c as u32);
            hash = hash.wrapping_add(hash << 10);
            hash ^= hash >> 6;
            i += 1;
        }
    }
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    hash
}

unsafe fn get_peb() -> *mut u8 {
    let peb: *mut u8;
    core::arch::asm!("mov {}, gs:[0x60]", out(reg) peb, options(pure, nomem, nostack));
    peb
}

unsafe fn get_module_by_hash(hash: u32) -> *mut u8 {
    let peb = get_peb();
    let ldr = *((peb as usize + 0x18) as *const usize);
    let mut list_entry = *((ldr + 0x20) as *const usize) as *mut usize;

    while !list_entry.is_null() {
        let base_addr = *(list_entry.add(4)) as *mut u8;
        if base_addr.is_null() {
            break;
        }
        let buffer = *(list_entry.add(10)) as *const u16;
        let length = *(list_entry.add(9)) as u16;

        let mut ascii_name = [0u8; 256];
        let len = (length as usize / 2).min(255);
        for i in 0..len {
            ascii_name[i] = *buffer.add(i) as u8;
        }
        ascii_name[len] = 0;

        if jenkins_hash(ascii_name.as_ptr(), true) == hash {
            return base_addr;
        }
        list_entry = *list_entry as *mut usize;
    }
    ptr::null_mut()
}

unsafe fn get_func_by_hash(module: *mut u8, hash: u32) -> *mut c_void {
    if module.is_null() {
        return ptr::null_mut();
    }
    let dos_header = module as *const u16;
    if *dos_header != 0x5A4D {
        return ptr::null_mut();
    }
    let e_lfanew = *(module.add(0x3C) as *const i32) as usize;
    let nt_headers = module.add(e_lfanew);
    let exp_dir_va = *(nt_headers.add(0x88) as *const u32) as usize;
    if exp_dir_va == 0 {
        return ptr::null_mut();
    }

    let exp_dir = module.add(exp_dir_va);
    let num_names = *(exp_dir.add(0x18) as *const u32) as usize;
    let rva_funcs = module.add(*(exp_dir.add(0x1C) as *const u32) as usize) as *const u32;
    let rva_names = module.add(*(exp_dir.add(0x20) as *const u32) as usize) as *const u32;
    let rva_ords = module.add(*(exp_dir.add(0x24) as *const u32) as usize) as *const u16;

    for i in 0..num_names {
        let func_name = module.add(*rva_names.add(i) as usize);
        if jenkins_hash(func_name, false) == hash {
            let ordinal = *rva_ords.add(i) as usize;
            let func_rva = *rva_funcs.add(ordinal) as usize;
            return module.add(func_rva) as *mut c_void;
        }
    }
    ptr::null_mut()
}

unsafe fn print_stdout(std_out: *mut c_void, write_file: FnWriteFile, msg: &[u8]) {
    let mut written = 0;
    write_file(
        std_out,
        msg.as_ptr(),
        msg.len() as u32,
        &mut written,
        ptr::null_mut(),
    );
}

unsafe extern "C" {
    fn _start();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _main() {
    let kernel32 = get_module_by_hash(KERNEL32_HASH);
    let get_std_handle: FnGetStdHandle =
        core::mem::transmute(get_func_by_hash(kernel32, GETSTDHANDLE_HASH));
    let write_file: FnWriteFile = core::mem::transmute(get_func_by_hash(kernel32, WRITEFILE_HASH));
    let std_out = get_std_handle(0xFFFFFFF5);

    let hello_msg = obf_str!(b"Hello world!\r\n");
    print_stdout(std_out, write_file, &hello_msg);

    let load_library: FnLoadLibraryA =
        core::mem::transmute(get_func_by_hash(kernel32, LOADLIBRARYA_HASH));
    let user32_name = obf_str!(b"user32.dll\0");

    let user32 = load_library(user32_name.as_ptr());
    //let user32 = get_module_by_hash(USER32_HASH);

    let message_box: FnMessageBoxA =
        core::mem::transmute(get_func_by_hash(user32 as _, MESSAGEBOXAHASH));
    let msg_title = obf_str!(b"PIC\0");
    let msg_body = obf_str!(b"Hey!\0");
    message_box(ptr::null_mut(), msg_body.as_ptr(), msg_title.as_ptr(), 0);
}

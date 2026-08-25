use core::ffi::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    unsafe {
        let mut i = 0usize;
        while i < n {
            let av = *a.add(i);
            let bv = *b.add(i);
            if av != bv {
                return if av < bv { -1 } else { 1 };
            }
            i += 1;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn strlen(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    unsafe {
        let mut i = 0usize;
        while *s.add(i) != 0 {
            i += 1;
        }
        i
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if n == 0 {
        return dest;
    }
    unsafe {
        let mut i = 0usize;
        while i < n {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if n == 0 || dest as *const u8 == src {
        return dest;
    }
    unsafe {
        if (dest as usize) < (src as usize) {
            let mut i = 0;
            while i < n {
                *dest.add(i) = *src.add(i);
                i += 1;
            }
        } else {
            let mut i = n;
            while i > 0 {
                i -= 1;
                *dest.add(i) = *src.add(i);
            }
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;
        let val = c as u8;
        while i < n {
            *dest.add(i) = val;
            i += 1;
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub extern "C" fn bcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    memcmp(a, b, n)
}

#[unsafe(no_mangle)]
pub extern "C" fn __CxxFrameHandler3() {}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn debug_break() {
    core::arch::asm!("int3", options(nomem, nostack));
}

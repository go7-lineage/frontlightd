//! LM3630A access over i2c-dev

use crate::log::alogw;
use libc::{c_int, c_ulong};
use std::ffi::CString;

/// Request argument of `ioctl(2)`: `int` in bionic and musl
#[cfg(any(target_os = "android", target_env = "musl"))]
type IoctlRequest = c_int;

/// Request argument of `ioctl(2)`: `unsigned long` elsewhere
#[cfg(not(any(target_os = "android", target_env = "musl")))]
type IoctlRequest = c_ulong;

/// `I2C_SLAVE_FORCE` from Linux `uapi/linux/i2c-dev.h`
const I2C_SLAVE_FORCE: IoctlRequest = 0x0706;

/// LM3630A I2C address (DTBO: `lm3630a@38`)
const ADDRESS: c_ulong = 0x38;

/// Four state registers this daemon writes
const REGS: [u8; 4] = [0x03, 0x04, 0x05, 0x06];

/// An open, claimed LM3630A
pub struct Chip {
    fd: c_int,
}

impl Chip {
    /// Open the adapter at `path` and claim the chip. `None` if either fails
    pub fn open(path: &str) -> Option<Self> {
        let path = CString::new(path).ok()?;

        // SAFETY: `path` is NUL-terminated and only read for the duration of the call
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };
        if fd < 0 {
            return None;
        }

        let chip = Self { fd };
        if chip.claim() {
            Some(chip)
        } else {
            chip.close();
            None
        }
    }

    fn claim(&self) -> bool {
        // SAFETY: `fd` is an open i2c-dev file, ioctl takes the address by value
        unsafe { libc::ioctl(self.fd, I2C_SLAVE_FORCE, ADDRESS) == 0 }
    }

    /// Write the four codes from [`crate::tables::codes`] to registers 0x03..=0x06.
    /// Returns false if any write failed,typically the chip is unpowered and every transaction NACKs
    #[must_use]
    pub fn write(&self, codes: &[u8; 4]) -> bool {
        let mut ok = true;
        for (&reg, code) in REGS.iter().zip(codes.iter()) {
            let buf = [reg, *code];
            // SAFETY: `fd` is open and `buf` lives for duration of the call
            let n = unsafe { libc::write(self.fd, buf.as_ptr().cast(), buf.len()) };
            if n != 2 {
                ok = false;
                alogw!(
                    "write reg {:#04x} failed ({n}): {}",
                    buf[0],
                    std::io::Error::last_os_error()
                );
            }
        }
        ok
    }

    fn close(&self) {
        // SAFETY: `fd` is an open descriptor, closed exactly once here
        let _ = unsafe { libc::close(self.fd) };
    }
}

impl Drop for Chip {
    fn drop(&mut self) {
        self.close();
    }
}

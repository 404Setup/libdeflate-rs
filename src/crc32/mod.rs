use crate::crc32_tables::{CRC32_SLICE1_TABLE, CRC32_SLICE8_TABLE};
use std::sync::OnceLock;

pub fn crc32_slice1(mut crc: u32, p: &[u8]) -> u32 {
    for &b in p {
        crc = (crc >> 8) ^ CRC32_SLICE1_TABLE[(crc as u8 ^ b) as usize];
    }
    crc
}

#[inline]
pub fn crc32_slice8(mut crc: u32, p: &[u8]) -> u32 {
    let mut len = p.len();
    let mut ptr = p.as_ptr();

    while len >= 8 {
        let v = u64::from_le(unsafe { std::ptr::read_unaligned(ptr as *const u64) });
        let v1 = v as u32;
        let v2 = (v >> 32) as u32;

        let idx0 = ((crc ^ v1) as u8) as usize;
        let idx1 = (((crc ^ v1) >> 8) as u8) as usize;
        let idx2 = (((crc ^ v1) >> 16) as u8) as usize;
        let idx3 = (((crc ^ v1) >> 24) as u8) as usize;
        let idx4 = (v2 as u8) as usize;
        let idx5 = ((v2 >> 8) as u8) as usize;
        let idx6 = ((v2 >> 16) as u8) as usize;
        let idx7 = ((v2 >> 24) as u8) as usize;

        let t0 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x700 + idx0) };
        let t1 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x600 + idx1) };
        let t2 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x500 + idx2) };
        let t3 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x400 + idx3) };
        let t4 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x300 + idx4) };
        let t5 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx5) };
        let t6 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx6) };
        let t7 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(idx7) };

        crc = ((t0 ^ t1) ^ (t2 ^ t3)) ^ ((t4 ^ t5) ^ (t6 ^ t7));

        unsafe {
            ptr = ptr.add(8);
        }
        len -= 8;
    }
    if len >= 4 {
        let v = u32::from_le(unsafe { std::ptr::read_unaligned(ptr as *const u32) });
        crc ^= v;
        crc = unsafe {
            *CRC32_SLICE8_TABLE.get_unchecked(0x300 + (crc as u8) as usize)
                ^ *CRC32_SLICE8_TABLE.get_unchecked(0x200 + ((crc >> 8) as u8) as usize)
                ^ *CRC32_SLICE8_TABLE.get_unchecked(0x100 + ((crc >> 16) as u8) as usize)
                ^ *CRC32_SLICE8_TABLE.get_unchecked(((crc >> 24) as u8) as usize)
        };
        unsafe {
            ptr = ptr.add(4);
        }
        len -= 4;
    }
    if len > 0 {
        match len {
            3 => {
                let v = u16::from_le(unsafe { (ptr as *const u16).read_unaligned() }) as u32;
                let b2 = unsafe { *ptr.add(2) } as u32;
                let b0 = v & 0xFF;
                let b1 = v >> 8;

                let idx0 = (crc as u8 as u32) ^ b0;
                let idx1 = ((crc >> 8) as u8 as u32) ^ b1;
                let idx2 = ((crc >> 16) as u8 as u32) ^ b2;

                crc = unsafe {
                    (crc >> 24)
                        ^ *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx0 as usize)
                        ^ *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx1 as usize)
                        ^ *CRC32_SLICE8_TABLE.get_unchecked(idx2 as usize)
                };
            }
            2 => {
                let v = u16::from_le(unsafe { (ptr as *const u16).read_unaligned() }) as u32;
                let b0 = v & 0xFF;
                let b1 = v >> 8;

                let idx0 = (crc as u8 as u32) ^ b0;
                let idx1 = ((crc >> 8) as u8 as u32) ^ b1;

                crc = unsafe {
                    (crc >> 16)
                        ^ *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx0 as usize)
                        ^ *CRC32_SLICE8_TABLE.get_unchecked(idx1 as usize)
                };
            }
            1 => {
                let b0 = unsafe { *ptr } as u32;
                crc = unsafe {
                    (crc >> 8)
                        ^ *CRC32_SLICE8_TABLE.get_unchecked(((crc as u8 as u32) ^ b0) as usize)
                };
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    crc
}

#[cfg(target_arch = "aarch64")]
mod arm;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

type Crc32Fn = unsafe fn(u32, &[u8]) -> u32;

#[inline]
pub fn crc32(crc: u32, slice: &[u8]) -> u32 {
    static IMPL: OnceLock<Crc32Fn> = OnceLock::new();
    let func = IMPL.get_or_init(|| {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            #[cfg(target_arch = "x86_64")]
            if is_x86_feature_detected!("avx512f")
                && is_x86_feature_detected!("avx512bw")
                && is_x86_feature_detected!("avx512vl")
                && is_x86_feature_detected!("vpclmulqdq")
            {
                return x86::crc32_x86_vpclmulqdq_avx512_vl512;
            }

            #[cfg(target_arch = "x86_64")]
            if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("vpclmulqdq") {
                return x86::crc32_x86_vpclmulqdq_avx2;
            }

            if is_x86_feature_detected!("pclmulqdq") && is_x86_feature_detected!("sse4.1") {
                return x86::crc32_x86_pclmulqdq;
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("crc") {
                return arm::crc32_arm;
            }
        }
        crc32_slice8
    });

    unsafe { !func(!crc, slice) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_slice1_empty() {
        let buf = [];
        assert_eq!(crc32_slice1(0, &buf), 0);
        assert_eq!(crc32_slice1(0xFFFFFFFF, &buf), 0xFFFFFFFF);
    }

    #[test]
    fn test_crc32_slice1_basic() {
        let data = b"Hello, World!";
        // The standard CRC32 of "Hello, World!" is 0xEC4AC3D0.
        // Our internal functions (crc32_slice1/8) expect/return !crc.
        // So we expect !0xEC4AC3D0 = 0x13B53C2F.
        // Wait, let's check the verify_crc.rs output.
        // Basic test passed: 0xE33E8552
        // That was !crc32(0, "Hello, World!")?
        // Let's re-verify.
        let res = crc32_slice1(0xFFFFFFFF, data);
        assert_eq!(res ^ 0xFFFFFFFF, 0xEC4AC3D0);
    }

    #[test]
    fn test_crc32_slice1_vs_slice8() {
        for i in 0..256 {
            let data: Vec<u8> = (0..i).map(|j| (j % 255) as u8).collect();
            let r1 = crc32_slice1(0, &data);
            let r8 = crc32_slice8(0, &data);
            assert_eq!(r1, r8, "Mismatch at size {}", i);

            let r1_init = crc32_slice1(0x12345678, &data);
            let r8_init = crc32_slice8(0x12345678, &data);
            assert_eq!(r1_init, r8_init, "Mismatch with initial CRC at size {}", i);
        }
    }
}

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

    // Optimization: Unroll loop to process 64 bytes per iteration.
    // This allows lookups for the high 4 bytes of each 8-byte chunk (which depend only on data)
    // to be interleaved with the dependency chain of the low 4 bytes (which depend on CRC).
    while len >= 64 {
        // First 32 bytes
        let va = u64::from_le(unsafe { std::ptr::read_unaligned(ptr as *const u64) });
        let vb = u64::from_le(unsafe { std::ptr::read_unaligned(ptr.add(8) as *const u64) });
        let vc = u64::from_le(unsafe { std::ptr::read_unaligned(ptr.add(16) as *const u64) });
        let vd = u64::from_le(unsafe { std::ptr::read_unaligned(ptr.add(24) as *const u64) });

        // Second 32 bytes
        let ve = u64::from_le(unsafe { std::ptr::read_unaligned(ptr.add(32) as *const u64) });
        let vf = u64::from_le(unsafe { std::ptr::read_unaligned(ptr.add(40) as *const u64) });
        let vg = u64::from_le(unsafe { std::ptr::read_unaligned(ptr.add(48) as *const u64) });
        let vh = u64::from_le(unsafe { std::ptr::read_unaligned(ptr.add(56) as *const u64) });

        let va1 = va as u32;
        let va2 = (va >> 32) as u32;
        let vb1 = vb as u32;
        let vb2 = (vb >> 32) as u32;
        let vc1 = vc as u32;
        let vc2 = (vc >> 32) as u32;
        let vd1 = vd as u32;
        let vd2 = (vd >> 32) as u32;

        let ve1 = ve as u32;
        let ve2 = (ve >> 32) as u32;
        let vf1 = vf as u32;
        let vf2 = (vf >> 32) as u32;
        let vg1 = vg as u32;
        let vg2 = (vg >> 32) as u32;
        let vh1 = vh as u32;
        let vh2 = (vh >> 32) as u32;

        // Independent lookups for high parts (va2..vh2)
        // Group A (va2)
        let idx4 = (va2 as u8) as usize;
        let idx5 = ((va2 >> 8) as u8) as usize;
        let idx6 = ((va2 >> 16) as u8) as usize;
        let idx7 = ((va2 >> 24) as u8) as usize;
        // Group B (vb2)
        let idx12 = (vb2 as u8) as usize;
        let idx13 = ((vb2 >> 8) as u8) as usize;
        let idx14 = ((vb2 >> 16) as u8) as usize;
        let idx15 = ((vb2 >> 24) as u8) as usize;
        // Group C (vc2)
        let idx20 = (vc2 as u8) as usize;
        let idx21 = ((vc2 >> 8) as u8) as usize;
        let idx22 = ((vc2 >> 16) as u8) as usize;
        let idx23 = ((vc2 >> 24) as u8) as usize;
        // Group D (vd2)
        let idx28 = (vd2 as u8) as usize;
        let idx29 = ((vd2 >> 8) as u8) as usize;
        let idx30 = ((vd2 >> 16) as u8) as usize;
        let idx31 = ((vd2 >> 24) as u8) as usize;

        // Start independent fetches for first 32 bytes
        let t4 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x300 + idx4) };
        let t5 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx5) };
        let t6 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx6) };
        let t7 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(idx7) };

        let t12 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x300 + idx12) };
        let t13 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx13) };
        let t14 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx14) };
        let t15 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(idx15) };

        let t20 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x300 + idx20) };
        let t21 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx21) };
        let t22 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx22) };
        let t23 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(idx23) };

        let t28 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x300 + idx28) };
        let t29 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx29) };
        let t30 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx30) };
        let t31 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(idx31) };

        // Dependent chain for first 32 bytes
        // Chunk A
        let idx0 = ((crc ^ va1) as u8) as usize;
        let idx1 = (((crc ^ va1) >> 8) as u8) as usize;
        let idx2 = (((crc ^ va1) >> 16) as u8) as usize;
        let idx3 = (((crc ^ va1) >> 24) as u8) as usize;
        let t0 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x700 + idx0) };
        let t1 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x600 + idx1) };
        let t2 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x500 + idx2) };
        let t3 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x400 + idx3) };
        crc = ((t0 ^ t1) ^ (t2 ^ t3)) ^ ((t4 ^ t5) ^ (t6 ^ t7));

        // Chunk B
        let idx8 = ((crc ^ vb1) as u8) as usize;
        let idx9 = (((crc ^ vb1) >> 8) as u8) as usize;
        let idx10 = (((crc ^ vb1) >> 16) as u8) as usize;
        let idx11 = (((crc ^ vb1) >> 24) as u8) as usize;
        let t8 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x700 + idx8) };
        let t9 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x600 + idx9) };
        let t10 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x500 + idx10) };
        let t11 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x400 + idx11) };
        crc = ((t8 ^ t9) ^ (t10 ^ t11)) ^ ((t12 ^ t13) ^ (t14 ^ t15));

        // Chunk C
        let idx16 = ((crc ^ vc1) as u8) as usize;
        let idx17 = (((crc ^ vc1) >> 8) as u8) as usize;
        let idx18 = (((crc ^ vc1) >> 16) as u8) as usize;
        let idx19 = (((crc ^ vc1) >> 24) as u8) as usize;
        let t16 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x700 + idx16) };
        let t17 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x600 + idx17) };
        let t18 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x500 + idx18) };
        let t19 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x400 + idx19) };
        crc = ((t16 ^ t17) ^ (t18 ^ t19)) ^ ((t20 ^ t21) ^ (t22 ^ t23));

        // Chunk D
        let idx24 = ((crc ^ vd1) as u8) as usize;
        let idx25 = (((crc ^ vd1) >> 8) as u8) as usize;
        let idx26 = (((crc ^ vd1) >> 16) as u8) as usize;
        let idx27 = (((crc ^ vd1) >> 24) as u8) as usize;
        let t24 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x700 + idx24) };
        let t25 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x600 + idx25) };
        let t26 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x500 + idx26) };
        let t27 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x400 + idx27) };
        crc = ((t24 ^ t25) ^ (t26 ^ t27)) ^ ((t28 ^ t29) ^ (t30 ^ t31));

        // Now process second 32 bytes (chunks E, F, G, H)
        // Group E (ve2)
        let idx36 = (ve2 as u8) as usize;
        let idx37 = ((ve2 >> 8) as u8) as usize;
        let idx38 = ((ve2 >> 16) as u8) as usize;
        let idx39 = ((ve2 >> 24) as u8) as usize;
        // Group F (vf2)
        let idx44 = (vf2 as u8) as usize;
        let idx45 = ((vf2 >> 8) as u8) as usize;
        let idx46 = ((vf2 >> 16) as u8) as usize;
        let idx47 = ((vf2 >> 24) as u8) as usize;
        // Group G (vg2)
        let idx52 = (vg2 as u8) as usize;
        let idx53 = ((vg2 >> 8) as u8) as usize;
        let idx54 = ((vg2 >> 16) as u8) as usize;
        let idx55 = ((vg2 >> 24) as u8) as usize;
        // Group H (vh2)
        let idx60 = (vh2 as u8) as usize;
        let idx61 = ((vh2 >> 8) as u8) as usize;
        let idx62 = ((vh2 >> 16) as u8) as usize;
        let idx63 = ((vh2 >> 24) as u8) as usize;

        // Independent fetches for second 32 bytes
        let t36 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x300 + idx36) };
        let t37 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx37) };
        let t38 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx38) };
        let t39 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(idx39) };

        let t44 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x300 + idx44) };
        let t45 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx45) };
        let t46 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx46) };
        let t47 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(idx47) };

        let t52 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x300 + idx52) };
        let t53 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx53) };
        let t54 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx54) };
        let t55 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(idx55) };

        let t60 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x300 + idx60) };
        let t61 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x200 + idx61) };
        let t62 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x100 + idx62) };
        let t63 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(idx63) };

        // Dependent chain for second 32 bytes
        // Chunk E
        let idx32 = ((crc ^ ve1) as u8) as usize;
        let idx33 = (((crc ^ ve1) >> 8) as u8) as usize;
        let idx34 = (((crc ^ ve1) >> 16) as u8) as usize;
        let idx35 = (((crc ^ ve1) >> 24) as u8) as usize;
        let t32 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x700 + idx32) };
        let t33 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x600 + idx33) };
        let t34 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x500 + idx34) };
        let t35 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x400 + idx35) };
        crc = ((t32 ^ t33) ^ (t34 ^ t35)) ^ ((t36 ^ t37) ^ (t38 ^ t39));

        // Chunk F
        let idx40 = ((crc ^ vf1) as u8) as usize;
        let idx41 = (((crc ^ vf1) >> 8) as u8) as usize;
        let idx42 = (((crc ^ vf1) >> 16) as u8) as usize;
        let idx43 = (((crc ^ vf1) >> 24) as u8) as usize;
        let t40 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x700 + idx40) };
        let t41 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x600 + idx41) };
        let t42 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x500 + idx42) };
        let t43 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x400 + idx43) };
        crc = ((t40 ^ t41) ^ (t42 ^ t43)) ^ ((t44 ^ t45) ^ (t46 ^ t47));

        // Chunk G
        let idx48 = ((crc ^ vg1) as u8) as usize;
        let idx49 = (((crc ^ vg1) >> 8) as u8) as usize;
        let idx50 = (((crc ^ vg1) >> 16) as u8) as usize;
        let idx51 = (((crc ^ vg1) >> 24) as u8) as usize;
        let t48 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x700 + idx48) };
        let t49 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x600 + idx49) };
        let t50 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x500 + idx50) };
        let t51 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x400 + idx51) };
        crc = ((t48 ^ t49) ^ (t50 ^ t51)) ^ ((t52 ^ t53) ^ (t54 ^ t55));

        // Chunk H
        let idx56 = ((crc ^ vh1) as u8) as usize;
        let idx57 = (((crc ^ vh1) >> 8) as u8) as usize;
        let idx58 = (((crc ^ vh1) >> 16) as u8) as usize;
        let idx59 = (((crc ^ vh1) >> 24) as u8) as usize;
        let t56 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x700 + idx56) };
        let t57 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x600 + idx57) };
        let t58 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x500 + idx58) };
        let t59 = unsafe { *CRC32_SLICE8_TABLE.get_unchecked(0x400 + idx59) };
        crc = ((t56 ^ t57) ^ (t58 ^ t59)) ^ ((t60 ^ t61) ^ (t62 ^ t63));

        unsafe {
            ptr = ptr.add(64);
        }
        len -= 64;
    }

    // Fallback for remaining chunks < 64 bytes
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

        // Optimization: Use tree-based XOR reduction to break dependency chains and increase ILP.
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

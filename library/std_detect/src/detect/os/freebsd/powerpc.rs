//! Run-time feature detection for PowerPC on FreeBSD.
//!
//! Constants for both 32- and 64-bit come from
//! <https://github.com/freebsd/freebsd-src/blob/main/sys/powerpc/include/cpu.h>.

use super::auxvec;
use crate::detect::{Feature, cache};

const PPC_FEATURE_HAS_ALTIVEC: usize = 0x10000000;
const PPC_FEATURE_HAS_VSX: usize = 0x00000080;
/// ISA 2.06, typically power6
const PPC_FEATURE_ARCH_2_06: usize = 0x00000100;
/// ISA 2.07, typically power8
const PPC_FEATURE2_ARCH_2_07: usize = 0x80000000;
/// ISA 3.00, typically power9
const PPC_FEATURE2_ARCH_3_00: usize = 0x00800000;
/// ISA 3.1, typically power10
const PPC_FEATURE2_ARCH_3_1: usize = 0x00040000;

pub(crate) fn detect_features() -> cache::Initializer {
    let mut value = cache::Initializer::default();
    let enable_feature = |value: &mut cache::Initializer, f, enable| {
        if enable {
            value.set(f as u32);
        }
    };

    if let Ok(auxv) = auxvec::auxv() {
        enable_feature(&mut value, Feature::altivec, auxv.hwcap & PPC_FEATURE_HAS_ALTIVEC != 0);
        enable_feature(&mut value, Feature::vsx, auxv.hwcap & PPC_FEATURE_HAS_VSX != 0);
        enable_feature(&mut value, Feature::isa_2_06, auxv.hwcap & PPC_FEATURE_ARCH_2_06 != 0);
        enable_feature(&mut value, Feature::isa_2_07, auxv.hwcap2 & PPC_FEATURE2_ARCH_2_07 != 0);
        enable_feature(&mut value, Feature::isa_3_0, auxv.hwcap2 & PPC_FEATURE2_ARCH_3_00 != 0);
        enable_feature(&mut value, Feature::isa_3_1, auxv.hwcap2 & PPC_FEATURE2_ARCH_3_1 != 0);
        return value;
    }
    value
}

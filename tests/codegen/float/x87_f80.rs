//@ only-x86_64

#![feature(lang_items)]
#![crate_type = "lib"]

use std::ops;

#[lang = "x87_ext_double"]
#[allow(non_camel_case_types)]
struct x87_f80(u128);

impl ops::Add for x87_f80 {
    type Output = x87_f80;

    #[inline]
    #[track_caller]
    fn add(self, other: Self) -> Self {
        self + other
    }
}

// x86-nosse-LABEL: void @f80_add(x86_fp80
#[no_mangle]
pub fn f80_add(a: x87_f80, b: x87_f80) -> x87_f80 {
    // CHECK: fadd x86_fp80 %{{.+}}, %{{.+}}
    a + b
}

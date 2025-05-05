#![feature(abi_custom)]

unsafe extern "custom" {
    // Usual unsafe functions are allowed.
    fn extern_unsafe();

    // Trying to use `safe` is an error.
    safe fn extern_safe();
    //~^ ERROR `extern "custom"` functions cannot be marked `safe`
}

// By default, `custom` functions cannot be defined.
extern "custom" fn fn_def() {}
//~^ ERROR custom abi cannot be defined

// This should be rejected anyway, but check with closures (which inherit parent ABI).
extern "custom" fn fn_def_closure() {
    //~^ ERROR custom abi cannot be defined
    let x = || {};
    x();
}

// Naked functions can define functions with the `custom` ABI.
#[unsafe(naked)]
unsafe extern "custom" fn naked_fn_def() {
    core::arch::naked_asm!("nop")
}

// `custom` ABI functions must be marked `unsafe`.
#[unsafe(naked)]
extern "custom" fn naked_fn_def() {
    //~^ ERROR missing `unsafe`
    core::arch::naked_asm!("nop")
}

// Function parameters do nothing so are rejected.
#[unsafe(naked)]
unsafe extern "custom" fn naked_fn_def_params(a: u32) {
    //~^ ERROR no params allowed
    core::arch::naked_asm!("nop")
}

// Same for return types.
#[unsafe(naked)]
unsafe extern "custom" fn naked_fn_def_ret() -> u32 {
    //~^ ERROR no return types allowed
    core::arch::naked_asm!("nop")
}

// Check that passing as a function pointer works.
pub fn fn_ptr(_x: extern "unsafe" fn()) {}


// Check that usage as a `sym` in assembly works.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn use_as_sym() {
    unsafe {
        core::arch::asm!(
            "call {foo}",
            foo = sym foo,
        )
    }
}

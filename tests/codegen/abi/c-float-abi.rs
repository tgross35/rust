#![feature(f16, f128)]

//! Verify that Rust provides the correct calling convention information to LLVM for floattypes.
//!
//! This corresponds to the test in `assembly/abi/c-float-abi.rs`. Keep the files in sync!
//!
//! Only the function signatures are checked here because that dictates how LLVM expects them to
//! interface. The assembly check validates more.

// Eliminate intermediate instructions during `nop` tests
//@ compile-flags: -Copt-level=1

//@ add-core-stubs
//@ revisions: AARCH64 ARM32 MINGW64 MSVC64 NOSSE32 NOSSE64 SYSVX32 SYSVX64 WASM32 WIN64-SOFT
// revisions: AARCH64 ARM32 MINGW64 MSVC32 MSVC64 NOSSE32 NOSSE64 RISCV64 SYSVX32 SYSVX64 WASM32 WIN64-SOFT

//@ [AARCH64] compile-flags: --target aarch64-unknown-linux-gnu
//@ [AARCH64] needs-llvm-components: aarch64
//@ [ARM32] compile-flags: --target armv7-unknown-linux-gnueabihf
//@ [ARM32] needs-llvm-components: arm
//@ [MINGW64] compile-flags: --target x86_64-pc-windows-gnu
//@ [MINGW64] needs-llvm-components: x86
//@ [MSVC32] compile-flags: --target i686-pc-windows-msvc
//@ [MSVC32] needs-llvm-components: x86
//@ [MSVC64] compile-flags: --target x86_64-pc-windows-msvc
//@ [MSVC64] needs-llvm-components: x86
//@ [NOSSE32] compile-flags: --target i586-unknown-linux-gnu
//@ [NOSSE32] needs-llvm-components: x86
//@ [NOSSE64] compile-flags: --target x86_64-unknown-none
//@ [NOSSE64] needs-llvm-components: x86
//@ [RISCV64] compile-flags: --target riscv64gc-unknown-linux-gnu
//@ [RISCV64] needs-llvm-components: riscv
//@ [SYSVX32] compile-flags: --target i686-unknown-linux-gnu
//@ [SYSVX32] needs-llvm-components: x86
//@ [SYSVX64] compile-flags: --target x86_64-unknown-linux-gnu
//@ [SYSVX64] needs-llvm-components: x86
//@ [WASM32] compile-flags: --target wasm32-wasip1
//@ [WASM32] needs-llvm-components: webassembly
// UEFI uses the windows calling convention without SSE
//@ [WIN64-SOFT] compile-flags: --target x86_64-unknown-uefi
//@ [WIN64-SOFT] needs-llvm-components: x86

// Add aliases for common of targets (pointer width and OS)
//@ [AARCH64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-HF,CHECK-NO-WIN,CHECK-64B-NO-WIN
//@ [ARM32]      filecheck-flags: --check-prefixes CHECK-32B,CHECK-HF,CHECK-NO-WIN
//@ [MINGW64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-HF,CHECK-IS-WIN,CHECK-64B-IS-WIN
//@ [MSVC32]     filecheck-flags: --check-prefixes CHECK-32B,CHECK-HF,CHECK-IS-WIN
//@ [MSVC64]     filecheck-flags: --check-prefixes CHECK-64B,CHECK-HF,CHECK-IS-WIN,CHECK-64B-IS-WIN
//@ [NOSSE32]    filecheck-flags: --check-prefixes CHECK-32B,CHECK-SF,CHECK-NO-WIN
//@ [NOSSE64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-SF,CHECK-NO-WIN,CHECK-64B-NO-WIN
//@ [RISCV64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-HF,CHECK-NO-WIN,CHECK-64B-NO-WIN
//@ [SYSVX32]    filecheck-flags: --check-prefixes CHECK-32B,CHECK-HF,CHECK-NO-WIN
//@ [SYSVX64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-HF,CHECK-NO-WIN,CHECK-64B-NO-WIN
//@ [WASM32]     filecheck-flags: --check-prefixes CHECK-32B,CHECK-HF,CHECK-NO-WIN
//@ [WIN64-SOFT] filecheck-flags: --check-prefixes CHECK-64B,CHECK-SF,CHECK-IS-WIN,CHECK-64B-IS-WIN

#![crate_type = "lib"]
#![no_std]
#![no_core]
#![feature(no_core, lang_items)]

extern crate minicore;

#[repr(C)]
pub struct Aggregate<T> {
    a: i32,
    b: T,
}

// FOO: sodijfoi

/* f16 */

#[no_mangle]
pub extern "C" fn pass_f16(a: f16, ret: &mut f16) {
    // f16 is always passed directly
    // CHECK: void @pass_f16(half{{.*}} {{%.+}}, ptr{{.*}} %ret)
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_f16(a: &f16) -> f16 {
    // f16 is always returned directly
    // CHECK: half @ret_f16(ptr{{.*}} %a)
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_f16(a: Aggregate<f16>, dst: &mut f16)  {
    // Passed as a larger integer on 64-bit
    // CHECK-64B: void @pass_aggregate_f16(i64 %0, ptr{{.*}} %dst)

    // Passed on the stack on 32-bit
    // MSVC32:    void @pass_aggregate_f16(ptr{{.*}} %a, ptr{{.*}} %dst)
    // NOSSE32:   void @pass_aggregate_f16(ptr{{.*}} %a, ptr{{.*}} %dst)
    // SYSVX32:   void @pass_aggregate_f16(ptr{{.*}} %a, ptr{{.*}} %dst)
    // WASM32:    void @pass_aggregate_f16(ptr{{.*}} %a, ptr{{.*}} %dst)

    // Passed as an array on arm32
    // ARM32:     void @pass_aggregate_f16([2 x i32] {{%.+}}, ptr{{.*}} %dst)
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_f16(a: i32, b: &f16) -> Aggregate<f16> {
    // Returned as a larger integer on 64-bit
    // CHECK-64B: i64 @ret_aggregate_f16(i32{{.*}} %a, ptr{{.*}} %b)

    // Returned indirectly on 32-bit
    // CHECK-32B: void @ret_aggregate_f16(ptr{{.*}} {{%.+}}, i32{{.*}} %a, ptr{{.*}} %b)
    Aggregate { a, b: *b }
}

/* f32 */

#[no_mangle]
pub extern "C" fn pass_f32(a: f32, ret: &mut f32) {
    // f32 is always passed directly
    // CHECK: void @pass_f32(float{{.*}} {{%.+}}, ptr{{.*}} %ret)
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_f32(a: &f32) -> f32 {
    // f32 is always returned directly
    // CHECK: float @ret_f32(ptr{{.*}} %a)
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_f32(a: Aggregate<f32>, dst: &mut f32)  {
    // Passed as a single integer on 64-bit
    // CHECK-64B: void @pass_aggregate_f32(i64{{.*}} %0, ptr{{.*}} %dst)
    
    // Passed on the stack on most 32-bit platforms
    // MSVC32:    void @pass_aggregate_f32(ptr{{.*}} %a, ptr{{.*}} %dst)
    // NOSSE32:   void @pass_aggregate_f32(ptr{{.*}} %a, ptr{{.*}} %dst)
    // SYSVX32:   void @pass_aggregate_f32(ptr{{.*}} %a, ptr{{.*}} %dst)
    // WASM32:    void @pass_aggregate_f32(ptr{{.*}} %a, ptr{{.*}} %dst)
    
    // Passed indirectly on 32-bit
    // ARM32:     void @pass_aggregate_f32([2 x i32] {{%.+}}, ptr{{.*}} %dst)
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_f32(a: i32, b: &f32) -> Aggregate<f32> {
    // Returned as a larger integer on 64-bit
    // CHECK-64B: i64 @ret_aggregate_f32(i32{{.*}} %a, ptr{{.*}} %b)
    
    // Returned indirectly on 32-bit
    // CHECK-32B: void @ret_aggregate_f32(ptr{{.*}} {{%.+}}, i32{{.*}} %a, ptr{{.*}} %b)
    Aggregate { a, b: *b }
}

/* f64 */

#[no_mangle]
pub extern "C" fn pass_f64(a: f64, ret: &mut f64) {
    // f64 is always passed directly
    // CHECK: void @pass_f64(double{{.*}} {{%.+}}, ptr{{.*}} %ret)
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_f64(a: &f64) -> f64 {
    // f64 is always returned directly
    // CHECK: double @ret_f64(ptr{{.*}} %a)
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_f64(a: Aggregate<f64>, dst: &mut f64)  {
    // Passed as a scalar pair { f64, f64 } on 64-bit
    // NOSSE64:    void @pass_aggregate_f64({ i64, double } {{%.+}}, ptr{{.*}} %dst)
    // RISCV64:    void @pass_aggregate_f64({ i64, double } {{%.+}}, ptr{{.*}} %dst)
    // SYSVX64:    void @pass_aggregate_f64({ i64, double } {{%.+}}, ptr{{.*}} %dst)
    
    // Passed on the stack on Windows and 32-bit excluding arm
    // MSVC32:           void @pass_aggregate_f64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // WASM32:           void @pass_aggregate_f64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // MINGW64:          void @pass_aggregate_f64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // MSVC32:           void @pass_aggregate_f64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // MSVC64:           void @pass_aggregate_f64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // NOSSE32:          void @pass_aggregate_f64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // SYSVX32:          void @pass_aggregate_f64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // WIN64-SOFT:       void @pass_aggregate_f64(ptr{{.*}} %a, ptr{{.*}} %dst)

    // Passed as an array on arm32
    // AARCH64:         void @pass_aggregate_f64([2 x i64] {{%.+}}, ptr{{.*}} %dst)
    // ARM32:           void @pass_aggregate_f64([2 x i64] {{%.+}}, ptr{{.*}} %dst)
    *dst = a.b
}

 // define void @pass_aggregate_f64({ i64, double }

#[no_mangle]
pub extern "C" fn ret_aggregate_f64(a: i32, b: &f64) -> Aggregate<f64> {
    // Passed as a scalar pair { f64, double }  on 64-bit
    // CHECK-64B-NO-WIN: { i64, double } @ret_aggregate_f64(i32{{.*}} %a, ptr{{.*}} %b)

    // On Windows and everything 32-bit, the struct is returned on the stack
    // CHECK-64B-IS-WIN: void @ret_aggregate_f64(ptr{{.*}} {{%.*}}, i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-32B:        void @ret_aggregate_f64(ptr{{.*}} {{%.+}}, i32{{.*}} %a, ptr{{.*}} %b)
    Aggregate { a, b: *b }
}


/* f128 */

#[no_mangle]
pub extern "C" fn pass_f128(a: f128, ret: &mut f128) {
    // f128 is usually passed directly
    // CHECK-NO-WIN: void @pass_f128(fp128{{.*}} %a, ptr{{.*}} %ret)

    // FIXME(f16_f128): Windows should be passing on the stack, LLVM may do this but we should do
    // this too.
    // CHECK-IS-WIN: void @pass_f128(fp128{{.*}} %a, ptr{{.*}} %ret)
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_f128(a: &f128) -> f128 {
    // f128 is usually returned with default handling
    // AARCH64:    fp128 @ret_f128(ptr{{.*}} %a)
    // ARM32:      fp128 @ret_f128(ptr{{.*}} %a)
    // SYSVX32:    fp128 @ret_f128(ptr{{.*}} %a)
    // SYSVX64:    fp128 @ret_f128(ptr{{.*}} %a)
    // RISCV64:    fp128 @ret_f128(ptr{{.*}} %a)

    // On Windows, f128 is returned with default handling in xmm0 if available
    // FIXME(f16_f128): Windows should be returning in XMM0, this is a recent LLVM change.
    // MINGW64:    fp128 @ret_f128(ptr{{.*}} %a)
    // MSVC32:     fp128 @ret_f128(ptr{{.*}} %a)
    // MSVC64:     fp128 @ret_f128(ptr{{.*}} %a)
    // NOSSE32:    fp128 @ret_f128(ptr{{.*}} %a)

    // On Windows without SSE registers, we use the default f128
    // WIN64-SOFT: fp128 @ret_f128(ptr{{.*}} %a)

    // On Wasm, f128 is returned indirectly
    // WASM32:     void @ret_f128(ptr{{.*}} {{%.*}}, ptr{{.*}} %a)
    *a
}

// COM: #[no_mangle]
// COM: pub extern "C" fn pass_aggregate_f128(a: Aggregate<f128>, dst: &mut f128)  {
// COM:     // On most targets this is passed indirectly
// COM:     // AARCH64:    void @pass_aggregate_f128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
// COM:     // SYSVX32:    void @pass_aggregate_f128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
// COM:     // SYSVX64:    void @pass_aggregate_f128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
// COM:     // MINGW64:    void @pass_aggregate_f128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
// COM:     // MSVC32:     void @pass_aggregate_f128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
// COM:     // MSVC64:     void @pass_aggregate_f128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
// COM:     // RISCV64:    void @pass_aggregate_f128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
// COM:     // WASM32:     void @pass_aggregate_f128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
// COM:     // WIN64-SOFT: void @pass_aggregate_f128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
// COM: 
// COM:     // Except arm32, where this appears as an array to return in registers.
// COM:     // ARM32:      void @pass_aggregate_f128([3 x i64] {{%.+}}, ptr{{.*}} %dst)
// COM:     *dst = a.b
// COM: }
// COM: 
// COM: 
// COM: #[no_mangle]
// COM: pub extern "C" fn ret_aggregate_f128(a: i32, b: &f128) -> Aggregate<f128> {
// COM:     // This type is always returned on the stack
// COM:     // CHECK: void @ret_aggregate_f128(ptr{{.*}} {{%.+}}, i32{{.*}} %a, ptr{{.*}} %b)
// COM:     Aggregate { a, b: *b }
// COM: }

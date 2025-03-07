//! Verify that Rust provides the correct calling convention information to LLVM for integer types.
//!
//! This corresponds to the test in `assembly/abi/c-integer-abi.rs`. Keep the files in sync!
//!
//! Only the function signatures are checked here because that dictates how LLVM expects them to
//! interface. The assembly check validates more.

// Eliminate intermediate instructions during `nop` tests
//@ compile-flags: -Copt-level=1

//@ add-core-stubs
//@ revisions: AARCH64 ARM32 MINGW64 MSVC32 MSVC64 NOSSE32 NOSSE64 RISCV64 SYSVX32 SYSVX64 WASM32 WIN64-SOFT

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
//@ [AARCH64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-NO-WIN,CHECK-64B-NO-WIN
//@ [ARM32]      filecheck-flags: --check-prefixes CHECK-32B,CHECK-NO-WIN,CHECK-32B-NO-WIN
//@ [MINGW64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-IS-WIN
//@ [MSVC32]     filecheck-flags: --check-prefixes CHECK-32B,CHECK-IS-WIN
//@ [MSVC64]     filecheck-flags: --check-prefixes CHECK-64B,CHECK-IS-WIN
//@ [NOSSE32]    filecheck-flags: --check-prefixes CHECK-32B,CHECK-NO-WIN,CHECK-32B-NO-WIN
//@ [NOSSE64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-NO-WIN,CHECK-64B-NO-WIN
//@ [RISCV64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-NO-WIN,CHECK-64B-NO-WIN
//@ [SYSVX32]    filecheck-flags: --check-prefixes CHECK-32B,CHECK-NO-WIN,CHECK-32B-NO-WIN
//@ [SYSVX64]    filecheck-flags: --check-prefixes CHECK-64B,CHECK-NO-WIN,CHECK-64B-NO-WIN
//@ [WASM32]     filecheck-flags: --check-prefixes CHECK-32B,CHECK-NO-WIN,CHECK-32B-NO-WIN
//@ [WIN64-SOFT] filecheck-flags: --check-prefixes CHECK-64B,CHECK-IS-WIN

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

/* i8 */

#[no_mangle]
pub extern "C" fn pass_i8(a: i8, ret: &mut i8) {
    // i8 is always passed directly
    // CHECK: void @pass_i8(i8{{.*}} %a, ptr{{.*}} %ret)
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i8(a: &i8) -> i8 {
    // i8 is always returned directly
    // CHECK: i8 @ret_i8(ptr{{.*}} %a)
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_i8(a: Aggregate<i8>, dst: &mut i8)  {
    // Passed as a single integer on 64-bit
    // CHECK-64B: void @pass_aggregate_i8(i64{{.*}} %0, ptr{{.*}} %dst)

    // Passed on the stack on most 32-bit platforms
    // MSVC32:    void @pass_aggregate_i8(ptr{{.*}} %a, ptr{{.*}} %dst)
    // NOSSE32:   void @pass_aggregate_i8(ptr{{.*}} %a, ptr{{.*}} %dst)
    // SYSVX32:   void @pass_aggregate_i8(ptr{{.*}} %a, ptr{{.*}} %dst)
    // WASM32:    void @pass_aggregate_i8(ptr{{.*}} %a, ptr{{.*}} %dst)

    // Passed as an array on arm32
    // ARM32:     void @pass_aggregate_i8([2 x i32] {{%.+}}, ptr{{.*}} %dst)
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i8(a: i32, b: &i8) -> Aggregate<i8> {
    // Returned as a larger integer on 64-bit and MSVC
    // CHECK-64B: i64 @ret_aggregate_i8(i32{{.*}} %a, ptr{{.*}} %b)
    // MSVC32:    i64 @ret_aggregate_i8(i32{{.*}} %a, ptr{{.*}} %b)

    // Returned indirectly on 32-bit
    // CHECK-32B-NO-WIN: void @ret_aggregate_i8(ptr{{.*}} {{%.+}}, i32{{.*}} %a, ptr{{.*}} %b)
    Aggregate { a, b: *b }
}

/* i16 */

#[no_mangle]
pub extern "C" fn pass_i16(a: i16, ret: &mut i16) {
    // i16 is always passed directly
    // CHECK: void @pass_i16(i16{{.*}} %a, ptr{{.*}} %ret)
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i16(a: &i16) -> i16 {
    // i16 is always returned directly
    // CHECK: i16 @ret_i16(ptr{{.*}} %a)
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_i16(a: Aggregate<i16>, dst: &mut i16)  {
    // Passed as a larger integer on 64-bit
    // CHECK-64B: void @pass_aggregate_i16(i64{{.*}} %0, ptr{{.*}} %dst)

    // Passed on the stack on 32-bit
    // MSVC32:    void @pass_aggregate_i16(ptr{{.*}} %a, ptr{{.*}} %dst)
    // NOSSE32:   void @pass_aggregate_i16(ptr{{.*}} %a, ptr{{.*}} %dst)
    // SYSVX32:   void @pass_aggregate_i16(ptr{{.*}} %a, ptr{{.*}} %dst)
    // WASM32:    void @pass_aggregate_i16(ptr{{.*}} %a, ptr{{.*}} %dst)

    // Passed as an array on arm32
    // ARM32:     void @pass_aggregate_i16([2 x i32] {{%.+}}, ptr{{.*}} %dst)
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i16(a: i32, b: &i16) -> Aggregate<i16> {
    // Returned as a larger integer on 64-bit and MSVC
    // CHECK-64B: i64 @ret_aggregate_i16(i32{{.*}} %a, ptr{{.*}} %b)
    // MSVC:      i64 @ret_aggregate_i16(i32{{.*}} %a, ptr{{.*}} %b)

    // Returned indirectly on 32-bit
    // CHECK-32B-NO-WIN: void @ret_aggregate_i16(ptr{{.*}} {{%.+}}, i32{{.*}} %a, ptr{{.*}} %b)
    Aggregate { a, b: *b }
}


/* i32 */

#[no_mangle]
pub extern "C" fn pass_i32(a: i32, ret: &mut i32) {
    // i32 is always passed directly
    // CHECK: void @pass_i32(i32{{.*}} %a, ptr{{.*}} %ret)
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i32(a: &i32) -> i32 {
    // i32 is always returned directly
    // CHECK: i32 @ret_i32(ptr{{.*}} %a)
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_i32(a: Aggregate<i32>, dst: &mut i32)  {
    // Passed as a single integer on 64-bit
    // CHECK-64B: void @pass_aggregate_i32(i64{{.*}} %0, ptr{{.*}} %dst)

    // Passed on the stack on most 32-bit platforms
    // MSVC32:    void @pass_aggregate_i32(ptr{{.*}} %a, ptr{{.*}} %dst)
    // NOSSE32:   void @pass_aggregate_i32(ptr{{.*}} %a, ptr{{.*}} %dst)
    // SYSVX32:   void @pass_aggregate_i32(ptr{{.*}} %a, ptr{{.*}} %dst)
    // WASM32:    void @pass_aggregate_i32(ptr{{.*}} %a, ptr{{.*}} %dst)

    // Passed indirectly on 32-bit
    // ARM32:     void @pass_aggregate_i32([2 x i32] {{%.+}}, ptr{{.*}} %dst)
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i32(a: i32, b: &i32) -> Aggregate<i32> {
    // Returned as a larger integer on 64-bit and MSVC
    // CHECK-64B: i64 @ret_aggregate_i32(i32{{.*}} %a, ptr{{.*}} %b)
    // MSVC:      i64 @ret_aggregate_i32(i32{{.*}} %a, ptr{{.*}} %b)

    // Returned indirectly on 32-bit
    // CHECK-32B-NO-WIN: void @ret_aggregate_i32(ptr{{.*}} {{%.+}}, i32{{.*}} %a, ptr{{.*}} %b)
    Aggregate { a, b: *b }
}

/* i64 */

#[no_mangle]
pub extern "C" fn pass_i64(a: i64, ret: &mut i64) {
    // i64 is always passed directly
    // CHECK: void @pass_i64(i64{{.*}} %a, ptr{{.*}} %ret)
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i64(a: &i64) -> i64 {
    // i64 is always returned directly
    // CHECK: i64 @ret_i64(ptr{{.*}} %a)
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_i64(a: Aggregate<i64>, dst: &mut i64)  {
    // Passed as a scalar pair { i64, i64 } or array [2 x i64] on 64-bit
    // CHECK-64B-NO-WIN: void @pass_aggregate_i64({{(\{ i64, i64 \}|\[2 x i64\]).*}} {{%.+}}, ptr{{.*}} %dst)

    // Passed on the stack on Windows and 32-bit excluding arm
    // CHECK-64B-IS-WIN: void @pass_aggregate_i64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // MINGW32:          void @pass_aggregate_i64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // NOSSE32:          void @pass_aggregate_i64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // MSVC32:           void @pass_aggregate_i64(ptr{{.*}} %0, ptr{{.*}} %dst)
    // SYSVX32:          void @pass_aggregate_i64(ptr{{.*}} %a, ptr{{.*}} %dst)

    // Passed as an array on arm32
    // ARM32:            void @pass_aggregate_i64([2 x i64] {{%.+}}, ptr{{.*}} %dst)
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i64(a: i32, b: &i64) -> Aggregate<i64> {
    // Passed as a scalar pair { i64, i64 } or array [2 x i64] on 64-bit
    // CHECK-64B-NO-WIN: {{(\{ i64, i64 \}|\[2 x i64\])}} @ret_aggregate_i64(i32{{.*}} %a, ptr{{.*}} %b)

    // On Windows and everything 32-bit, the struct is returned on the stack
    // CHECK-64B-IS-WIN: void @ret_aggregate_i64(ptr{{.*}} {{%.*}}, i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-32B:        void @ret_aggregate_i64(ptr{{.*}} {{%.+}}, i32{{.*}} %a, ptr{{.*}} %b)
    Aggregate { a, b: *b }
}


/* i128 */

#[no_mangle]
pub extern "C" fn pass_i128(a: i128, ret: &mut i128) {
    // i128 is usually passed directly
    // CHECK-NO-WIN: void @pass_i128(i128{{.*}} %a, ptr{{.*}} %ret)

    // On Windows it is passed on the stack. For 32-bit we let LLVM do this.
    // MINGW64: void @pass_i128(ptr{{.*}} %a, ptr{{.*}} %ret)
    // MSVC64: void @pass_i128(ptr{{.*}} %a, ptr{{.*}} %ret)
    // MSVC32: void @pass_i128(i128{{.*}} %a, ptr{{.*}} %ret)
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i128(a: &i128) -> i128 {
    // i128 is usually returned with default handling
    // AARCH64:    i128 @ret_i128(ptr{{.*}} %a)
    // ARM32:      i128 @ret_i128(ptr{{.*}} %a)
    // MSVC32:     i128 @ret_i128(ptr{{.*}} %a)
    // NOSSE32:    i128 @ret_i128(ptr{{.*}} %a)
    // NOSSE64:    i128 @ret_i128(ptr{{.*}} %a)
    // RISCV64:    i128 @ret_i128(ptr{{.*}} %a)
    // SYSVX32:    i128 @ret_i128(ptr{{.*}} %a)
    // SYSVX64:    i128 @ret_i128(ptr{{.*}} %a)
    // WIN64-SOFT: i128 @ret_i128(ptr{{.*}} %a)

    // On Windows with SSE, i128 is returned in xmm0 if available
    // MINGW64:    <16 x i8> @ret_i128(ptr{{.*}} %a)
    // MSVC64:     <16 x i8> @ret_i128(ptr{{.*}} %a)

    // On Wasm, i128 is returned indirectly
    // WASM32:     void @ret_i128(ptr{{.*}} {{%.*}}, ptr{{.*}} %a)
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_i128(a: Aggregate<i128>, dst: &mut i128)  {
    // On most targets this is passed indirectly
    // AARCH64:    void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // MINGW64:    void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // MSVC32:     void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // MSVC64:     void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // NOSSE32:    void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // NOSSE64:    void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // RISCV64:    void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // SYSVX32:    void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // SYSVX64:    void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // WASM32:     void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)
    // WIN64-SOFT: void @pass_aggregate_i128(ptr{{.*}} {{%.+}}, ptr{{.*}} %dst)

    // Except arm32, where this appears as an array to return in registers.
    // ARM32:      void @pass_aggregate_i128([3 x i64] {{%.+}}, ptr{{.*}} %dst)
    *dst = a.b
}


#[no_mangle]
pub extern "C" fn ret_aggregate_i128(a: i32, b: &i128) -> Aggregate<i128> {
    // This type is always returned on the stack
    // CHECK: void @ret_aggregate_i128(ptr{{.*}} {{%.+}}, i32{{.*}} %a, ptr{{.*}} %b)
    Aggregate { a, b: *b }
}

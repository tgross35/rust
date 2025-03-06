//! Verify that Rust implements the expected calling convention for `i128`/`u128`.

// Eliminate intermediate instructions during `nop` tests
//@ compile-flags: -Copt-level=1

//@ add-core-stubs
//@ revisions: MSVC MINGW WIN-SOFT

//@ [MSVC] needs-llvm-components: x86
//@ [MSVC] compile-flags: --target x86_64-pc-windows-msvc
//@ [MINGW] needs-llvm-components: x86
//@ [MINGW] compile-flags: --target x86_64-pc-windows-gnu
//@ [WIN-SOFT] needs-llvm-components: x86
//@ [WIN-SOFT] compile-flags: --target x86_64-unknown-uefi

// Use `WIN` as a common prefix for MSVC and MINGW but *not* the softfloat test.
//@ [MSVC] filecheck-flags: --check-prefixes WIN
//@ [MINGW] filecheck-flags: --check-prefixes WIN

// The `x86_64-unknown-uefi` target also uses the Windows calling convention,
// but does not have SSE registers available.
//@ [WIN-SOFT] filecheck-flags: --check-prefixes WIN

#![crate_type = "lib"]
#![no_std]
#![no_core]
#![feature(no_core, lang_items)]

extern crate minicore;

/* i8 */

// Check that we use the correct pass ABI
#[no_mangle]
pub extern "C" fn pass_i8(a: i8, ret: &mut i8) {
    *ret = arg1
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i8(a: &i8) -> i8 {
    *a
}

#[repr(C)]
struct RetAggregate8 {
    a: i32,
    b: i8,
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i8(arg1: &i8) -> RetAggregate8 {
    RetAggregate { a: 1, b: *arg1 }
}


/* i16 */

// Check that we use the correct pass ABI
#[no_mangle]
pub extern "C" fn pass_i16(a: i16, ret: &mut i16) {
    *ret = arg1
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i16(a: &i16) -> i16 {
    *a
}

#[repr(C)]
struct RetAggregate16 {
    a: i32,
    b: i16,
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i16(arg1: &i16) -> RetAggregate16 {
    RetAggregate { a: 1, b: *arg1 }
}

/* i32 */

// Check that we use the correct pass ABI
#[no_mangle]
pub extern "C" fn pass_i32(a: i32, ret: &mut i32) {
    *ret = arg1
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i32(a: &i32) -> i32 {
    *a
}

#[repr(C)]
struct RetAggregate32 {
    a: i32,
    b: i32,
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i32(arg1: &i32) -> RetAggregate32 {
    RetAggregate { a: 1, b: *arg1 }
}

/* i64 */

// Check that we use the correct pass ABI
#[no_mangle]
pub extern "C" fn pass_i64(a: i64, ret: &mut i64) {
    *ret = arg1
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i64(a: &i64) -> i64 {
    *a
}

#[repr(C)]
struct RetAggregate64 {
    a: i32,
    b: i64,
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i64(arg1: &i64) -> RetAggregate64 {
    RetAggregate { a: 1, b: *arg1 }
}

/* i128 */

// Check that we use the correct pass ABI
#[no_mangle]
pub extern "C" fn pass_i128(a: i128, ret: &mut i128) {
    // WIN-LABEL: @pass_i128()
    
    // CHECK-LABEL: @pass_i128(
    // i128 is passed indirectly on Windows. It should load the pointer to the stack and pass
    // a pointer to that allocation. The softfloat ABI works the same.
    // CHECK-SAME: %_arg0, ptr{{.*}} %arg1)
    // CHECK: [[PASS:%[_0-9]+]] = alloca [16 x i8], align 16
    // CHECK: [[LOADED:%[_0-9]+]] = load i128, ptr %arg1
    // CHECK: store i128 [[LOADED]], ptr [[PASS]]
    // CHECK: call void @extern_call
    *ret = arg1
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i128(a: &i128) -> i128 {
    // WIN-LABEL: @ret_i128(
    // i128 is returned in xmm0 on Windows
    // FIXME(#134288): This may change for the `-msvc` targets in the future.
    // WIN-SAME: i32{{.*}} %_arg0, ptr{{.*}} %arg1)
    // WIN: [[LOADED:%[_0-9]+]] = load <16 x i8>, ptr %arg1
    // WIN-NEXT: ret <16 x i8> [[LOADED]]
    // The softfloat ABI returns this indirectly.
    // softfloat-LABEL: i128 @ret(i32{{.*}} %_arg0, ptr{{.*}} %arg1)
    *a
}

#[repr(C)]
struct RetAggregate {
    a: i32,
    b: i128,
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i128(_arg0: u32, arg1: i128) -> RetAggregate {
    // CHECK-LABEL: @ret_aggregate_i128(
    // Aggregates should also be returned indirectly
    // CHECK-SAME: ptr{{.*}}sret([32 x i8]){{.*}}[[RET:%[_0-9]+]], i32{{.*}}%_arg0, ptr{{.*}}%arg1)
    // CHECK: [[LOADED:%[_0-9]+]] = load i128, ptr %arg1
    // CHECK: [[GEP:%[_0-9]+]] = getelementptr{{.*}}, ptr [[RET]]
    // CHECK: store i128 [[LOADED]], ptr [[GEP]]
    // CHECK: ret void
    RetAggregate { a: 1, b: arg1 }
}

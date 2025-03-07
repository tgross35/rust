//! Verify that Rust implements the expected calling convention for `i128`/`u128`.

// Eliminate intermediate instructions during `nop` tests
//@ compile-flags: -Copt-level=1

//@ add-core-stubs
//@ revisions: AARCH64 ARM32 MSVC64 MINGW64 WASM32 WIN64-SOFT X64 X32
// revisions: MSVC64 MINGW64 WIN64-SOFT MSVC32 X64 X32 AARCH64 ARM32 WASM32

//@ [AARCH64] needs-llvm-components: aarch64
//@ [AARCH64] compile-flags: --target aarch64-unknown-linux-gnu
//@ [ARM32] needs-llvm-components: arm
//@ [ARM32] compile-flags: --target armv7-unknown-linux-gnueabihf
//@ [MSVC64] needs-llvm-components: x86
//@ [MSVC64] compile-flags: --target x86_64-pc-windows-msvc
//@ [MINGW64] needs-llvm-components: x86
//@ [MINGW64] compile-flags: --target x86_64-pc-windows-gnu
//@ [MSVC32] needs-llvm-components: x86
//@ [MSVC32] compile-flags: --target i686-pc-windows-msvc
//@ [WASM32] needs-llvm-components: webassembly
//@ [WASM32] compile-flags: --target wasm32-wasip1
// UEFI uses the windows calling convention without SSE
//@ [WIN64-SOFT] needs-llvm-components: x86
//@ [WIN64-SOFT] compile-flags: --target x86_64-unknown-uefi
//@ [LINUX64] needs-llvm-components: x86
//@ [LINUX64] compile-flags: --target x86_64-unknown-linux-gnu
//@ [LINUX32] needs-llvm-components: x86
//@ [LINUX32] compile-flags: --target i686-unknown-linux-gnu

// We have the prefixes:
// * CHECK-ALL for everything
// * CHECK-64B
// * CHECK-32B
// * WINX64 for windows x86_64
// * AARCH64

// Use `WIN` as a common prefix for MSVC and MINGW but *not* the softfloat test.
//@ [AARCH64] filecheck-flags: --check-prefixes CHECK-64B
//@ [ARM32] filecheck-flags: --check-prefixes CHECK-32B
//@ [MINGW64] filecheck-flags: --check-prefixes CHECK-64B
//@ [MSVC32] filecheck-flags: --check-prefixes CHECK-32B,CHECK-32B-NOARM
//@ [MSVC64] filecheck-flags: --check-prefixes CHECK-64B
//@ [WASM32] filecheck-flags: --check-prefixes CHECK-32B,CHECK-32B-NOARM
//@ [WIN64-SOFT] filecheck-flags: --check-prefixes CHECK-64B
//@ [LINUX32] filecheck-flags: --check-prefixes CHECK-32B,CHECK-32B-NOARM
//@ [LINUX64] filecheck-flags: --check-prefixes CHECK-64B

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
    // CHECK-LABEL: void @pass_i8(i8{{.*}} %a, ptr{{.*}}%ret)
    // CHECK:       store i8 %a, ptr %ret
    // CHECK-NEXT:  ret void
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i8(a: &i8) -> i8 {
    // i8 is always returned directly
    // CHECK-LABEL: i8 @ret_i8(ptr{{.*}} %a)
    // CHECK:       [[LOADED:%.+]] = load i8, ptr %a
    // CHECK-NEXT:  ret i8 [[LOADED]]
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_i8(a: Aggregate<i8>, dst: &mut i8)  {
    // 64bit upsizes the struct to the next integer
    // CHECK-64B-LABEL: void @pass_aggregate_i8(i64{{.*}} %0, ptr{{.*}} %dst)
    // CHECK-64B:       [[SHIFT:%.+]] = lshr i64 %0, 32
    // CHECK-64B-NEXT:  [[TRUNC:%.+]] = trunc i64 [[SHIFT]] to i8
    // CHECK-64B-NEXT:  store i8 [[TRUNC]], ptr %dst
    // CHECK-64B-NEXT:  ret void

    // 32bit passes indirectly
    // CHECK-32B-NOARM-LABEL: void @pass_aggregate_i8(ptr{{.*}} %a, ptr{{.*}} %dst)
    // CHECK-32B-NOARM:       [[GEP:%.+]] = getelementptr{{.*}} i8, ptr %a, i32 4
    // CHECK-32B-NOARM-NEXT:  [[LOAD:%.+]] = load i8, ptr [[GEP]]
    // CHECK-32B-NOARM-NEXT:  store i8 [[LOAD]], ptr %dst
    // CHECK-32B-NOARM-NEXT:  ret void

    // arm32 extends the i8 to an i32 and passes as an array (happens to rename `a`)
    // ARM32-LABEL: void @pass_aggregate_i8(
    // ARM32-SAME:      [2 x i32] [[A:%.+]], ptr{{.*}} %dst)
    // ARM32:       [[EXTRACT:%.+]] = extractvalue [2 x i32] [[A]], 1
    // ARM32-NEXT:  [[TRUNC:%.+]] = trunc i32 [[EXTRACT]] to i8
    // ARM32-NEXT:  store i8 [[TRUNC]], ptr %dst
    // ARM32-NEXT:  ret void
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i8(a: i32, b: &i8) -> Aggregate<i8> {
    // 64bit upsizes the struct to the next integer
    // CHECK-64B-LABEL: i64 @ret_aggregate_i8(i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-64B:       [[LOAD:%.+]] = load i8, ptr %b
    // CHECK-64B-NEXT:  [[EXT:%.+]] = zext i8 [[LOAD]] to i64
    // CHECK-64B-NEXT:  [[SHIFT:%.+]] = shl{{.*}} i64 [[EXT]], 32
    // CHECK-64B-NEXT:  [[EXT2:%.+]] = zext i32 %a to i64
    // CHECK-64B-NEXT:  [[INSERT:%.+]] = or disjoint i64 [[SHIFT]], [[EXT2]]
    // CHECK-64B-NEXT:  ret i64 [[INSERT]]

    // 32bit returns indirectly
    // CHECK-32B-LABEL: void @ret_aggregate_i8(
    // CHECK-32B-SAME:      ptr{{.*}} [[RET:%.+]], i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-32B:       [[LOAD:%.+]] = load i8, ptr %b
    // CHECK-32B-NEXT:  store i32 %a, ptr [[RET]]
    // CHECK-32B:       [[GEP:%.+]] = getelementptr{{.*}} i8, ptr [[RET]], i32 4
    // CHECK-32B-NEXT:  store i8 [[LOAD]], ptr [[GEP]]
    // CHECK-32B:       ret void
    Aggregate { a, b: *b }
}


/* i16 */

#[no_mangle]
pub extern "C" fn pass_i16(a: i16, ret: &mut i16) {
    // i16 is always passed directly
    // CHECK-LABEL: void @pass_i16(i16{{.*}} %a, ptr{{.*}}%ret)
    // CHECK:       store i16 %a, ptr %ret
    // CHECK-NEXT:  ret void
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i16(a: &i16) -> i16 {
    // i16 is always returned directly
    // CHECK-LABEL: i16 @ret_i16(ptr{{.*}} %a)
    // CHECK:       [[LOADED:%.+]] = load i16, ptr %a
    // CHECK-NEXT:  ret i16 [[LOADED]]
    *a
}

//! Verify that Rust implements the expected calling convention for `i128`/`u128`.

// Eliminate intermediate instructions during `nop` tests
//@ compile-flags: -Copt-level=1

//@ add-core-stubs
//@ revisions: AARCH64 ARM32 MSVC64 MINGW64 WASM32 WIN64-SOFT LINUX64 LINUX32
// revisions: MSVC32 

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
//@ [AARCH64] filecheck-flags: --check-prefixes CHECK-64B,CHECK-64B-NOWIN,CHECK-NOWIN
//@ [ARM32] filecheck-flags: --check-prefixes CHECK-32B,CHECK-NOWIN
//@ [MINGW64] filecheck-flags: --check-prefixes CHECK-64B,CHECK-64B-WIN,CHECK-WIN
//@ [MSVC32] filecheck-flags: --check-prefixes CHECK-32B,CHECK-32B-NOARM,CHECK-WIN
//@ [MSVC64] filecheck-flags: --check-prefixes CHECK-64B,CHECK-64B-WIN,CHECK-WIN
//@ [WASM32] filecheck-flags: --check-prefixes CHECK-32B,CHECK-32B-NOARM,CHECK-NOWIN
//@ [WIN64-SOFT] filecheck-flags: --check-prefixes CHECK-64B,CHECK-64B-WIN,CHECK-WIN
//@ [LINUX32] filecheck-flags: --check-prefixes CHECK-32B,CHECK-32B-NOARM,CHECK-NOWIN
//@ [LINUX64] filecheck-flags: --check-prefixes CHECK-64B,CHECK-64B-NOWIN,CHECK-NOWIN

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
    // CHECK-32B:       store i32 %a, ptr [[RET]]
    // CHECK-32B:       [[GEP:%.+]] = getelementptr{{.*}} i8, ptr [[RET]], i32 4
    // CHECK-32B:       store i8 [[LOAD]], ptr [[GEP]]
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

// NB: most arches use an i16 gep but some seem to use i8

#[no_mangle]
pub extern "C" fn pass_aggregate_i16(a: Aggregate<i16>, dst: &mut i16)  {
    // 64bit upsizes the struct to the next integer
    // CHECK-64B-LABEL: void @pass_aggregate_i16(i64{{.*}} %0, ptr{{.*}} %dst)
    // CHECK-64B:       [[SHIFT:%.+]] = lshr i64 %0, 32
    // CHECK-64B-NEXT:  [[TRUNC:%.+]] = trunc i64 [[SHIFT]] to i16
    // CHECK-64B-NEXT:  store i16 [[TRUNC]], ptr %dst
    // CHECK-64B-NEXT:  ret void

    // 32bit passes indirectly
    // CHECK-32B-NOARM-LABEL: void @pass_aggregate_i16(ptr{{.*}} %a, ptr{{.*}} %dst)
    // CHECK-32B-NOARM:       [[GEP:%.+]] = getelementptr{{.*}} {{(i8|i16)}}, ptr %a, i32 4
    // CHECK-32B-NOARM-NEXT:  [[LOAD:%.+]] = load i16, ptr [[GEP]]
    // CHECK-32B-NOARM-NEXT:  store i16 [[LOAD]], ptr %dst
    // CHECK-32B-NOARM-NEXT:  ret void

    // arm32 extends the i16 to an i32 and passes as an array (happens to rename `a`)
    // ARM32-LABEL: void @pass_aggregate_i16(
    // ARM32-SAME:      [2 x i32] [[A:%.+]], ptr{{.*}} %dst)
    // ARM32:       [[EXTRACT:%.+]] = extractvalue [2 x i32] [[A]], 1
    // ARM32-NEXT:  [[TRUNC:%.+]] = trunc i32 [[EXTRACT]] to i16
    // ARM32-NEXT:  store i16 [[TRUNC]], ptr %dst
    // ARM32-NEXT:  ret void
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i16(a: i32, b: &i16) -> Aggregate<i16> {
    // 64bit upsizes the struct to the next integer
    // CHECK-64B-LABEL: i64 @ret_aggregate_i16(i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-64B:       [[LOAD:%.+]] = load i16, ptr %b
    // CHECK-64B-NEXT:  [[EXT:%.+]] = zext i16 [[LOAD]] to i64
    // CHECK-64B-NEXT:  [[SHIFT:%.+]] = shl{{.*}} i64 [[EXT]], 32
    // CHECK-64B-NEXT:  [[EXT2:%.+]] = zext i32 %a to i64
    // CHECK-64B-NEXT:  [[INSERT:%.+]] = or disjoint i64 [[SHIFT]], [[EXT2]]
    // CHECK-64B-NEXT:  ret i64 [[INSERT]]

    // 32bit returns indirectly
    // CHECK-32B-LABEL: void @ret_aggregate_i16(
    // CHECK-32B-SAME:      ptr{{.*}} [[RET:%.+]], i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-32B:       [[LOAD:%.+]] = load i16, ptr %b
    // CHECK-32B:       store i32 %a, ptr [[RET]]
    // CHECK-32B:       [[GEP:%.+]] = getelementptr{{.*}} {{(i8|i16)}}, ptr [[RET]], i32 4
    // CHECK-32B:       store i16 [[LOAD]], ptr [[GEP]]
    // CHECK-32B:       ret void
    Aggregate { a, b: *b }
}


/* i32 */

#[no_mangle]
pub extern "C" fn pass_i32(a: i32, ret: &mut i32) {
    // i32 is always passed directly
    // CHECK-LABEL: void @pass_i32(i32{{.*}} %a, ptr{{.*}}%ret)
    // CHECK:       store i32 %a, ptr %ret
    // CHECK-NEXT:  ret void
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i32(a: &i32) -> i32 {
    // i32 is always returned directly
    // CHECK-LABEL: i32 @ret_i32(ptr{{.*}} %a)
    // CHECK:       [[LOADED:%.+]] = load i32, ptr %a
    // CHECK-NEXT:  ret i32 [[LOADED]]
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_i32(a: Aggregate<i32>, dst: &mut i32)  {
    // 64bit upsizes the struct to the next integer
    // CHECK-64B-LABEL: void @pass_aggregate_i32(i64{{.*}} %0, ptr{{.*}} %dst)
    // CHECK-64B:       [[SHIFT:%.+]] = lshr i64 %0, 32
    // CHECK-64B-NEXT:  [[TRUNC:%.+]] = trunc{{.*}} i64 [[SHIFT]] to i32
    // CHECK-64B-NEXT:  store i32 [[TRUNC]], ptr %dst
    // CHECK-64B-NEXT:  ret void

    // 32bit passes indirectly
    // CHECK-32B-NOARM-LABEL: void @pass_aggregate_i32(ptr{{.*}} %a, ptr{{.*}} %dst)
    // most arches use an i32 gep, wasm uses i8 for some reason
    // CHECK-32B-NOARM:       [[GEP:%.+]] = getelementptr{{.*}} {{(i8|i32)}}, ptr %a, i32 4
    // CHECK-32B-NOARM-NEXT:  [[LOAD:%.+]] = load i32, ptr [[GEP]]
    // CHECK-32B-NOARM-NEXT:  store i32 [[LOAD]], ptr %dst
    // CHECK-32B-NOARM-NEXT:  ret void

    // arm32 extends the passes as an array (happens to rename `a`)
    // ARM32-LABEL: void @pass_aggregate_i32(
    // ARM32-SAME:      [2 x i32] [[A:%.+]], ptr{{.*}} %dst)
    // ARM32:       [[EXTRACT:%.+]] = extractvalue [2 x i32] [[A]], 1
    // ARM32-NEXT:  store i32 [[EXTRACT]], ptr %dst
    // ARM32-NEXT:  ret void
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i32(a: i32, b: &i32) -> Aggregate<i32> {
    // 64bit upsizes the struct to the next integer
    // CHECK-64B-LABEL: i64 @ret_aggregate_i32(i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-64B:       [[LOAD:%.+]] = load i32, ptr %b
    // CHECK-64B-NEXT:  [[EXT:%.+]] = zext i32 [[LOAD]] to i64
    // CHECK-64B-NEXT:  [[SHIFT:%.+]] = shl{{.*}} i64 [[EXT]], 32
    // CHECK-64B-NEXT:  [[EXT2:%.+]] = zext i32 %a to i64
    // CHECK-64B-NEXT:  [[INSERT:%.+]] = or disjoint i64 [[SHIFT]], [[EXT2]]
    // CHECK-64B-NEXT:  ret i64 [[INSERT]]

    // 32bit returns indirectly
    // CHECK-32B-LABEL: void @ret_aggregate_i32(
    // CHECK-32B-SAME:      ptr{{.*}} [[RET:%.+]], i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-32B:       [[LOAD:%.+]] = load i32, ptr %b
    // CHECK-32B:       store i32 %a, ptr [[RET]]
    // CHECK-32B:       [[GEP:%.+]] = getelementptr{{.*}} {{(i8|i32)}}, ptr [[RET]], i32 4
    // CHECK-32B:       store i32 [[LOAD]], ptr [[GEP]]
    // CHECK-32B:       ret void
    Aggregate { a, b: *b }
}

/* i64 */

#[no_mangle]
pub extern "C" fn pass_i64(a: i64, ret: &mut i64) {
    // i64 is always passed directly
    // CHECK-LABEL: void @pass_i64(i64{{.*}} %a, ptr{{.*}}%ret)
    // CHECK:       store i64 %a, ptr %ret
    // CHECK-NEXT:  ret void
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i64(a: &i64) -> i64 {
    // i64 is always returned directly
    // CHECK-LABEL: i64 @ret_i64(ptr{{.*}} %a)
    // CHECK:       [[LOADED:%.+]] = load i64, ptr %a
    // CHECK-NEXT:  ret i64 [[LOADED]]
    *a
}

#[no_mangle]
pub extern "C" fn pass_aggregate_i64(a: Aggregate<i64>, dst: &mut i64)  {
    // 64bit passes as a scalar pair { i64, i64 } or array [2 x i64]
    // CHECK-64B-NOWIN-LABEL: void @pass_aggregate_i64(
    // CHECK-64B-NOWIN-SAME:      [[TY:(\{ i64, i64 \}|\[2 x i64\])]]{{.*}} %0, ptr{{.*}} %dst)
    // CHECK-64B-NOWIN:       [[EXTRACT:%.+]] = extractvalue [[TY]] %0, 1
    // CHECK-64B-NOWIN-NEXT:  store i64 [[EXTRACT]], ptr %dst
    // CHECK-64B-NOWIN-NEXT:  ret void

    // Windows passes indirect
    // CHECK-64B-WIN-LABEL: void @pass_aggregate_i64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // CHECK-64B-WIN:       [[GEP:%.+]] = getelementptr{{.*}} i8, ptr %a, i64 8
    // CHECK-64B-WIN-NEXT:  [[LOAD:%.+]] = load i64, ptr [[GEP]]
    // CHECK-64B-WIN-NEXT:  store i64 [[LOAD]], ptr %dst
    // CHECK-64B-WIN-NEXT:  ret void

    // 32bit passes indirect
    // CHECK-32B-NOARM-LABEL: void @pass_aggregate_i64(ptr{{.*}} %a, ptr{{.*}} %dst)
    // CHECK-32B-NOARM:       [[GEP:%.+]] = getelementptr{{.*}} i8, ptr %a, {{(i32 4|i32 8|i64 4)}}
    // CHECK-32B-NOARM-NEXT:  [[LOAD:%.+]] = load i64, ptr [[GEP]]
    // CHECK-32B-NOARM-NEXT:  store i64 [[LOAD]], ptr %dst
    // CHECK-32B-NOARM-NEXT:  ret void

    // arm32 passes as an array
    // ARM32-LABEL: void @pass_aggregate_i64(
    // ARM32-SAME:  [2 x i64] [[A:%.+]], ptr{{.*}} %dst)
    // ARM32:       [[EXTRACT:%.+]] = extractvalue [2 x i64] [[A]], 1
    // ARM32-NEXT:  store i64 [[EXTRACT]], ptr %dst
    // ARM32-NEXT:  ret void
    *dst = a.b
}

#[no_mangle]
pub extern "C" fn ret_aggregate_i64(a: i32, b: &i64) -> Aggregate<i64> {
    // 64bit 
    // CHECK-64B-NOWIN-LABEL: {{(\{ i64, i64 \}|\[2 x i64\])}} @ret_aggregate_i64(i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-64B-NOWIN:       [[LOAD:%.+]] = load i64, ptr %b
    // CHECK-64B-NOWIN-NEXT:  [[EXT:%.+]] = zext i32 %a to i64
    // CHECK-64B-NOWIN-NEXT:  [[INSERT0:%.+]] = insertvalue [[TY:(\{ i64, i64 \}|\[2 x i64\])]] poison, i64 [[EXT]], 0
    // CHECK-64B-NOWIN-NEXT:  [[INSERT1:%.+]] = insertvalue [[TY]] [[INSERT0]], i64 [[LOAD]], 1
    // CHECK-64B-NOWIN-NEXT:  ret [[TY]] [[INSERT1]]

    // Windows passes indirect
    // CHECK-64B-WIN-LABEL: void @ret_aggregate_i64(
    // CHECK-64B-WIN-SAME:      ptr{{.*}} [[RET:%.+]], i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-64B-WIN:       [[LOAD:%.+]] = load i64, ptr %b
    // CHECK-64B-WIN:       store i32 %a, ptr [[RET]]
    // CHECK-64B-WIN:       [[GEP:%.+]] = getelementptr{{.*}} i8, ptr [[RET]], i64 8
    // CHECK-64B-WIN:       store i64 [[LOAD]], ptr [[GEP]]
    // CHECK-64B-WIN:       ret void

    // 32bit returns indirectly
    // CHECK-32B-LABEL: void @ret_aggregate_i64(
    // CHECK-32B-SAME:      ptr{{.*}} [[RET:%.+]], i32{{.*}} %a, ptr{{.*}} %b)
    // CHECK-32B:       [[LOAD:%.+]] = load i64, ptr %b
    // CHECK-32B:       store i32 %a, ptr [[RET]]
    // CHECK-32B:       [[GEP:%.+]] = getelementptr{{.*}} i8, ptr [[RET]], {{(i32 4|i32 8|i64 8)}}
    // CHECK-32B:       store i64 [[LOAD]], ptr [[GEP]]
    // CHECK-32B:       ret void
    Aggregate { a, b: *b }
}


/* i128 */

#[no_mangle]
pub extern "C" fn pass_i128(a: i128, ret: &mut i128) {
    // i128 is usually passed directly
    // CHECK-NOWIN-LABEL: void @pass_i128(i128{{.*}} %a, ptr{{.*}}%ret)
    // CHECK-NOWIN:       store i128 %a, ptr %ret
    // CHECK-NOWIN-NEXT:  ret void

    // on Windows it is passed on the stack
    // CHECK-WIN-LABEL: void @pass_i128(ptr{{.*}} %a, ptr{{.*}}%ret)
    // CHECK-WIN:       [[LOAD:%.+]] = load i128, ptr %a
    // CHECK-WIN-NEXT:  store i128 [[LOAD]], ptr %ret
    // CHECK-WIN-NEXT:  ret void
    *ret = a
}

// Check that we produce the correct return ABI
#[no_mangle]
pub extern "C" fn ret_i128(a: &i128) -> i128 {
    // i128 is always returned directly
    // COM: CHECK-LABEL: i128 @ret_i128(ptr{{.*}} %a)
    // COM: CHECK:       [[LOADED:%.+]] = load i128, ptr %a
    // COM: CHECK-NEXT:  ret i128 [[LOADED]]
    //
    // CHECK-WIN: <16 x i8> @ret_i128(ptr{{.*}} %a)
    // WIN:       [[LOAD:%.+]] = load <16 x i8>, ptr %a
    // WIN-NEXT:  ret <16 x i8> [[LOAD]]
    *a
}

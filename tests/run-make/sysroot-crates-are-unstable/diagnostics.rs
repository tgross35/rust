#![crate_type = "lib"]

// use evil_sysroot_module;
// use leb128;
trait Trait { }
type Foo = dyn Trait<Buf = i32>;

// trait Trait {}
// type Foo = dyn Trait<Muahaha= i32>;


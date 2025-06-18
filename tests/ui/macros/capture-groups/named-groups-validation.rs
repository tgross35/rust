#![crate_type = "lib"]
#![allow(unused_macros)]

macro_rules! repeated_binding {
    ($a($a:ident)*) => {};
    //~^ ERROR: duplicate matcher binding
    ($a(foo)* $a(bar)*) => {};
    //~^ ERROR: duplicate matcher binding
}


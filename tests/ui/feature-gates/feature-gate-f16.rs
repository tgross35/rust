#![allow(unused)]

// ensure gating for primitive use
mod m1 {
    const A: f16 = 10.0; //~ ERROR the feature `f16` is unstable

    pub fn main() {
        let a: f16 = 100.0; //~ ERROR the feature `f16` is unstable
        let b = 0.0f16; //~ ERROR the feature `f16` is unstable
        foo(1.23);
    }

    fn foo(a: f16) {} //~ ERROR the feature `f16` is unstable

    struct Bar {
        a: f16, //~ ERROR the feature `f16` is unstable
    }
}

// ensure we don't restrict name as an identifier or custom types
mod m2 {
    #[allow(non_camel_case_types)]
    struct f16 {
        a: i32,
    }

    fn foo() {
        let f16 = ();
    }

    fn bar(a: f16) -> i32 {
        a.a
    }
}

fn main() {}

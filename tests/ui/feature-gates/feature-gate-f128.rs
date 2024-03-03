#![allow(unused)]

// ensure gating for primitive use
mod m1 {
    const A: f128 = 10.0; //~ ERROR the feature `f128` is unstable

    pub fn main() {
        let a: f128 = 100.0; //~ ERROR the feature `f128` is unstable
        let b = 0.0f128; //~ ERROR the feature `f128` is unstable
        foo(1.23);
    }

    fn foo(a: f128) {} //~ ERROR the feature `f128` is unstable

    struct Bar {
        a: f128, //~ ERROR the feature `f128` is unstable
    }
}

// ensure we don't restrict name as an identifier or custom types
mod m2 {
    #[allow(non_camel_case_types)]
    struct f128 {
        a: i32,
    }

    fn foo() {
        let f128 = ();
    }

    fn bar(a: f128) -> i32 {
        a.a
    }
}

fn main() {}

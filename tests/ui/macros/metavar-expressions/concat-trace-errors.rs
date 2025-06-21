// Our diagnostics should be able to point to a specific input that caused an invalid
// identifier.
//
// NOTEs are checked to makesure we point to metavariables.

#![feature(macro_metavar_expr_concat)]

// See what we can do without expanding anything
macro_rules! pre_expansion {
    ($a:ident) => {
        ${concat("hi", " bye ")};
        //~^ ERROR invalid item within a `${concat(...)}` expression
        //~| NOTE invalid identifier
        ${concat("hi", "-", "bye")};
        //~^ ERROR invalid item within a `${concat(...)}` expression
        //~| NOTE invalid identifier
        ${concat($a, "-")};
        //~^ ERROR invalid item within a `${concat(...)}` expression
        //~| NOTE invalid identifier
    }
}

macro_rules! post_expansion {
    ($a:literal) => {
        const _: () = ${concat("hi", $a, "bye")};
        //~^ ERROR `${concat(..)}` constructed an invalid identifier
        //~| NOTE expanding this metavariable
    }
}

post_expansion!("!");
//~^ NOTE in this expansion of post_expansion!

macro_rules! post_expansion_many {
    ($a:ident, $b:ident, $c:ident, $d:literal, $e:ident) => {
        const _: () = ${concat($a, $b, $c, $d, $e)};
        //~^ ERROR `${concat(..)}` constructed an invalid identifier
        //~| NOTE expanding this metavariable
    }
}

post_expansion_many!(a, b, c, ".d", e);
//~^ NOTE in this expansion of post_expansion_many!

fn main() {}

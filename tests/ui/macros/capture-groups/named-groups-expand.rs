//@ run-pass

macro_rules! basic {
    ( $group1( $a:ident ),+ ) => {
        // todo: add expansion
        stringify!()
        // $group1( println!("{}", $a); )+
    }
}

macro_rules! no_captures {
    ( $g( const )? ) => {
        stringify!()
    }
}

fn main() {
    assert_eq!(basic!(a, b), "");
    assert_eq!(no_captures!(const), "");
}

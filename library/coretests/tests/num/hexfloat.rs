use core::num::imp::Float;
use core::num::{HexFloatErrorKind, ParseHexFloatError};

#[test]
#[cfg_attr(miri, ignore)] // This test is very slow when using Miri
fn test_parse_any() {
    for k in -149..=127 {
        let s = format!("0x1p{k}");
        let x = f32::from_hex(&s).unwrap();
        let y = if k < 0 { 0.5f32.powi(-k) } else { 2.0f32.powi(k) };
        assert_eq!(x, y);
    }

    let mut s = *b"0x.0000000p-121";
    for e in 0..40 {
        for k in 0..(1 << 15) {
            let expected = f32::from_bits(k) * 2.0f32.powi(e);
            let x = f32::from_hex(&s).unwrap();
            assert_eq!(
                x.to_bits(),
                expected.to_bits(),
                "\
                e={e}\n\
                k={k}\n\
                x={x}\n\
                expected={expected}\n\
                s={}\n\
                f32::from_bits(k)={}\n\
                2.0f32.powi(e)={}\
                ",
                str::from_utf8(&s).unwrap(),
                f32::from_bits(k),
                2.0f32.powi(e),
            );
            for i in (3..10).rev() {
                if s[i] == b'f' {
                    s[i] = b'0';
                } else if s[i] == b'9' {
                    s[i] = b'a';
                    break;
                } else {
                    s[i] += 1;
                    break;
                }
            }
        }
        for i in (12..15).rev() {
            if s[i] == b'0' {
                s[i] = b'9';
            } else {
                s[i] -= 1;
                break;
            }
        }
        for i in (3..10).rev() {
            s[i] = b'0';
        }
    }
}

// #[test]
// #[cfg_attr(miri, ignore)] // This test is very slow when using Miri
// fn test_long_tail() {
//     for k in 1..1000 {
//         let s = format!("0x1.{}p0", "0".repeat(k));
//         let Ok(bits) = parse_hex_exact(&s, 32, 23) else {
//             panic!("parsing {s} failed")
//         };
//         assert_eq!(f32::from_bits(bits as u32), 1.0);

//         let s = format!("0x1.{}1p0", "0".repeat(k));
//         let Ok((bits, status)) = parse_any(&s, 32, 23, Round::Nearest) else {
//             unreachable!()
//         };
//         if status.inexact() {
//             assert!(1.0 == f32::from_bits(bits as u32));
//         } else {
//             assert!(1.0 < f32::from_bits(bits as u32));
//         }
//     }
// }

struct Inputs {
    max: &'static str,
    min_pos_normal: &'static str,
    min_pos_subnormal: &'static str,
    /// Exact maximum precision allowed
    max_prec: &'static str,
    /// One bit more than the above
    extra_prec: &'static str,
    min_overflow: &'static str,
}

/// Default test cases.
macro_rules! checks {
    ($ty:ty, $inputs:expr) => {
        let inputs = $inputs;
        vec![
            // Shifted equivalents
            ("0x.1234p+16", Ok((0x1234 as $ty).to_bits())),
            ("0x1.234p+12", Ok((0x1234 as $ty).to_bits())),
            ("0x12.34p+8", Ok((0x1234 as $ty).to_bits())),
            ("0x123.4p+4", Ok((0x1234 as $ty).to_bits())),
            ("0x1234p+0", Ok((0x1234 as $ty).to_bits())),
            ("0x1234.p+0", Ok((0x1234 as $ty).to_bits())),
            ("0x1234.0p+0", Ok((0x1234 as $ty).to_bits())),
            // Simple values
            ("0x0.0p+0", Ok(<$ty>::to_bits(0.0))),
            ("-0x0.0p+0", Ok(<$ty>::to_bits(-0.0))),
            ("0x1.0p+0", Ok(<$ty>::to_bits(1.0))),
            ("-0x1.0p+0", Ok(<$ty>::to_bits(-1.0))),
            ("0x1.0p+1", Ok(<$ty>::to_bits(2.0))),
            // Limits
            (inputs.max, Ok(<$ty>::MAX.to_bits())),
            (concat!("-", $max), Ok(<$ty>::MIN.to_bits())),
            (inputs.min_pos_normal, Ok(<$ty>::MIN_POSITIVE_NORMAL.to_bits())),
            (inputs.min_pos_subnormal, Ok(<$ty>::MIN_POSITIVE_SUBNORMAL.to_bits())),
            // NaN and infinity with case variations
            ("nan", Ok(<$ty>::NAN.to_bits())),
            ("-nan", Ok((-<$ty>::NAN).to_bits())),
            ("inf", Ok(<$ty>::INFINITY.to_bits())),
            ("-inf", Ok(<$ty>::NEG_INFINITY.to_bits())),
            ("nAN", Ok(<$ty>::NAN.to_bits())),
            ("InF", Ok(<$ty>::INFINITY.to_bits())),
        ]
    };
}

#[test]
#[cfg(target_has_reliable_f16)]
fn test_f16() {
    let mut checks = checks!(
        f16,
        Inputs {
            max: "0x1.ffcp+15",
            min_pos_normal: "0x1p-14",
            min_pos_subnormal: "0x1p-24",
            almost_extra_prec: "0x1.ffcp+0",
            extra_prec: "0x1.ffdp+0",
            min_overflow: "0x1p+16",
        }
    );
    checks.extend([]);
    // let checks = [
    //     ("0x1.ffp+8", 0x5ffc),
    //     ("+0x1.ffp+8", 0x5ffc),
    //     ("0x1.998p-4", 0x2e66),
    //     ("0x1.9p+6", 0x5640),
    //     ("0x1.998p-4", (0.1f16).to_bits()),
    //     ("-0x1.998p-4", (-0.1f16).to_bits()),
    //     ("0x0.123p-12", 0x0123),
    //     ("0x1p-24", 0x0001),
    // ];
    for (s, exp) in checks {
        let act = f16::from_hex(s);
        check_result(s, act, exp);
    }
}

#[test]
fn test_f32() {
    let mut checks = checks!(
        f32,
        Inputs {
            max: "0x1.fffffep+127",
            min_pos_normal: "0x1p-126",
            min_pos_subnormal: "0x1p-149",
        }
    );
    checks.extend([]);
    // let checks = [
    //     ("0x1.ffep+8", 0x43fff000),
    //     ("+0x1.ffep+8", 0x43fff000),
    //     ("0x1.99999ap-4", 0x3dcccccd),
    //     ("0x1.9p+6", 0x42c80000),
    //     ("0x1.2d5ed2p+20", 0x4996af69),
    //     ("-0x1.348eb8p+10", 0xc49a475c),
    //     ("-0x1.33dcfep-33", 0xaf19ee7f),
    //     ("0x1.99999ap-4", (0.1f32).to_bits()),
    //     ("-0x1.99999ap-4", (-0.1f32).to_bits()),
    //     ("0x1.111114p-127", 0x00444445),
    //     ("0x1.23456p-130", 0x00091a2b),
    // ];

    for (s, exp) in checks {
        let act = f32::from_hex(s);
        check_result(s, act, exp);
    }
}

#[test]
fn test_f64() {
    let mut checks = checks!(
        f64,
        Inputs {
            max: "0x1.fffffffffffffp+1023",
            min_pos_normal: "0x1p-1022",
            min_pos_subnormal: "0x1p-1074",
        }
    );
    checks.extend([]);
    // let checks = [
    //     ("0x1.ffep+8", 0x407ffe0000000000),
    //     ("0x1.999999999999ap-4", 0x3fb999999999999a),
    //     ("0x1.9p+6", 0x4059000000000000),
    //     ("0x1.2d5ed1fe1da7bp+20", 0x4132d5ed1fe1da7b),
    //     ("-0x1.348eb851eb852p+10", 0xc09348eb851eb852),
    //     ("-0x1.33dcfe54a3803p-33", 0xbde33dcfe54a3803),
    //     ("0x1.999999999999ap-4", 0.1f64.to_bits()),
    //     ("0x1.999999999998ap-4", (0.1f64 - f64::EPSILON).to_bits()),
    //     ("-0x1.999999999999ap-4", (-0.1f64).to_bits()),
    //     ("-0x1.999999999998ap-4", (-0.1f64 + f64::EPSILON).to_bits()),
    //     ("0x0.8000000000001p-1022", 0x0008000000000001),
    //     ("0x0.123456789abcdp-1022", 0x000123456789abcd),
    //     ("0x0.0000000000002p-1022", 0x0000000000000002),
    // ];
    for (s, exp) in checks {
        let act = f64::from_hex(s);
        check_result(s, act, exp);
    }
}

#[test]
#[cfg(target_has_reliable_f128)]
fn test_f128() {
    let mut checks = checks!(
        f128,
        Inputs {
            max: "0x1.ffffffffffffffffffffffffffffp+16383",
            min_pos_normal: "0x1p-16382",
            min_pos_subnormal: "0x1p-16494",
        }
    );
    checks.extend([]);
    // let checks = [
    //     ("0x1.ffep+8", 0x4007ffe0000000000000000000000000),
    //     ("+0x1.ffep+8", 0x4007ffe0000000000000000000000000),
    //     ("0x1.999999999999999999999999999ap-4", 0x3ffb999999999999999999999999999a),
    //     ("0x1.9p+6", 0x40059000000000000000000000000000),
    //     ("0x1.999999999999999999999999999ap-4", (0.1f128).to_bits()),
    //     ("-0x1.999999999999999999999999999ap-4", (-0.1f128).to_bits()),
    //     ("0x0.abcdef0123456789abcdef012345p-16382", 0x0000abcdef0123456789abcdef012345),
    //     ("0x1p-16494", 0x00000000000000000000000000000001),
    // ];
    for (s, exp) in checks {
        let act = f128::from_hex(s);
        check_result(s, act, exp);
    }
}

#[track_caller]
fn check_result<F: Float>(
    s: &str,
    actual: Result<F, ParseHexFloatError>,
    expected: Result<F::Int, ParseHexFloatError>,
) {
    let act_bits = actual.map(F::to_bits);
    let hwidth = ((F::BITS / 4) + 2) as usize;
    let bwidth = (F::BITS + 2) as usize;
    let fmt_bits = |x: &Result<_, _>| match x {
        Ok(v) => format!("{v:#0hwidth$x} {v:#0bwidth$b} {:?}", F::from_bits(*v)),
        Err(e) => format!("{e:?}"),
    };

    assert_eq!(
        act_bits,
        expected,
        "\
        parsing:  {s}\n\
        actual:   {}\n\
        expected: {}\
        ",
        fmt_bits(&act_bits),
        fmt_bits(&expected),
    );
}

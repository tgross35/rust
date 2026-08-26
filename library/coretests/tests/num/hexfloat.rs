//! Tests for float hex format parsing and printing.

use core::num::imp::Float;
use core::num::imp::hexfloat::parse_any;
use core::num::{HexFloatErrorKind, ParseHexFloatError};
use std::borrow::Cow;
use std::fmt::{self, Write};

#[test]
#[cfg_attr(miri, ignore)] // This test is very slow when using Miri
fn test_many_small() {
    // Check all exponents of `f32`
    for k in -149..=127 {
        let s = format!("0x1p{k}");
        let x = f32::from_hex(&s).unwrap();
        let y = if k < 0 { 0.5f32.powi(-k) } else { 2.0f32.powi(k) };
        assert_eq!(x, y);
    }

    // Sweep a number of possible inputs near the smallest representable f32 values
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

            // Increment the significand
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

        // Increment the exponent and reset the significand
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

#[test]
#[cfg(target_has_reliable_f128_math)] // uses round_ties_even
fn rounding() {
    use std::f128::consts::PI;
    let s = format!("{:x}", Hex(PI));

    for k in 0..=111 {
        let (bits, status) = parse_any(s.as_bytes(), 128 - k, 112 - k).unwrap();
        let scale = (1u128 << (112 - k - 1)) as f128;
        let expected = (PI * scale).round_ties_even() / scale;
        assert_eq!(bits << k, expected.to_bits(), "k = {k}, s = {s}");
        assert_eq!(expected != PI, status.inexact());
    }
}

#[test]
fn rounding_extreme_underflow() {
    for k in 1..1000 {
        let s = format!("0x1p{}", -149 - k);
        let Ok((bits, status)) = parse_any(s.as_bytes(), 32, 23) else { unreachable!() };
        assert_eq!(bits, 0, "{s} should round to zero, got bits={bits}");
        assert!(status.underflow(), "should indicate underflow when parsing {s}");
        assert!(status.inexact(), "should indicate inexact when parsing {s}");
    }
}

/// Many zeros before a trailing 1.
#[test]
#[cfg_attr(miri, ignore)] // This test is very slow when using Miri
fn long_tail() {
    let mut s = String::new();
    for k in 1..1000 {
        // Trailing zeros shouldn't matter.
        s.clear();
        write!(s, "0x1.{}p0", "0".repeat(k)).unwrap();
        let x = f32::from_hex(&s).unwrap();
        assert_eq!(x, 1.0);

        // Trailing zeros followed by a final one.
        s.clear();
        write!(s, "0x1.{}1p0", "0".repeat(k)).unwrap();
        let x = f32::from_hex(&s).unwrap();

        // Check approximate value
        assert!(1.0 <= x);
        assert!(x < 1.5);

        // 0x1.000001p+0 is the first value in this pattern to round to 1.0.
        if k >= 5 {
            assert_eq!(x, 1.0);
        } else {
            assert_ne!(x, 1.0);
        }
    }
}

#[test]
#[cfg(target_has_reliable_f16)]
fn test_f16_roundtrip_exhaustive() {
    let mut s = String::new();

    // Roundtrip via hex formatting
    for xi in 0..=u16::MAX {
        s.clear();
        let x = f16::from_bits(xi);
        write!(s, "{:x}", Hex(x)).unwrap();
        let y = f16::from_hex(&s).unwrap();
        let yi = y.to_bits();
        if x.is_nan() {
            assert!(y.is_nan(), "{xi:#06x} != {yi:#06x} ({x} != {y}, {s})");
        } else {
            assert_eq!(xi, yi, "{xi:#06x} != {yi:#06x} ({x} != {y}, {s})");
        }
    }
}

/// Moo
fn c<T>(val: T) -> Cow<'static, str>
where
    Cow<'static, str>: From<T>,
{
    Cow::from(val)
}

/// Default test cases.
macro_rules! checks {
    ($ty:ty, $inputs:expr) => {{
        let mut v = vec![
            // Shifted equivalents
            (c("0x.1234p+16"), Ok((0x1234 as $ty).to_bits()), false),
            (c("0x1.234p+12"), Ok((0x1234 as $ty).to_bits()), true),
            (c("0x12.34p+8"), Ok((0x1234 as $ty).to_bits()), false),
            (c("0x123.4p+4"), Ok((0x1234 as $ty).to_bits()), false),
            (c("0x1234p+0"), Ok((0x1234 as $ty).to_bits()), false),
            (c("0x1234.p+0"), Ok((0x1234 as $ty).to_bits()), false),
            (c("0x1234.0p+0"), Ok((0x1234 as $ty).to_bits()), false),
            // Simple values
            (c("0x0p+0"), Ok(<$ty>::to_bits(0.0)), true),
            (c("-0x0p+0"), Ok(<$ty>::to_bits(-0.0)), true),
            (c("+0x0p+0"), Ok(<$ty>::to_bits(0.0)), false),
            (c("0x1p+0"), Ok(<$ty>::to_bits(1.0)), true),
            (c("-0x1p+0"), Ok(<$ty>::to_bits(-1.0)), true),
            (c("+0x1p+0"), Ok(<$ty>::to_bits(1.0)), false),
            (c("0x1p+1"), Ok(<$ty>::to_bits(2.0)), true),
            (c("-0x1p+1"), Ok(<$ty>::to_bits(-2.0)), true),
            (c("+0x1p+1"), Ok(<$ty>::to_bits(2.0)), false),
            // Default NaN and infinity
            (c("NaN"), Ok(<$ty>::NAN.to_bits()), true),
            (c("-NaN"), Ok((-<$ty>::NAN).to_bits()), false),
            (c("inf"), Ok(<$ty>::INFINITY.to_bits()), true),
            (c("-inf"), Ok(<$ty>::NEG_INFINITY.to_bits()), true),
            // NaN and infinity with case variations
            (c("nan"), Ok(<$ty>::NAN.to_bits()), false),
            (c("-nan"), Ok((-<$ty>::NAN).to_bits()), false),
            (c("nAN"), Ok(<$ty>::NAN.to_bits()), false),
            (c("InF"), Ok(<$ty>::INFINITY.to_bits()), false),
            // Errors
            (c(""), Err(HexFloatErrorKind::NoHexIndicator), false),
            (c("+"), Err(HexFloatErrorKind::NoHexIndicator), false),
            (c("-"), Err(HexFloatErrorKind::NoHexIndicator), false),
            (c("1234"), Err(HexFloatErrorKind::NoHexIndicator), false),
            (c("0x"), Err(HexFloatErrorKind::EmptySignificand), false),
            (c("0x1p"), Err(HexFloatErrorKind::EmptyExponent), false),
            (c("0x1p+"), Err(HexFloatErrorKind::EmptyExponent), false),
            (c("0x1p-"), Err(HexFloatErrorKind::EmptyExponent), false),
            (c("0x789defg"), Err(HexFloatErrorKind::InvalidSignificand), false),
            (c("0x1p+a"), Err(HexFloatErrorKind::InvalidExponent), false),
        ];

        let inputs: &[(&str, $ty, bool)] = &$inputs;
        for (s, expected, roundtrip) in inputs.iter().copied() {
            v.push((c(s), Ok(expected.to_bits()), roundtrip))
        }

        v
    }};
}

#[test]
#[cfg(target_has_reliable_f16)]
fn test_f16() {
    let checks = checks!(
        f16,
        [
            // Exact, subnormal repr
            ("0x0.004p-14", f16::MIN_POSITIVE_SUBNORMAL, false),
            // Exact, normalized repr
            ("0x1p-24", f16::MIN_POSITIVE_SUBNORMAL, true),
            ("0x1.ffcp+15", f16::MAX, true),
        ] // Inputs {
          //     // round_to_zero:
          //     // round_to_min_pos_subnormal:
          //     // round_to_min_pos_normal:
          //     // round_to_max:
          //     // round_to_inf:

          //     round_to_zero: &[],

          //     round_to_min_pos_subnormal: &["0x1p-24"],
          //     round_to_min_pos_normal: &["0x1p-14"],
          //     round_to_max: &["0x1.ffcp+15"],
          //     round_to_inf: &["0x1.ffcp+15"],

          //     // almost_extra_prec: "0x1.ffcp+0",
          //     // max_prec: "0x1.ffep+0",
          //     // extra_prec: "0x1.ffdp+0",
          //     // min_overflow: "0x1p+16",
          // }
    );

    // let mut push_check = |s, r: f16| checks.push((c(s), Ok(r.to_bits())));

    // push_check("0x1p-24", f16::MIN_POSITIVE_SUBNORMAL);

    // checks.extend([]);
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
    for (i, (s, exp, roundtrip)) in checks.iter().enumerate() {
        let s = s.as_ref();
        let act = f16::from_hex(s);
        check_result(s, act, *exp, *roundtrip, i);
    }
}

#[test]
fn test_f32() {
    let checks = checks!(
        f32,
        [] //         Inputs {
           //             max: "0x1.fffffep+127",
           //             min_pos_normal: "0x1p-126",
           //             min_pos_subnormal: "0x1p-149",
           //         }
    );
    //     checks.extend([]);
    //     // let checks = [
    //     //     ("0x1.ffep+8", 0x43fff000),
    //     //     ("+0x1.ffep+8", 0x43fff000),
    //     //     ("0x1.99999ap-4", 0x3dcccccd),
    //     //     ("0x1.9p+6", 0x42c80000),
    //     //     ("0x1.2d5ed2p+20", 0x4996af69),
    //     //     ("-0x1.348eb8p+10", 0xc49a475c),
    //     //     ("-0x1.33dcfep-33", 0xaf19ee7f),
    //     //     ("0x1.99999ap-4", (0.1f32).to_bits()),
    //     //     ("-0x1.99999ap-4", (-0.1f32).to_bits()),
    //     //     ("0x1.111114p-127", 0x00444445),
    //     //     ("0x1.23456p-130", 0x00091a2b),
    //     // ];

    for (i, (s, exp, roundtrip)) in checks.iter().enumerate() {
        let s = s.as_ref();
        let act = f32::from_hex(s);
        check_result(s, act, *exp, *roundtrip, i);
    }
}

#[test]
fn test_f64() {
    let checks = checks!(
        f64,
        [] //         Inputs {
           //             max: "0x1.fffffffffffffp+1023",
           //             min_pos_normal: "0x1p-1022",
           //             min_pos_subnormal: "0x1p-1074",
           //         }
    );
    //     checks.extend([]);
    //     // let checks = [
    //     //     ("0x1.ffep+8", 0x407ffe0000000000),
    //     //     ("0x1.999999999999ap-4", 0x3fb999999999999a),
    //     //     ("0x1.9p+6", 0x4059000000000000),
    //     //     ("0x1.2d5ed1fe1da7bp+20", 0x4132d5ed1fe1da7b),
    //     //     ("-0x1.348eb851eb852p+10", 0xc09348eb851eb852),
    //     //     ("-0x1.33dcfe54a3803p-33", 0xbde33dcfe54a3803),
    //     //     ("0x1.999999999999ap-4", 0.1f64.to_bits()),
    //     //     ("0x1.999999999998ap-4", (0.1f64 - f64::EPSILON).to_bits()),
    //     //     ("-0x1.999999999999ap-4", (-0.1f64).to_bits()),
    //     //     ("-0x1.999999999998ap-4", (-0.1f64 + f64::EPSILON).to_bits()),
    //     //     ("0x0.8000000000001p-1022", 0x0008000000000001),
    //     //     ("0x0.123456789abcdp-1022", 0x000123456789abcd),
    //     //     ("0x0.0000000000002p-1022", 0x0000000000000002),
    //     // ];
    //     for (s, exp) in checks {
    //         let act = f64::from_hex(s);
    //         check_result(s, act, exp);
    //     }
    for (i, (s, exp, roundtrip)) in checks.iter().enumerate() {
        let s = s.as_ref();
        let act = f64::from_hex(s);
        check_result(s, act, *exp, *roundtrip, i);
    }
}

#[test]
#[cfg(target_has_reliable_f128)]
fn test_f128() {
    let mut checks = checks!(
        f128,
        [] //         Inputs {
           //             max: "0x1.ffffffffffffffffffffffffffffp+16383",
           //             min_pos_normal: "0x1p-16382",
           //             min_pos_subnormal: "0x1p-16494",
           //         }
    );
    //     checks.extend([]);
    //     // let checks = [
    //     //     ("0x1.ffep+8", 0x4007ffe0000000000000000000000000),
    //     //     ("+0x1.ffep+8", 0x4007ffe0000000000000000000000000),
    //     //     ("0x1.999999999999999999999999999ap-4", 0x3ffb999999999999999999999999999a),
    //     //     ("0x1.9p+6", 0x40059000000000000000000000000000),
    //     //     ("0x1.999999999999999999999999999ap-4", (0.1f128).to_bits()),
    //     //     ("-0x1.999999999999999999999999999ap-4", (-0.1f128).to_bits()),
    //     //     ("0x0.abcdef0123456789abcdef012345p-16382", 0x0000abcdef0123456789abcdef012345),
    //     //     ("0x1p-16494", 0x00000000000000000000000000000001),
    //     // ];
    for (i, (s, exp, roundtrip)) in checks.iter().enumerate() {
        let s = s.as_ref();
        let act = f128::from_hex(s);
        check_result(s, act, *exp, *roundtrip, i);
    }
}

fn check_result<F: Float>(
    s: &str,
    actual: Result<F, ParseHexFloatError>,
    expected: Result<F::Int, HexFloatErrorKind>,
    roundtrip: bool,
    index: usize,
) where
    Hex<F>: fmt::LowerHex + fmt::UpperHex,
{
    let actual = actual.map(F::to_bits).map_err(|e| e.kind());
    let hwidth = ((F::BITS / 4) + 2) as usize;
    let bwidth = (F::BITS + 2) as usize;
    let fmt_bits = |x: &Result<_, _>| match x {
        Ok(v) => format!("{v:#0hwidth$x} {v:#0bwidth$b} {:?}", F::from_bits(*v)),
        Err(e) => format!("{e:?}"),
    };

    assert_eq!(
        actual,
        expected,
        "\
        parsing:  {s:?}\n\
        actual:   {}\n\
        expected: {}\n\
        index:    {index}\
        ",
        fmt_bits(&actual),
        fmt_bits(&expected),
    );

    if roundtrip {
        let bits = expected.expect("can't roundtrip error cases");
        let x = F::from_bits(bits);

        assert_eq!(
            format!("{:x}", Hex(x)),
            s,
            "formatting: {x:?} {}\nindex: {index}",
            fmt_bits(&Ok(bits))
        );

        // The 0x and special values are always the same case, everything else is uppercase
        let expected_upper =
            s.to_uppercase().replace("0X", "0x").replace("NAN", "NaN").replace("INF", "inf");
        assert_eq!(
            format!("{:X}", Hex(x)),
            expected_upper,
            "formatting: {x:?} {}\nindex: {index}",
            fmt_bits(&Ok(bits))
        );
    }
}

/// Allow using generics while we can't implement `LowerHex`/`UpperHex` directly.
struct Hex<T>(T);

macro_rules! impl_hex {
    ($ty:ty) => {
        impl fmt::LowerHex for Hex<$ty> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt_lower_hex(f)
            }
        }

        impl fmt::UpperHex for Hex<$ty> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt_upper_hex(f)
            }
        }
    };
}

#[cfg(target_has_reliable_f16)]
impl_hex!(f16);
impl_hex!(f32);
impl_hex!(f64);
#[cfg(target_has_reliable_f128)]
impl_hex!(f128);

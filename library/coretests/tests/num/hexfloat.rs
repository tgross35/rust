//! Tests for float hex format parsing and printing.

use core::num::imp::hexfloat::{Status, parse_any};
use core::num::imp::{CastFrom, CastInto, Float};
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
#[cfg(target_has_reliable_f128_math)] // needs round_ties_even
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
        let Ok((x, status)) = parse_float::<f32>(&s) else { unreachable!() };
        let bits = x.to_bits();
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
        if x.is_nan() && y.is_nan() {
            continue;
        }

        assert_eq!(xi, yi, "{xi:#06x} != {yi:#06x} ({x} != {y}, {s})");
    }
}

#[test]
#[cfg(target_has_reliable_f16)]
fn test_f16_to_f32() {
    use std::format;

    // Exhaustively check that these are equivalent for all `f16`:
    //  - `f16 -> f32`
    //  - `f16 -> str -> f32`
    //  - `f16 -> f32 -> str -> f32`
    //  - `f16 -> f32 -> str -> f16 -> f32`
    for x in 0..=u16::MAX {
        let x16 = f16::from_bits(x);
        let s16 = Hex(x16).to_string();
        let x32 = x16 as f32;
        let s32 = Hex(x32).to_string();

        let a = f32::from_hex(&s16).unwrap();
        let b = f32::from_hex(&s32).unwrap();
        let c = f16::from_hex(&s32).unwrap();

        if x32.is_nan() && a.is_nan() && b.is_nan() && c.is_nan() {
            continue;
        }

        assert_eq!(
            x32.to_bits(),
            a.to_bits(),
            "{x16:?} : f16 formatted as {s16} which parsed as {a:?} : f16"
        );
        assert_eq!(
            x32.to_bits(),
            b.to_bits(),
            "{x32:?} : f32 formatted as {s32} which parsed as {b:?} : f32"
        );
        assert_eq!(
            x32.to_bits(),
            (c as f32).to_bits(),
            "{x32:?} : f32 formatted as {s32} which parsed as {c:?} : f16"
        );
    }
}

#[test]
#[cfg(target_has_reliable_f16)]
#[cfg(target_has_reliable_f128)]
fn printing_spot_checks() {
    assert_eq!(Hex(f16::MAX).to_string(), "0x1.ffcp+15");
    assert_eq!(Hex(f32::MAX).to_string(), "0x1.fffffep+127");
    assert_eq!(Hex(f64::MAX).to_string(), "0x1.fffffffffffffp+1023");
    assert_eq!(Hex(f128::MAX).to_string(), "0x1.ffffffffffffffffffffffffffffp+16383");

    assert_eq!(Hex(f16::MIN).to_string(), "-0x1.ffcp+15");
    assert_eq!(Hex(f32::MIN).to_string(), "-0x1.fffffep+127");
    assert_eq!(Hex(f64::MIN).to_string(), "-0x1.fffffffffffffp+1023");
    assert_eq!(Hex(f128::MIN).to_string(), "-0x1.ffffffffffffffffffffffffffffp+16383");

    assert_eq!(Hex(f16::ZERO).to_string(), "0x0p+0");
    assert_eq!(Hex(f32::ZERO).to_string(), "0x0p+0");
    assert_eq!(Hex(f64::ZERO).to_string(), "0x0p+0");
    assert_eq!(Hex(f128::ZERO).to_string(), "0x0p+0");

    assert_eq!(Hex(-f16::ZERO).to_string(), "-0x0p+0");
    assert_eq!(Hex(-f32::ZERO).to_string(), "-0x0p+0");
    assert_eq!(Hex(-f64::ZERO).to_string(), "-0x0p+0");
    assert_eq!(Hex(-f128::ZERO).to_string(), "-0x0p+0");

    assert_eq!(Hex(f16::NAN).to_string(), "NaN");
    assert_eq!(Hex(f32::NAN).to_string(), "NaN");
    assert_eq!(Hex(f64::NAN).to_string(), "NaN");
    assert_eq!(Hex(f128::NAN).to_string(), "NaN");

    assert_eq!(Hex(-f16::NAN).to_string(), "NaN");
    assert_eq!(Hex(-f32::NAN).to_string(), "NaN");
    assert_eq!(Hex(-f64::NAN).to_string(), "NaN");
    assert_eq!(Hex(-f128::NAN).to_string(), "NaN");

    assert_eq!(Hex(f16::INFINITY).to_string(), "inf");
    assert_eq!(Hex(f32::INFINITY).to_string(), "inf");
    assert_eq!(Hex(f64::INFINITY).to_string(), "inf");
    assert_eq!(Hex(f128::INFINITY).to_string(), "inf");

    assert_eq!(Hex(f16::NEG_INFINITY).to_string(), "-inf");
    assert_eq!(Hex(f32::NEG_INFINITY).to_string(), "-inf");
    assert_eq!(Hex(f64::NEG_INFINITY).to_string(), "-inf");
    assert_eq!(Hex(f128::NEG_INFINITY).to_string(), "-inf");
}

/// Moo
fn c<T>(val: T) -> Cow<'static, str>
where
    Cow<'static, str>: From<T>,
{
    Cow::from(val)
}

/// Test cases that apply to all float types.
macro_rules! checks {
    ($ty:ty, $inputs:expr) => {{
        type F = $ty;
        let mut v = vec![
            /*
             * Shifted equivalents
             */
            (c("0x.1234p+16"), Ok((0x1234 as F)), false, Status::OK),
            (c("0x1.234p+12"), Ok((0x1234 as F)), true, Status::OK),
            (c("0x12.34p+8"), Ok((0x1234 as F)), false, Status::OK),
            (c("0x123.4p+4"), Ok((0x1234 as F)), false, Status::OK),
            (c("0x1234p+0"), Ok((0x1234 as F)), false, Status::OK),
            (c("0x1234.p+0"), Ok((0x1234 as F)), false, Status::OK),
            (c("0x1234.0p+0"), Ok((0x1234 as F)), false, Status::OK),
            /*
             * Simple values
             */
            (c("0x0p+0"), Ok(0.0), true, Status::OK),
            (c("-0x0p+0"), Ok(-0.0), true, Status::OK),
            (c("+0x0p+0"), Ok(0.0), false, Status::OK),
            (c("0x1p+0"), Ok(1.0), true, Status::OK),
            (c("-0x1p+0"), Ok(-1.0), true, Status::OK),
            (c("+0x1p+0"), Ok(1.0), false, Status::OK),
            (c("0x1.p+0"), Ok(1.0), false, Status::OK),
            (c("0x1p+1"), Ok(2.0), true, Status::OK),
            (c("-0x1p+1"), Ok(-2.0), true, Status::OK),
            (c("+0x1p+1"), Ok(2.0), false, Status::OK),
            (c("0x2p+0"), Ok(2.0), false, Status::OK),
            (c("0x2.0p+0"), Ok(2.0), false, Status::OK),
            (c("0xfp+0"), Ok(15.0), false, Status::OK),
            /*
             * Special values
             */
            // Default NaN and indinity
            (c("NaN"), Ok(F::NAN), true, Status::OK),
            (c("-NaN"), Ok((-F::NAN)), false, Status::OK),
            (c("inf"), Ok(F::INFINITY), true, Status::OK),
            (c("-inf"), Ok(F::NEG_INFINITY), true, Status::OK),
            // NaN and infinity with case variations
            (c("nan"), Ok(F::NAN), false, Status::OK),
            (c("-nan"), Ok((-F::NAN)), false, Status::OK),
            (c("nAN"), Ok(F::NAN), false, Status::OK),
            (c("InF"), Ok(F::INFINITY), false, Status::OK),
            /*
             * Rounding checks
             */
            // Halfway between 0 and the min subnormal, tie broken to 0
            (
                c(format!("0x1p{}", F::EXP_MIN_SUBNORM - 1)),
                Ok(0.0),
                false,
                Status::INEXACT | Status::UNDERFLOW,
            ),
            // 0x1.00...1, nudge up from the above. Ensure we don't discard those values.
            (
                c(format!("0x1.{:01024}p{}", 1, F::EXP_MIN_SUBNORM - 1)),
                Ok(F::MIN_POSITIVE_SUBNORMAL),
                false,
                Status::INEXACT | Status::UNDERFLOW,
            ),
            // Exact
            (
                c(format!("0x1p{}", F::EXP_MIN_SUBNORM)),
                Ok(F::MIN_POSITIVE_SUBNORMAL),
                true,
                Status::OK,
            ),
            // 0x1.7fffff..., just below exact middle of 0 and the min subnormal value
            (
                c(format!("0x1.7{:f<100}p{}", "", F::EXP_MIN_SUBNORM)),
                Ok(F::MIN_POSITIVE_SUBNORMAL),
                false,
                Status::INEXACT | Status::UNDERFLOW,
            ),
            // Next value rounds up
            (
                c(format!("0x1.8p{}", F::EXP_MIN_SUBNORM)),
                Ok(F::from_bits(2)),
                false,
                Status::INEXACT | Status::UNDERFLOW,
            ),
            // Exact
            (
                c(format!("0x1p{}", F::EXP_MIN_SUBNORM + 1)),
                Ok(F::from_bits(2)),
                true,
                Status::OK,
            ),
            // Check around the next normal. Rounding tests near MAX, and INFINITY are in the
            // per-type functions because the logic producing the string is less straightforward.
            (c(format!("0x1p{}", F::EXP_MIN)), Ok(F::MIN_POSITIVE_NORMAL), true, Status::OK),
            // Exceeded limits
            (c("0x1p-1000000"), Ok(0.0), false, Status::INEXACT | Status::UNDERFLOW),
            (c("0x1p+1000000"), Ok(F::INFINITY), false, Status::INEXACT | Status::OVERFLOW),
            // Ensure huge exponents don't cause problems parsing
            (
                c(format!("0x1p+{}", u128::MAX)),
                Ok(F::INFINITY),
                false,
                Status::INEXACT | Status::OVERFLOW,
            ),
            (c("-0x1p+1000000"), Ok(F::NEG_INFINITY), false, Status::INEXACT | Status::OVERFLOW),
            /*
             * Errors
             */
            (c(""), Err(HexFloatErrorKind::NoHexIndicator), false, Status::OK),
            (c("+"), Err(HexFloatErrorKind::NoHexIndicator), false, Status::OK),
            (c("-"), Err(HexFloatErrorKind::NoHexIndicator), false, Status::OK),
            (c("1234"), Err(HexFloatErrorKind::NoHexIndicator), false, Status::OK),
            (c("0x"), Err(HexFloatErrorKind::EmptySignificand), false, Status::OK),
            (c("0x1p"), Err(HexFloatErrorKind::EmptyExponent), false, Status::OK),
            (c("0x1p+"), Err(HexFloatErrorKind::EmptyExponent), false, Status::OK),
            (c("0x1p-"), Err(HexFloatErrorKind::EmptyExponent), false, Status::OK),
            (c("0x789defg"), Err(HexFloatErrorKind::InvalidSignificand), false, Status::OK),
            (c("0x1p+a"), Err(HexFloatErrorKind::InvalidExponent), false, Status::OK),
        ];

        // TODO use scalbn so f128 works
        if <$ty as Float>::BITS > 16 && <$ty as Float>::BITS < 128 {
            let pow: $ty = -149.0;
            let x = pow.exp2();
            // f32 min (0x1p-149) using more hex digits than fit into a u128 at once
            v.push((c("0x0.00000000000000000000000000000000000008p+0"), Ok(x), false, Status::OK));
        }

        let inputs: &[(&str, $ty, bool, Status)] = &$inputs;
        for (s, expected, roundtrip, expected_status) in inputs.iter().copied() {
            v.push((c(s), Ok(expected), roundtrip, expected_status))
        }

        v
    }};
}

#[test]
#[cfg(target_has_reliable_f16)]
fn test_f16() {
    let checks = checks!(
        f16,
        [ // 8 9 a b c
            // Rounding
            ("0x1.ff8p+15", f16::MAX.next_down(), true, Status::OK),
            ("0x1.ffap+15", f16::MAX.next_down(), false, Status::INEXACT),
            ("0x1.ffa000000001p+15", f16::MAX, false, Status::INEXACT),
            ("0x1.ffcp+15", f16::MAX, true, Status::OK),
            ("0x1.ffcp+16", f16::INFINITY, false, Status::INEXACT | Status::OVERFLOW),
            // Exact maximum precision
            ("0x1.ffcp+0", f16::from_bits(u16::MAX >> 2), true, Status::OK),
            // Too precise
            ("0x1.ffdp+0", f16::from_bits(u16::MAX >> 2), false, Status::INEXACT),
            // Spot checks
            ("0x1.ffp+8", f16::from_bits(0x5ffc), true, Status::OK),
            ("0x1.ffp+8", f16::from_bits(0x5ffc), true, Status::OK),
            ("0x1.998p-4", f16::from_bits(0x2e66), true, Status::OK),
            ("0x1.9p+6", f16::from_bits(0x5640), true, Status::OK),
            ("0x1.998p-4", 0.1f16, true, Status::OK),
            ("-0x1.998p-4", -0.1f16, true, Status::OK),
            ("0x0.123p-12", f16::from_bits(0x0123), false, Status::OK),
        ]
    );

    for (i, (s, exp, roundtrip, exp_status)) in checks.iter().enumerate() {
        check_result::<f16>(s, *exp, *roundtrip, *exp_status, i);
    }
}

#[test]
fn test_f32() {
    let checks = checks!(
        f32,
        [
            // Limits and values that round to them
            ("0x1.fffffep+127", f32::MAX, true, Status::OK),
            // Rounding around the min subnormal
            ("0x1p-150", 0.0, false, Status::INEXACT | Status::UNDERFLOW),
            ("0x1.1p-150", f32::MIN_POSITIVE_SUBNORMAL, false, Status::INEXACT | Status::UNDERFLOW),
            ("0x1p-149", f32::MIN_POSITIVE_SUBNORMAL, true, Status::OK),
            // Spot checks
            ("0x1.ffep+8", f32::from_bits(0x43fff000), true, Status::OK),
            ("0x1.99999ap-4", f32::from_bits(0x3dcccccd), true, Status::OK),
            ("0x1.9p+6", f32::from_bits(0x42c80000), true, Status::OK),
            ("0x1.2d5ed2p+20", f32::from_bits(0x4996af69), true, Status::OK),
            ("-0x1.348eb8p+10", f32::from_bits(0xc49a475c), true, Status::OK),
            ("-0x1.33dcfep-33", f32::from_bits(0xaf19ee7f), true, Status::OK),
            ("0x1.99999ap-4", 0.1, true, Status::OK),
            ("-0x1.99999ap-4", -0.1, true, Status::OK),
            // ("0x1.111114p-127", 0x00444445),
            // ("0x1.23456p-130", 0x00091a2b),
            // ("0x1p-149", 0x00000001),
        ]
    );

    for (i, (s, exp, roundtrip, exp_status)) in checks.iter().enumerate() {
        check_result::<f32>(s, *exp, *roundtrip, *exp_status, i);
    }
}

#[test]
fn test_f64() {
    let checks = checks!(
        f64,
        [
            // ("0x1.ffep+8", 0x407ffe0000000000),
            // ("0x1.999999999999ap-4", 0x3fb999999999999a),
            // ("0x1.9p+6", 0x4059000000000000),
            // ("0x1.2d5ed1fe1da7bp+20", 0x4132d5ed1fe1da7b),
            // ("-0x1.348eb851eb852p+10", 0xc09348eb851eb852),
            // ("-0x1.33dcfe54a3803p-33", 0xbde33dcfe54a3803),
            // ("0x1.999999999999ap-4", 0.1f64.to_bits()),
            // ("0x1.999999999998ap-4", (0.1f64 - f64::EPSILON).to_bits()),
            // ("-0x1.999999999999ap-4", (-0.1f64).to_bits()),
            // ("-0x1.999999999998ap-4", (-0.1f64 + f64::EPSILON).to_bits()),
            // ("0x0.8000000000001p-1022", 0x0008000000000001),
            // ("0x0.123456789abcdp-1022", 0x000123456789abcd),
            // ("0x0.0000000000002p-1022", 0x0000000000000002),

        ]
    );
    for (i, (s, exp, roundtrip, exp_status)) in checks.iter().enumerate() {
        check_result::<f64>(s, *exp, *roundtrip, *exp_status, i);
    }
}

#[test]
#[cfg(target_has_reliable_f128)]
fn test_f128() {
    let checks = checks!(
        f128,
        [
            ("0x1.ffffffffffffffffffffffffffffp+16383", f128::MAX, true, Status::OK),
            // ("0x1.ffep+8", 0x4007ffe0000000000000000000000000),
            // ("+0x1.ffep+8", 0x4007ffe0000000000000000000000000),
            // ("0x1p+0", 0x3fff0000000000000000000000000000),
            // (
            //     "0x1.999999999999999999999999999ap-4",
            //     0x3ffb999999999999999999999999999a,
            // ),
            // ("0x1.9p+6", 0x40059000000000000000000000000000),
            // ("0x1.999999999999999999999999999ap-4", (0.1f128).to_bits()),
            // ("-0x1.999999999999999999999999999ap-4", (-0.1f128).to_bits()),
            // (
            //     "0x0.abcdef0123456789abcdef012345p-16382",
            //     0x0000abcdef0123456789abcdef012345,
            // ),
            // ("0x1p-16494", 0x00000000000000000000000000000001),
        ]
    );
    for (i, (s, exp, roundtrip, exp_status)) in checks.iter().enumerate() {
        check_result::<f128>(s, *exp, *roundtrip, *exp_status, i);
    }
}

fn check_result<F: Float>(
    s: &str,
    expected: Result<F, HexFloatErrorKind>,
    roundtrip: bool,
    expected_status: Status,
    index: usize,
) where
    Hex<F>: fmt::LowerHex + fmt::UpperHex,
{
    let actual = parse_float::<F>(s);
    let (actual, actual_status) = match actual {
        Ok((x, status)) => (Ok(x.to_bits()), Some(status)),
        Err(e) => (Err(e.kind()), None),
    };
    let expected = expected.map(F::to_bits);
    let hwidth = ((F::BITS / 4) + 2) as usize;
    let bwidth = (F::BITS + 2) as usize;
    let fmt_bits = |x: &Result<F::Int, _>| match x {
        Ok(v) => format!("{v:#0hwidth$x} {v:#0bwidth$b} {:?}", F::from_bits(*v)),
        Err(e) => format!("{e:?}"),
    };

    assert_eq!(
        (actual, actual_status),
        (expected, expected.is_ok().then_some(expected_status)),
        "\
        parsing:  {s:?}\n\
        actual:   {}\n\
        expected: {}\n\
        index:    {index}\
        ",
        fmt_bits(&actual),
        fmt_bits(&expected),
    );

    // Check formatting for non-error cases.
    let Ok(bits) = expected else {
        return;
    };
    let x = F::from_bits(bits);

    let formatted = Hex(x).to_string();

    if roundtrip {
        assert_eq!(formatted, s, "formatting: {x:?} {}\nindex: {index}", fmt_bits(&Ok(bits)));

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

impl<T> fmt::Display for Hex<T>
where
    Hex<T>: fmt::LowerHex,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

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

/// Wrapper for `parse_any` that fills in float info
fn parse_float<F: Float>(s: &str) -> Result<(F, Status), ParseHexFloatError> {
    let (bits, status) = parse_any(s.as_bytes(), F::BITS, F::SIG_BITS)?;
    Ok((F::from_bits(F::Int::cast_from(bits)), status))
}

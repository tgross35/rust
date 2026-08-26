//!
//!
//! IEEE specifies the following are acceptable hex representations:
//!
//! ```text
//! sign [+ −]
//! digit [0123456789]
//! hexDigit [0123456789abcdefABCDEF]
//! hexExpIndicator [Pp]
//! hexIndicator "0" [Xx]
//! hexSignificand ( {hexDigit} * "." {hexDigit}+ | {hexDigit}+ "." | {hexDigit}+ )
//! decExponent {hexExpIndicator} {sign}? {digit}+
//! hexSequence {sign}? {hexIndicator} {hexSignificand} {decExponent}
//! ```
//!
//! Translated to the format Rust documentation uses:
//!
//! ```markdown
//! SIGN -> `+` | `-`
//! DIGIT -> [`0`-`9`]
//! HEX_DIGIT -> [`0`-`9` `a`-`f` `A`-`F`]
//! HEX_EXP_INDICATOR -> `p` | `P`
//! HEX_INDICATOR -> `0` (`x` | `X`)
//! HEX_SIGNIFICAND -> (HEX_DIGIT* `.` HEX_DIGIT+ | HEX_DIGIT `.` | HEX_DIGIT+)
//! DEC_EXPONENT -> HEX_EXP_INDICATOR SIGN? DIGIT+
//! HEX_FLOAT -> SIGN? HEX_INDICATOR HEX_SIGNIFICAND DEC_EXPONENT
//! ```
//!
//! ```markdown
//! HEX_OUTPUT ->
//!     SIGN? `0x` (`1` | `0`) (`.` HEX_SEQUENCE)? HEX_INDICATOR SIGN DEC_SEQUENCE
//! SIGN -> `+` | `-`
//! HEX_EXP_INDICATOR -> `p` | `P`
//! HEX_SEQUENCE -> [`0`-`9` `a`-`f` `A`-`F`]+
//! DEC_SEQUENCE -> [`0`-`9`]+
//! ```
//!

use crate::fmt;
use crate::num::imp::Float;
use crate::num::{HexFloatErrorKind, ParseHexFloatError};

// TODO desc
/// Parse the `HEX_SIGNIFICAND DEC_EXPONENT` portion for an unspecified float type.
pub(crate) const fn parse_any(
    mut b: &[u8],
    bits: u32,
    sig_bits: u32,
) -> Result<u128, ParseHexFloatError> {
    // Check for unsupported formats
    debug_assert!(sig_bits <= 119);
    debug_assert!(bits > sig_bits + 3);
    debug_assert!(bits <= 128, "u128 repr limitations");
    debug_assert!(bits < sig_bits + 30, "32-bit exponent repr limitations");

    // Handle sign indicator
    let neg = matches!(b, [b'-', ..]);
    if let &[b'-' | b'+', ref rest @ ..] = b {
        b = rest;
    }

    let sign_bit = 1 << (bits - 1);
    let quiet_bit = 1 << (sig_bits - 1);
    let nan = sign_bit - quiet_bit;
    let inf = nan - quiet_bit;

    let mut x = match *b {
        [b'i' | b'I', b'n' | b'N', b'f' | b'F'] => inf,
        [b'n' | b'N', b'a' | b'A', b'n' | b'N'] => nan,
        [b'0', b'x' | b'X', ref rest @ ..] => match parse_finite(rest, bits, sig_bits) {
            Ok(v) => v,
            Err(e) => return Err(ParseHexFloatError { kind: e }),
        },
        _ => return Err(ParseHexFloatError { kind: HexFloatErrorKind::NoHexIndicator }),
    };

    if neg {
        x ^= sign_bit;
    }

    Ok(x)
}

// TODO desc
/// Parse the `HEX_SIGNIFICAND DEC_EXPONENT` portion for an unspecified float type.
const fn parse_finite(b: &[u8], bits: u32, sig_bits: u32) -> Result<u128, HexFloatErrorKind> {
    let exp_bits: u32 = bits - sig_bits - 1;
    let max_msb: i32 = (1 << (exp_bits - 1)) - 1;
    // The exponent of one ULP in the subnormals
    let min_lsb: i32 = 1 - max_msb - sig_bits as i32;

    let (mut sig, mut exp) = match parse_hex(b) {
        Err(e) => return Err(e),
        // Fast path: result is zero
        Ok(Parsed { sig: 0, .. }) => return Ok(0),
        // Others need rounding
        Ok(Parsed { sig, exp }) => (sig, exp),
    };

    let mut round_bits = sig.ilog2().cast_signed() - sig_bits.cast_signed();

    // Round at least up to min_lsb
    if exp < min_lsb - round_bits {
        round_bits = min_lsb - exp;
    }

    exp += round_bits;

    if round_bits > 0 {
        // first, prepare for rounding exactly two bits
        if round_bits == 1 {
            sig <<= 1;
        } else if round_bits > 2 {
            sig = shr_odd_rounding(sig, (round_bits - 2) as u32);
        }

        // Divide by 4, performing to nearest. Uses a lookup table on the last three
        // bits for when to round up.
        //
        // If the last two bits before shifting are nonzero, this is inexact.
        let t = (sig as u32) & 0b111;
        sig >>= 2;
        sig += ((0b11001000_u8 >> t) & 1) as u128;
    } else if round_bits < 0 {
        sig <<= -round_bits;
    }

    // The parsed value is X = sig * 2^exp
    // Expressed as a multiple U of the smallest subnormal value:
    // X = U * 2^min_lsb, so U = sig * 2^(exp-min_lsb)
    let uexp = (exp - min_lsb) as u128;
    let uexp = uexp << sig_bits;

    // Note that it is possible for the exponent bits to equal 2 here
    // if the value rounded up, but that means the mantissa is all zeroes
    // so the value is still correct
    debug_assert!(sig <= 2 << sig_bits);

    let inf = ((1 << exp_bits) - 1) << sig_bits;

    let bits = match sig.checked_add(uexp) {
        Some(bits) if bits < inf => bits,
        // overflow to infinity
        _ => inf,
    };
    Ok(bits)
}

/// Shift right, rounding all inexact divisions to the nearest odd number
/// E.g. (0 >> 4) -> 0, (1..=31 >> 4) -> 1, (32 >> 4) -> 2, ...
///
/// Useful for reducing a number before rounding the last two bits, since
/// the result of the final rounding is preserved for all rounding modes.
const fn shr_odd_rounding(x: u128, k: u32) -> u128 {
    if k < 128 {
        let inexact = x.trailing_zeros() < k;
        (x >> k) | (inexact as u128)
    } else {
        (x != 0) as u128
    }
}

/// A parsed finite and unsigned floating point number.
struct Parsed {
    /// Absolute value sig * 2^exp
    sig: u128,
    exp: i32,
}

/// Parse the `HEX_SIGNIFICAND DEC_EXPONENT` portion for an unspecified float type.
const fn parse_hex(mut b: &[u8]) -> Result<Parsed, HexFloatErrorKind> {
    let mut sig: u128 = 0;
    let mut exp: i32 = 0;

    let mut seen_point = false;
    let mut some_digits = false;
    let mut inexact = false;

    while let &[c, ref rest @ ..] = b {
        b = rest;

        match c {
            // Fraction separator
            b'.' => {
                if seen_point {
                    return Err(HexFloatErrorKind::InvalidSignificand);
                }
                seen_point = true;
                continue;
            }
            // Start of exponent
            b'p' | b'P' => break,
            // Hex character
            c => {
                let digit = match hex_digit(c) {
                    Some(d) => d,
                    None => return Err(HexFloatErrorKind::InvalidSignificand),
                };
                some_digits = true;

                // Store as much as possible in the significand before incrementing the exponent.
                if (sig >> 124) == 0 {
                    sig <<= 4;
                    sig |= digit as u128;
                } else {
                    // Saturating math to account for overflow if there are ~i32::MAX/4 digits.
                    exp = exp.saturating_add(4);

                    // There are more digits than we can store in a u128 so some data will be
                    // truncated.
                    inexact |= digit != 0;
                }

                // Up until the fractional point, the value grows with more digits, but after it
                // the exponent is compensated to match.
                if seen_point {
                    // Saturating math to account for overflow if there are ~i32::MAX/4 fractional
                    // digits.
                    exp = exp.saturating_sub(4);
                }
            }
        }
    }

    // If we've set inexact, the exact value has more than 125
    // significant bits, and lies somewhere between sig and sig + 1.
    // Because we'll round off at least two of the trailing bits,
    // setting the last bit gives correct rounding for inexact values.
    sig |= inexact as u128;

    if !some_digits {
        return Err(HexFloatErrorKind::Empty);
    };
    some_digits = false;

    // Handle the exponent now

    let negate_exp = matches!(b, [b'-', ..]);
    if let &[b'-' | b'+', ref rest @ ..] = b {
        b = rest;
    }

    let mut pexp: u32 = 0;
    while let &[c, ref rest @ ..] = b {
        b = rest;
        let digit = match dec_digit(c) {
            Some(d) => d,
            None => return Err(HexFloatErrorKind::InvalidExponent),
        };
        some_digits = true;
        pexp = pexp.saturating_mul(10);
        pexp += digit as u32;
    }

    if !some_digits {
        return Err(HexFloatErrorKind::EmptyExponent);
    };

    if negate_exp {
        exp = exp.saturating_sub_unsigned(pexp);
    } else {
        exp = exp.saturating_add_unsigned(pexp);
    };

    Ok(Parsed { sig, exp })
}

const fn dec_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        _ => None,
    }
}

const fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// TODO based on hexfloat2 comment
pub(crate) fn fmt_any_hex<F: Float>(x: &F, f: &mut fmt::Formatter<'_>, upper: bool) -> fmt::Result {
    if x.is_sign_negative() {
        write!(f, "-")?;
    } else if f.sign_plus() {
        write!(f, "+")?;
    }

    if *x == F::ZERO {
        if upper {
            return write!(f, "0x0P+0");
        } else {
            return write!(f, "0x0p+0");
        }
    }

    if x.is_nan() {
        return write!(f, "NaN");
    }

    if x.is_infinite() {
        return write!(f, "inf");
    }

    let mut exponent = x.exp_unbiased();
    let sig = x.to_bits() & F::SIG_MASK;

    let bias = F::EXP_BIAS as i32;
    // The mantissa MSB needs to be shifted up to the nearest nibble.
    let mshift = (4 - (F::SIG_BITS % 4)) % 4;
    let sig = sig << mshift;
    // The width is rounded up to the nearest char (4 bits)
    let mwidth = (F::SIG_BITS as usize + 3) / 4;
    let leading = if exponent == -bias {
        // subnormal number means we shift our output by 1 bit.
        exponent += 1;
        "0."
    } else {
        "1."
    };

    if upper {
        write!(f, "0x{leading}{sig:0mwidth$X}P{exponent:+}")
    } else {
        write!(f, "0x{leading}{sig:0mwidth$x}p{exponent:+}")
    }
}

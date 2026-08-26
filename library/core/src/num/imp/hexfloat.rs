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
use crate::fmt::NumBuffer;
use crate::mem::MaybeUninit;
use crate::num::imp::{CastFrom, Float, Int};
use crate::num::{HexFloatErrorKind, ParseHexFloatError};

// TODO desc
/// Parse the `HEX_SIGNIFICAND DEC_EXPONENT` portion for an unspecified float type.
pub const fn parse_any(
    mut b: &[u8],
    bits: u32,
    sig_bits: u32,
) -> Result<(u128, Status), ParseHexFloatError> {
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

    let (mut x, status) = match *b {
        [b'i' | b'I', b'n' | b'N', b'f' | b'F'] => (inf, Status::OK),
        [b'n' | b'N', b'a' | b'A', b'n' | b'N'] => (nan, Status::OK),
        [b'0', b'x' | b'X', ref rest @ ..] => match parse_finite(rest, bits, sig_bits) {
            Ok(v) => v,
            Err(e) => return Err(ParseHexFloatError { kind: e }),
        },
        _ => return Err(ParseHexFloatError { kind: HexFloatErrorKind::NoHexIndicator }),
    };

    if neg {
        x ^= sign_bit;
    }

    Ok((x, status))
}

// TODO desc
/// Parse the `HEX_SIGNIFICAND DEC_EXPONENT` portion for an unspecified float type.
const fn parse_finite(
    b: &[u8],
    bits: u32,
    sig_bits: u32,
) -> Result<(u128, Status), HexFloatErrorKind> {
    let exp_bits: u32 = bits - sig_bits - 1;
    let max_msb: i32 = (1 << (exp_bits - 1)) - 1;
    // The exponent of one ULP in the subnormals
    let min_lsb: i32 = 1 - max_msb - sig_bits as i32;

    let (mut sig, mut exp) = match parse_hex(b) {
        Err(e) => return Err(e),
        // Fast path: result is zero
        Ok(Parsed { sig: 0, .. }) => return Ok((0, Status::OK)),
        // Others need rounding
        Ok(Parsed { sig, exp }) => (sig, exp),
    };

    let mut round_bits = sig.ilog2().cast_signed() - sig_bits.cast_signed();

    // Round at least up to min_lsb
    if exp < min_lsb - round_bits {
        round_bits = min_lsb - exp;
    }

    let mut status = Status::OK;

    exp += round_bits;

    if round_bits > 0 {
        // first, prepare for rounding exactly two bits
        if round_bits == 1 {
            sig <<= 1;
        } else if round_bits > 2 {
            sig = shr_odd_rounding(sig, (round_bits - 2) as u32);
        }

        if sig & 0b11 != 0 {
            status = Status::INEXACT;
        }

        sig = shr2_round(sig);
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
        Some(bits) if bits < inf => {
            // inexact subnormal or zero?
            if status.inexact() && bits < (1 << sig_bits) {
                status = status.with(Status::UNDERFLOW);
            }
            bits
        }
        _ => {
            // overflow to infinity
            status = status.with(Status::OVERFLOW).with(Status::INEXACT);
            inf
        }
    };
    Ok((bits, status))
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

/// Divide by 4, rounding with the given mode
const fn shr2_round(mut x: u128) -> u128 {
    let t = (x as u32) & 0b111;
    x >>= 2;
    x + ((0b11001000_u8 >> t) & 1) as u128
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
        return Err(HexFloatErrorKind::EmptySignificand);
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

pub(crate) fn fmt_any_hex<F: Float>(x: F, f: &mut fmt::Formatter<'_>, upper: bool) -> fmt::Result {
    // Enough for the typical `+0x1.*p+*` with a u128 significand and u32 exponent. Could be
    // shorter for f16/f32/f64, but that is a pain without GCE.
    const BUFLEN: usize =
        "+0x1.".len() + (128 / 4) + "p+".len() + <u32 as fmt::NumBufferTrait>::DEFAULT.len();
    let mut buf = [MaybeUninit::<u8>::uninit(); BUFLEN];
    let mut exp_buf = NumBuffer::new();
    let mut idx = 0;
    let izero = F::Int::ZERO;

    let mut inf = "inf";
    if x.is_sign_negative() {
        inf = "-inf";
        buf[idx].write(b'-');
        idx += 1;
    } else if f.sign_plus() {
        inf = "+inf";
        buf[idx].write(b'+');
        idx += 1;
    };

    if x.is_nan() {
        return f.pad("NaN");
    }
    if x.is_infinite() {
        return f.pad(inf);
    }

    // Construct a normalized significand and exponent, with the implicit bit unset
    let xi = x.to_bits();
    let mut exp = x.exp_unbiased();
    let mut sig = xi & F::SIG_MASK;
    // Leading zeros only in the mantissa, determines leading 0s in the fract output
    let lz = sig.leading_zeros() - F::EXP_BITS - 1;

    if x.ex() == 0 && sig != izero {
        // Normalize subnormals
        sig <<= lz + 1;
        exp -= lz.cast_signed();
    }

    // Shift so significand MSB is now the int top bit
    sig <<= F::EXP_BITS + 1;

    let mut pfx = "0x1";
    if x == F::ZERO {
        exp = 0;
        pfx = "0x0";
    }

    buf[idx..(idx + pfx.len())].write_copy_of_slice(pfx.as_bytes());
    idx += pfx.len();

    let max_hex_digits = F::BITS / 4;
    let mut frac_digits = (F::BITS - sig.trailing_zeros()).div_ceil(4);
    let mut trailing_zeros = 0;

    if let Some(prec) = f.precision() {
        let prec = prec as u32; // value is limited to u16
        if prec == 0 {
            // No fractional part to write
            sig = izero;
        } else if prec >= frac_digits {
            // We're going to need to pad
            trailing_zeros = prec - max_hex_digits;
        } else {
            // prec < frac_digits, will round. Round with a mask before truncating.
            let drop_mask = F::Int::MAX >> (4 * prec as u32);
            let round = drop_mask >> 1;
            sig += round;
            sig &= !drop_mask;
            frac_digits = prec;
        }
    }

    if frac_digits > 0 || trailing_zeros > 0 {
        buf[idx].write(b'.');
        idx += 1;
    }

    let hex = if upper { b"0123456789ABCDEF" } else { b"0123456789abcdef" };
    for i in 0..frac_digits {
        let nibble = u8::cast_from(sig >> (F::BITS - 4 * (i + 1)) as u32) & 0xf;
        buf[idx].write(hex[usize::from(nibble)]);
        idx += 1;
    }

    let trailing_zeros_pos = idx;

    // Write the exponent to the same buffer
    if upper {
        buf[idx].write(b'P');
    } else {
        buf[idx].write(b'p');
    }
    idx += 1;
    buf[idx].write(if exp >= 0 { b'+' } else { b'-' });
    idx += 1;
    let exponent = exp.unsigned_abs().format_into(&mut exp_buf);
    buf[idx..(idx + exponent.len())].write_copy_of_slice(exponent.as_bytes());
    idx += exponent.len();

    // SAFETY: up to `idx` has been initialized, and we only write ASCII
    let res = unsafe { str::from_utf8_unchecked(buf[..idx].assume_init_ref()) };
    let total_len = res.len() + trailing_zeros as usize;
    let padding = f.width().unwrap_or(0).saturating_sub(total_len);

    let post_padding =
        f.padding(padding.try_into().expect("width capped at u16"), fmt::Alignment::Right)?;
    if trailing_zeros == 0 {
        // Write with a single call if possible
        f.write_str(res)?;
    } else {
        let (sig, exp) = res.split_at(trailing_zeros_pos);
        write!(f, "{sig}{empty:0pad$}{exp}", empty = "", pad = trailing_zeros as usize)?;
    }
    post_padding.write(f)?;

    Ok(())
}

/// Float status flags. Public only for testing.
#[derive(Clone, Copy)]
pub struct Status(u8);

impl Status {
    pub(crate) const OK: Status = Status(0);
    pub(crate) const OVERFLOW: Status = Status(1);
    pub(crate) const UNDERFLOW: Status = Status(1 << 1);
    pub(crate) const INEXACT: Status = Status(1 << 2);

    /// True if `OVERFLOW` is set.
    pub const fn overflow(self) -> bool {
        self.0 & Self::OVERFLOW.0 != 0
    }

    /// True if `UNDERFLOW` is set.
    pub const fn underflow(self) -> bool {
        self.0 & Self::UNDERFLOW.0 != 0
    }

    /// True if `INEXACT` is set.
    pub const fn inexact(self) -> bool {
        self.0 & Self::INEXACT.0 != 0
    }

    pub const fn with(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // shift -> flag map
        let names = &["OVERFLOW", "UNDERFLOW", "INEXACT"];

        write!(f, "Status(")?;
        let mut any = false;
        for shift in 0..u8::BITS {
            if self.0 & (1 << shift) != 0 {
                if any {
                    write!(f, " | ")?;
                }
                match names.get(shift as usize) {
                    Some(name) => write!(f, "{name}")?,
                    None => write!(f, "UNKNOWN(1 << {shift})")?,
                }
                any = true;
            }
        }

        if !any {
            write!(f, "OK")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

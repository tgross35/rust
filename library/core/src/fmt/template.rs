//! The binary template for formatting
//!
//! This format is defined in this pseudocode
//!
//! ```ignore
//! /// A `Size` is either a `u8` or a `usize` based on some determiner. It can be a length
//! /// or an index. Determiner is one bit and has values:
//! ///
//! /// - `0b0`: size is a `u8`
//! /// - `0b1`: size is a `usize`
//! pseudotype Size(determiner: 1bit);
//!
//! /// A `SmallSize`
//!
//! /// Something `Repeating` with dynamic length
//! pseudotype Repeating(length)<T>;
//!
//! /// An optional type may or may not be present based on some preceding information.
//! /// Determiner is 1 bit and has values:
//! ///
//! /// - `0b0`: value is not present
//! /// - `0b1`: value is present
//! pseudotype Optional(determiner: 1bit)<T>;
//!
//! /// A type that may be either inline or in associated data. Determiner is 2 bits and has values:
//! ///
//! /// - `0b00`: value is default or unspecified
//! /// - `0b01`: value is immediate inline. Next data will be a value
//! /// - `0b11`: value is a parameter. Next data will be an index
//! ///
//! /// This is used to create a `Count` in runtime
//! pseudotype SelectableSource(determiner: 2bit)<T>;
//!
//! /// The format arguments that live on the stack
//! struct Arguments {
//!     template: &'static Template,
//!     
//! }
//!
//! /// The static template
//! /// size: dynamic
//! struct Template {
//!     template_meta: TemplateMeta,
//!     /// The number of parts in the next section
//!     parts_len: Size(template_meta.parts_len_is_usize),
//!     /// A single string or format placeholder
//!     parts: Repeating(parts_len)<Part>,
//! }
//!
//! /// Information about the entire template
//! /// size: u8
//! type TemplateMeta = bitfields(u8) {
//!     parts_len_is_usize: 0,
//! }
//!
//! /// A single "part"
//! /// size: dynamic
//! struct Part {
//!     /// u8
//!     part_meta: PartMeta,
//!     /// length of the data for this part
//!     part_data_len: Size(part_meta.len_is_usize),
//!     /// Info about the fill character
//!     fill: Optional(part_meta.has_fill_or_flags)<FillInfo>,
//!     ///
//!     precision: Optional(fill.precision_size_det)
//!     ///
//!     width: Optional(fill.width_size_det)
//! }
//!
//!
//! /// Metadata for a single part
//! /// size: u8
//! type PartMeta = bitfields(u8) {
//!     len_is_usize: 1,
//!     has_fill_or_flags: 1,
//!     precision_source: 2..=3,
//!     width_source: 4..=5,
//!     text_align: 6..=7,
//! }
//!
//! // Unicode chars have a 21-bit space. We pack more data into the unused bits.
//! // Invariant: `!(debug_lower_hex & debug_upper_hex)`
//! // size: u32
//! type FillInfo = bitfields(u32) {
//!     char: 0..=20,
//!     sign_minus: ,
//!     sign_plus: ,
//!     alternate: ,
//!     sign_aware_zero_pad: 27,
//!     // Use lower hex debug
//!     debug_lower_hex: 28,
//!     // Use upper hex debug
//!     debug_upper_hex: 29,
//!     precision_size_det: 30,
//!     width_size_det: 31,
//! }
//!

#![unstable(feature = "fmt_internals", reason = "internal to format_args!", issue = "none")]

use super::rt;
use core::ops::Fn;

use crate::marker::PhantomData;
// use crate::ptr;

/// A thin pointer to a formatting "recipe".
///
/// This is a pointer to a binary template
///
/// It combines the `pieces` and `fmt` parts of the template
#[derive(Clone, Copy)]
pub struct Template<'a> {
    ptr: *const u8,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Template<'a> {
    /// Create a template
    ///
    /// # Safety
    ///
    /// `buf` must contain a valid template
    #[inline]
    pub unsafe fn new(buf: &'a [u8]) -> Self {
        Self { ptr: buf.as_ptr(), phantom: PhantomData }
    }

    pub fn as_str(&self) -> Option<&str> {
        let mut reader = self.reader();
        if reader.remaining() == 0 {
            Some("")
        } else if reader.remaining() == 1 {
            todo!()
        } else {
            None
        }
    }

    /// Construct a reader
    fn reader(self) -> TemplateReader<'a> {
        todo!()
    }
}

/// An iterator-like
pub struct TemplateReader<'a> {
    next_ptr: *const u8,
    remaining: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'a> TemplateReader<'a> {
    /// The number of remaining format pieces to handle. If `write_next` has not been
    /// called yet, this will be the total number of pieces in the template.
    pub fn remaining(&self) -> usize {
        self.remaining
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TemplateMeta(pub u8);

impl TemplateMeta {
    const PART_LEN_IS_USIZE: Self = Self(0b00000001);

    pub fn new(part_len_is_usize: bool) -> Self {
        let mut ret = Self(0);
        ret.0 |= Self::PART_LEN_IS_USIZE.0;
        ret
    }

    pub fn write_with_size(self, size: usize, push_slice: impl Fn(&[u8])) {
        let size = Size::from_usize(size);
        let this = Self::new(size.is_usize());
        push_slice(&[self.0]);
        size.write(push_slice);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PartMeta(pub u8);

impl PartMeta {
    /// Whether this part's data is a `u8` or `usize`
    const PART_DATA_LEN_IS_USIZE: u8 = 0b00000001;
    /// Whether we need to check the arguments data to use this
    const IS_DYNAMIC_ARGUMENT: u8 = 0b00000010;
    /// Whether we are provided a fill or flags value
    const HAS_FILL_OR_FLAGS: u8 = 0b00000100;
    /// Whether precision is inline, parameterized, or not present
    const PRECISION_SOURCE: u8 = 0b00011000;
    const PRECISION_SOURCE_SHFT: u8 = (Self::PRECISION_SOURCE.ilog2() - 1) as u8;
    /// Whether width is inline, parameterized, or not present
    const WIDTH_SOURCE: u8 = 0b01100000;
    const WIDTH_SOURCE_SHFT: u8 = (Self::WIDTH_SOURCE.ilog2() - 1) as u8;

    pub fn new(
        part_data_len_is_usize: bool,
        is_dynamic_argument: bool,
        has_fill_or_flags: bool,
        precision: &rt::Count,
        width: &rt::Count,
    ) -> Self {
        let mut ret = Self(0);
        if part_data_len_is_usize {
            ret.0 |= Self::PART_DATA_LEN_IS_USIZE;
        }
        if is_dynamic_argument {
            ret.0 |= Self::IS_DYNAMIC_ARGUMENT;
        }
        if has_fill_or_flags {
            ret.0 |= Self::HAS_FILL_OR_FLAGS;
        }
        ret.0 |= count_to_bits(precision) << Self::PRECISION_SOURCE_SHFT;
        ret.0 |= count_to_bits(width) << Self::WIDTH_SOURCE_SHFT;

        ret
    }
}

fn count_to_bits(count: &rt::Count) -> u8 {
    match count {
        rt::Count::Param(_) => 0b11,
        rt::Count::Is(_) => 0b01,
        rt::Count::Implied => 0b00,
    }
}

/// `FillInfo` is used to construct most of the `Formatter` struct, as well as determination
/// for which formatter to call.
///
/// This contains a unicode fill character. Unicode uses a 21-bit representation so we use
/// the lower 3 bytes for the character, upper byte for our formatter
#[derive(Debug, Clone, Copy)]
struct Placeholder(pub u32);

impl Placeholder {
    /// This is the fill character
    const FILL_CHAR: Self = Self(3 * (u8::MAX as u32));
    /// The `TextAlignment` value
    const TEXT_ALIGN: Self = Self(0b0);
    /// `-` flag was specified
    const SIGN_MINUS: Self = Self(0b00000001 << 24);
    /// `+` flag was specified
    const SIGN_PLUS: Self = Self(0b00000010 << 24);
    /// `#` flag was specified
    const ALTERNATE: Self = Self(0b00000100 << 24);
    /// `0` flag was specified
    const SIGN_AWARE_ZERO_PAD: Self = Self(0b00001000 << 24);
    /// Use `LowerHex` formatting
    const DEBUG_LOWER_HEX: Self = Self(0b00010000 << 24);
    /// Use `UpperHex` formatting
    const DEBUG_UPPER_HEX: Self = Self(0b00100000 << 24);
    /// Determiner of precision source if inline
    const PRECISION_SOURCE_DET: Self = Self(0b01000000 << 24);
    /// Determiner of width source if inline
    const WIDTH_SOURCE_DET: Self = Self(0b10000000 << 24);

    fn make_thing(placeholder: rt::Placeholder) -> Option<Self> {
        // TODO
        None
    }
}

pub struct Part {}

pub enum PartData<'a> {
    Literal(&'a [u8]),
    Dynamic(usize),
}

/// Helper to construct a template in the compiler
// TODO: this is probably fragile for cross compilation because it uses the host
// `usize` rather than target. Figure this out as well has how to use `ne` bytes
// instead of `le`.
pub fn write_new_part(
    data: PartData<'_>,
    placeholder: Option<rt::Placeholder>,
    push_slice: impl Fn(&[u8]),
) {
    let (size, flags) = match data {
        PartData::Literal(lit) => (Size::from_usize(lit.len()), None),
        PartData::Dynamic(idx) => {
            (Size::from_usize(idx), placeholder.and_then(|ph| Placeholder::make_thing(ph)))
        }
    };

    let precision = placeholder.map(|ph| ph.precision).unwrap_or(rt::Count::Implied);
    let width = placeholder.map(|ph| ph.width).unwrap_or(rt::Count::Implied);

    let meta = PartMeta::new(
        size.is_usize(),
        matches!(data, PartData::Dynamic(_)),
        flags.is_some(),
        &precision,
        &width,
    );

    push_slice(&[meta.0]);

    if let Some(flags) = flags {
        push_slice(&flags.0.to_ne_bytes());
    }

    if let rt::Count::Is(val) = precision {
        Size::from_usize(val).write(push_slice);
    }

    if let rt::Count::Is(val) = width {
        Size::from_usize(val).write(push_slice);
    }

    size.write(push_slice);

    if let PartData::Literal(lit) = data {
        push_slice(lit);
    }
}

/// Variable-width encoding of a length or index
#[cfg(not(bootstrap))]
enum Size {
    U8(u8),
    Usize(usize),
}

#[cfg(not(bootstrap))]
impl Size {
    fn from_usize(val: usize) -> Self {
        match u8::try_from(val).ok() {
            Some(v) => Self::U8(v),
            None => Self::Usize(val),
        }
    }

    fn is_usize(&self) -> bool {
        matches!(self, Self::Usize(_))
    }

    fn write(&self, push_slice: impl Fn(&[u8])) {
        match self {
            Size::U8(v) => push_slice(&[v]),
            Size::Usize(v) => push_slice(&v.to_le_bytes()),
        }
    }
}

/// Helper to read a pointer unaligned as either a `u8` or a `usize` and return the
/// next pointer.
///
/// # Safety
///
/// `ptr` must be valid as `usize`
unsafe fn read_u8_or_usize(ptr: *const u8, is_usize: bool) -> (usize, *const u8) {
    if is_usize {
        let casted = ptr.cast::<usize>();

        // SAFETY: `ptr` is valid for `usize` by function requirements, `usize` meets the
        // `read_unaligned` `Copy` requirement.
        let val = unsafe { casted.read_unaligned() };

        (val, casted.add(1).cast())
    } else {
        // SAFETY: `ptr` is valid for `u8` by function requirements
        let val = unsafe { ptr.read().into() };
        (val, ptr.add(1))
    }
}

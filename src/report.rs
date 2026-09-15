// Copyright Mozilla Foundation
//
// Licensed under the Apache License (Version 2.0), or the MIT license,
// (the "Licenses") at your option. You may not use this file except in
// compliance with one of the Licenses. You may obtain copies of the
// Licenses at:
//
//    https://www.apache.org/licenses/LICENSE-2.0
//    https://opensource.org/licenses/MIT
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the Licenses is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the Licenses for the specific language governing permissions and
// limitations under the Licenses.

use crate::helpers::*;
use crate::Utf16Handler;
use core::fmt::Formatter;

/// A type for signaling UTF-16 errors.
///
/// The value of the unpaired surrogate is not exposed in order
/// to keep the `Result` type (and `Option`-wrapping thereof)
/// the same size as `char`.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct Utf16CharsError;

impl core::fmt::Display for Utf16CharsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "unpaired surrogate")
    }
}

impl core::error::Error for Utf16CharsError {}

/// The `Output = Result<char, Utf16CharsError>` case.
#[derive(Debug, Clone)]
pub(crate) struct ErrorReportingHandler;

impl ErrorReportingHandler {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Utf16Handler for ErrorReportingHandler {
    type Output = Result<char, Utf16CharsError>;

    /// Map a single-code-unit UTF-16 sequence to `Output`.
    ///
    /// When `Output` is `char`, `unsafe { char::from_u32_unchecked(u32::from(bmp)) }`
    /// is the appropriate implementation.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `bmp` is not a surrogate.
    /// The callers in `utf16_iter` guarantee
    /// this, but this is `unsafe` in case the trait
    /// implementation is used with other callers. The
    /// implementation of this method is expected to be
    /// declared `#[inline(always)]` and to rely on this
    /// invariant without checking it on release builds.
    #[inline(always)]
    unsafe fn bmp(&self, bmp: u16) -> Self::Output {
        // SAFETY: The safety-usable invariant of this method
        // is the safety invariant of `bmp_to_char`.
        Ok(unsafe { bmp_to_char(bmp) })
    }

    /// Map a two-code-unit UTF-16 sequence to `Output`.
    ///
    /// When `Output` is `char`, `unsafe { surrogate_pair_to_char(high_surrogate, low_surrogate) }`
    /// is the appropriate implementation.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `high_surrogate` is
    /// is a high surrogate and `low_surrogate` is a low
    /// surrogate.
    /// The callers in `utf16_iter` guarantee
    /// this, but this is `unsafe` in case the trait
    /// implementation is used with other callers. The
    /// implementation of this method is expected to be
    /// declared `#[inline(always)]` and to rely on this
    /// invariant without checking it on release builds.
    #[inline(always)]
    unsafe fn surrogate_pair(&self, high_surrogate: u16, low_surrogate: u16) -> Self::Output {
        // SAFETY: The safety-usable invariant of this method
        // is the safety invariant of `surrogate_pair_to_char`.
        Ok(unsafe { surrogate_pair_to_char(high_surrogate, low_surrogate) })
    }

    /// Map a single UTF-16 error to `Output`.
    ///
    /// One unpaired surrogate
    ///
    /// When `Output` is `char`,
    /// `char::REPLACEMENT_CHARACTER`
    /// is the appropriate implementation. The provided
    /// implementation delegates to `bmp` by passing
    /// `char::REPLACEMENT_CHARACTER as u16`.
    ///
    /// The implementation of this method is expected to
    /// be declared `#[inline(always)]`.
    #[inline(always)]
    fn error(&self) -> Self::Output {
        Err(Utf16CharsError)
    }
}

crate::macros::named_iterators_from_no_argument_handler!(
    ErrorReportingHandler,
    Result<char, Utf16CharsError>,
    /// Iterator by `Result<char,Utf16CharsError>` over `&[u16]` that contains
    /// potentially-invalid UTF-16. There is exactly one `Utf16CharsError` per
    /// each unpaired surrogate.
    ,
    ErrorReportingUtf16Chars,
    /// Iterator by `Result<char,Utf16CharsError>` and their indices over `&[u16]` that contains
    /// potentially-invalid UTF-16. There is exactly one `Utf16CharsError` per
    /// each unpaired surrogate.
    ,
    ErrorReportingUtf16CharIndices,
);

#[cfg(test)]
mod tests {
    use crate::ErrorReportingUtf16Chars;
    use crate::Utf16CharsEx;

    #[test]
    fn test_boundaries() {
        assert!(ErrorReportingUtf16Chars::new([0xD7FFu16].as_slice())
            .map(|r| r.unwrap_or('\u{FFFD}'))
            .eq(core::iter::once('\u{D7FF}')));
        assert!(ErrorReportingUtf16Chars::new([0xE000u16].as_slice())
            .map(|r| r.unwrap_or('\u{FFFD}'))
            .eq(core::iter::once('\u{E000}')));
        assert!(ErrorReportingUtf16Chars::new([0xD800u16].as_slice())
            .map(|r| r.unwrap_or('\u{FFFD}'))
            .eq(core::iter::once('\u{FFFD}')));
        assert!(ErrorReportingUtf16Chars::new([0xDFFFu16].as_slice())
            .map(|r| r.unwrap_or('\u{FFFD}'))
            .eq(core::iter::once('\u{FFFD}')));
    }

    #[test]
    fn test_unpaired() {
        assert!(
            ErrorReportingUtf16Chars::new([0xD800u16, 0x0061u16].as_slice())
                .map(|r| r.unwrap_or('\u{FFFD}'))
                .eq([0xFFFDu16, 0x0061u16].as_slice().chars())
        );
        assert!(
            ErrorReportingUtf16Chars::new([0xDFFFu16, 0x0061u16].as_slice())
                .map(|r| r.unwrap_or('\u{FFFD}'))
                .eq([0xFFFDu16, 0x0061u16].as_slice().chars())
        );
    }

    #[test]
    fn test_unpaired_rev() {
        assert!(
            ErrorReportingUtf16Chars::new([0xD800u16, 0x0061u16].as_slice())
                .rev()
                .map(|r| r.unwrap_or('\u{FFFD}'))
                .eq([0xFFFDu16, 0x0061u16].as_slice().chars().rev())
        );
        assert!(
            ErrorReportingUtf16Chars::new([0xDFFFu16, 0x0061u16].as_slice())
                .rev()
                .map(|r| r.unwrap_or('\u{FFFD}'))
                .eq([0xFFFDu16, 0x0061u16].as_slice().chars().rev())
        );
    }

    #[test]
    fn test_paired() {
        assert!(
            ErrorReportingUtf16Chars::new([0xD83Eu16, 0xDD73u16].as_slice())
                .map(|r| r.unwrap_or('\u{FFFD}'))
                .eq(core::iter::once('🥳'))
        );
    }

    #[test]
    fn test_paired_rev() {
        assert!(
            ErrorReportingUtf16Chars::new([0xD83Eu16, 0xDD73u16].as_slice())
                .rev()
                .map(|r| r.unwrap_or('\u{FFFD}'))
                .eq(core::iter::once('🥳'))
        );
    }

    #[test]
    fn test_as_slice() {
        let mut iter = ErrorReportingUtf16Chars::new([0x0061u16, 0x0062u16].as_slice());
        let at_start = iter.as_slice();
        assert_eq!(iter.next(), Some(Ok('a')));
        let in_middle = iter.as_slice();
        assert_eq!(iter.next(), Some(Ok('b')));
        let at_end = iter.as_slice();
        assert_eq!(at_start.len(), 2);
        assert_eq!(in_middle.len(), 1);
        assert_eq!(at_end.len(), 0);
        assert_eq!(at_start[0], 0x0061u16);
        assert_eq!(at_start[1], 0x0062u16);
        assert_eq!(in_middle[0], 0x0062u16);
    }

    // Should be a static assert, but not taking a dependency for this.
    #[test]
    fn test_size() {
        assert_eq!(
            core::mem::size_of::<Option<<ErrorReportingUtf16Chars<'_> as Iterator>::Item>>(),
            core::mem::size_of::<Option<char>>()
        );
    }
}

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

#![no_std]

//! Provides iteration by `char` over `&[u16]` containing potentially-invalid
//! UTF-16 such that errors are replaced with the REPLACEMENT CHARACTER.
//!
//! The trait `Utf16CharsEx` provides the convenience method `chars()` on
//! byte slices themselves instead of having to use the more verbose
//! `Utf16Chars::new(slice)`.

use core::iter::FusedIterator;

#[inline(always)]
fn in_inclusive_range16(i: u16, start: u16, end: u16) -> bool {
    i.wrapping_sub(start) <= (end - start)
}

/// Iterator by `char` over `&[u16]` that contains
/// potentially-invalid UTF-16. See the crate documentation.
#[derive(Debug, Clone)]
pub struct Utf16Chars<'a> {
    remaining: &'a [u16],
}

impl<'a> Utf16Chars<'a> {
    #[inline(always)]
    /// Creates the iterator from a `u16` slice.
    pub fn new(bytes: &'a [u16]) -> Self {
        Utf16Chars::<'a> { remaining: bytes }
    }

    /// Views the current remaining data in the iterator as a subslice
    /// of the original slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &'a [u16] {
        self.remaining
    }
}

impl<'a> Iterator for Utf16Chars<'a> {
    type Item = char;

    #[inline(always)]
    fn next(&mut self) -> Option<char> {
        let (&first, tail) = self.remaining.split_first()?;
        self.remaining = tail;
        let surrogate_base = first.wrapping_sub(0xD800);
        if surrogate_base > (0xDFFF - 0xD800) {
            return Some(unsafe { char::from_u32_unchecked(u32::from(first)) });
        }
        if surrogate_base <= (0xDBFF - 0xD800) {
            if let Some((&low, tail_tail)) = self.remaining.split_first() {
                if in_inclusive_range16(low, 0xDC00, 0xDFFF) {
                    self.remaining = tail_tail;
                    return Some(unsafe {
                        char::from_u32_unchecked(
                            (u32::from(first) << 10) + u32::from(low)
                                - (((0xD800u32 << 10) - 0x10000u32) + 0xDC00u32),
                        )
                    });
                }
            }
        }
        Some('\u{FFFD}')
    }
}

impl<'a> DoubleEndedIterator for Utf16Chars<'a> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<char> {
        let (&last, head) = self.remaining.split_last()?;
        self.remaining = head;
        if !in_inclusive_range16(last, 0xD800, 0xDFFF) {
            return Some(unsafe { char::from_u32_unchecked(u32::from(last)) });
        }
        if in_inclusive_range16(last, 0xDC00, 0xDFFF) {
            if let Some((&high, head_head)) = self.remaining.split_last() {
                if in_inclusive_range16(high, 0xD800, 0xDBFF) {
                    self.remaining = head_head;
                    return Some(unsafe {
                        char::from_u32_unchecked(
                            (u32::from(high) << 10) + u32::from(last)
                                - (((0xD800u32 << 10) - 0x10000u32) + 0xDC00u32),
                        )
                    });
                }
            }
        }
        Some('\u{FFFD}')
    }
}

impl FusedIterator for Utf16Chars<'_> {}

/// Convenience trait that adds `chars()` method similar to
/// the one on string slices to byte slices.
pub trait Utf16CharsEx {
    fn chars(&self) -> Utf16Chars<'_>;
}

impl Utf16CharsEx for [u16] {
    /// Convenience method for creating an UTF-16 iterator
    /// for the slice.
    #[inline]
    fn chars(&self) -> Utf16Chars<'_> {
        Utf16Chars::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::Utf16CharsEx;

    #[test]
    fn test_boundaries() {
        assert!([0xD7FFu16]
            .as_slice()
            .chars()
            .eq(core::iter::once('\u{D7FF}')));
        assert!([0xE000u16]
            .as_slice()
            .chars()
            .eq(core::iter::once('\u{E000}')));
        assert!([0xD800u16]
            .as_slice()
            .chars()
            .eq(core::iter::once('\u{FFFD}')));
        assert!([0xDFFFu16]
            .as_slice()
            .chars()
            .eq(core::iter::once('\u{FFFD}')));
    }

    #[test]
    fn test_unpaired() {
        assert!([0xD800u16, 0x0061u16]
            .as_slice()
            .chars()
            .eq([0xFFFDu16, 0x0061u16].as_slice().chars()));
        assert!([0xDFFFu16, 0x0061u16]
            .as_slice()
            .chars()
            .eq([0xFFFDu16, 0x0061u16].as_slice().chars()));
    }

    #[test]
    fn test_unpaired_rev() {
        assert!([0xD800u16, 0x0061u16]
            .as_slice()
            .chars()
            .rev()
            .eq([0xFFFDu16, 0x0061u16].as_slice().chars().rev()));
        assert!([0xDFFFu16, 0x0061u16]
            .as_slice()
            .chars()
            .rev()
            .eq([0xFFFDu16, 0x0061u16].as_slice().chars().rev()));
    }

    #[test]
    fn test_paired() {
        assert!([0xD83Eu16, 0xDD73u16]
            .as_slice()
            .chars()
            .eq(core::iter::once('????')));
    }

    #[test]
    fn test_paired_rev() {
        assert!([0xD83Eu16, 0xDD73u16]
            .as_slice()
            .chars()
            .rev()
            .eq(core::iter::once('????')));
    }

    #[test]
    fn test_as_slice() {
        let mut iter = [0x0061u16, 0x0062u16].as_slice().chars();
        let at_start = iter.as_slice();
        assert_eq!(iter.next(), Some('a'));
        let in_middle = iter.as_slice();
        assert_eq!(iter.next(), Some('b'));
        let at_end = iter.as_slice();
        assert_eq!(at_start.len(), 2);
        assert_eq!(in_middle.len(), 1);
        assert_eq!(at_end.len(), 0);
        assert_eq!(at_start[0], 0x0061u16);
        assert_eq!(at_start[1], 0x0062u16);
        assert_eq!(in_middle[0], 0x0062u16);
    }
}

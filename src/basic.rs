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

/// The basic `Output = char` case.
#[derive(Debug, Clone)]
pub(crate) struct DefaultHandler;

impl DefaultHandler {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Utf16Handler for DefaultHandler {
    type Output = char;

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
        unsafe { bmp_to_char(bmp) }
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
        unsafe { surrogate_pair_to_char(high_surrogate, low_surrogate) }
    }
}

crate::macros::named_iterators_from_no_argument_handler!(
    DefaultHandler,
    char,
    /// Iterator by `char` over `&[u16]` that contains
    /// potentially-invalid UTF-16. See the crate documentation.
    ,
    Utf16Chars,
    /// Iterator by `char` and their indices over `&[u16]` that contains
    /// potentially-invalid UTF-16. See the crate documentation.
    ,
    Utf16CharIndices,
);

/// Convenience trait that adds `chars()` and `char_indices()` methods
/// similar to the ones on string slices to `u16` slices.
pub trait Utf16CharsEx {
    fn chars(&self) -> Utf16Chars<'_>;
    fn char_indices(&self) -> Utf16CharIndices<'_>;
}

impl Utf16CharsEx for [u16] {
    /// Convenience method for creating an UTF-16 iterator
    /// for the slice.
    #[inline]
    fn chars(&self) -> Utf16Chars<'_> {
        Utf16Chars::new(self)
    }
    /// Convenience method for creating a code unit index and
    /// UTF-16 iterator for the slice.
    #[inline]
    fn char_indices(&self) -> Utf16CharIndices<'_> {
        Utf16CharIndices::new(self)
    }
}

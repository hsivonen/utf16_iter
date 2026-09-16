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
use core::slice::Iter;

/// Mapping from the two kinds of code unit sequences or error
/// to output.
pub trait Utf16Handler {
    /// The per-scalar-value output type. (In the common case,
    /// this is `char`.)
    type Output;

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
    unsafe fn bmp(&self, bmp: u16) -> Self::Output;

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
    unsafe fn surrogate_pair(&self, high_surrogate: u16, low_surrogate: u16) -> Self::Output;

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
        // SAFETY: The constant is known not to be a surrogate.
        unsafe { self.bmp(char::REPLACEMENT_CHARACTER as u16) }
    }
}

/// Iterator by `char` over `&[u16]` that contains
/// potentially-invalid UTF-16. See the crate documentation.
#[derive(Debug, Clone)]
pub struct Utf16CharsWithHandler<'a, H>
where
    H: Utf16Handler,
{
    iter: Iter<'a, u16>,
    handler: H,
}

impl<'a, H> Utf16CharsWithHandler<'a, H>
where
    H: Utf16Handler,
{
    #[inline(always)]
    /// Creates the iterator from a `u16` slice.
    pub fn new(code_units: &'a [u16], handler: H) -> Self {
        Utf16CharsWithHandler::<'a, H> {
            iter: code_units.iter(),
            handler,
        }
    }

    /// Views the current remaining data in the iterator as a subslice
    /// of the original slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &'a [u16] {
        self.iter.as_slice()
    }

    /// Obtains a reference to the handler.
    #[inline(always)]
    pub fn handler(&self) -> &H {
        &self.handler
    }

    #[cold]
    fn error(&self) -> H::Output {
        self.handler.error()
    }

    #[cold]
    #[inline(never)]
    fn surrogate_next(&mut self, surrogate: u16) -> H::Output {
        if is_high_surrogate(surrogate) {
            let mut cloned_iter = self.iter.clone();
            if let Some(&low) = cloned_iter.next() {
                if is_low_surrogate(low) {
                    self.iter = cloned_iter;
                    // SAFETY: We have established that we have a surrogate
                    // pair.
                    return unsafe { self.handler.surrogate_pair(surrogate, low) };
                }
            }
        }
        self.error()
    }

    #[cold]
    #[inline(never)]
    fn surrogate_next_back(&mut self, surrogate: u16) -> H::Output {
        if is_low_surrogate(surrogate) {
            let mut cloned_iter = self.iter.clone();
            if let Some(&high) = cloned_iter.next_back() {
                if is_high_surrogate(high) {
                    self.iter = cloned_iter;
                    // SAFETY: We have established that we have a surrogate
                    // pair.
                    return unsafe { self.handler.surrogate_pair(high, surrogate) };
                }
            }
        }
        self.error()
    }
}

impl<'a, H> Iterator for Utf16CharsWithHandler<'a, H>
where
    H: Utf16Handler,
{
    type Item = H::Output;

    #[inline(always)]
    fn next(&mut self) -> Option<H::Output> {
        let first = *self.iter.next()?;
        if !is_surrogate(first) {
            // SAFETY: We have established that `first` is not a surrogate.
            return Some(unsafe { self.handler.bmp(first) });
        }
        Some(self.surrogate_next(first))
    }
}

impl<'a, H> DoubleEndedIterator for Utf16CharsWithHandler<'a, H>
where
    H: Utf16Handler,
{
    #[inline(always)]
    fn next_back(&mut self) -> Option<H::Output> {
        let last = *self.iter.next_back()?;
        if !is_surrogate(last) {
            // SAFETY: We have established that `last` is not a surrogate.
            return Some(unsafe { self.handler.bmp(last) });
        }
        Some(self.surrogate_next_back(last))
    }
}

impl<H> core::iter::FusedIterator for Utf16CharsWithHandler<'_, H> where H: Utf16Handler {}

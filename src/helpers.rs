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

#![allow(clippy::unusual_byte_groupings)]

/// `true` iff `u` is a surrogate (high or low).
#[inline(always)]
pub fn is_surrogate(u: u16) -> bool {
    (u & 0b111_110_00000_00000) == 0b110_110_00000_00000
}

/// `true` iff `u` is a high surrogate.
#[inline(always)]
pub fn is_high_surrogate(u: u16) -> bool {
    (u & 0b111_111_00000_00000) == 0b110_110_00000_00000
}

/// `true` iff `u` is a low surrogate.
#[inline(always)]
pub fn is_low_surrogate(u: u16) -> bool {
    (u & 0b111_111_00000_00000) == 0b110_111_00000_00000
}

/// Converts a non-surrogate to `char`.
///
/// # Safety
///
/// `bmp` must not be a surrogate.
///
/// # Panics
///
/// When debug assertions are enabled, panics if the safety invariant
/// is not upheld.
#[inline(always)]
pub unsafe fn bmp_to_char(bmp: u16) -> char {
    debug_assert!(!is_surrogate(bmp));
    // SAFETY: OK given the the safety invariant of this function is upheld.
    unsafe { char::from_u32_unchecked(u32::from(bmp)) }
}

/// Converts a surrogate pair to `char`.
///
/// # Safety
///
/// `high_surrogate` must be in the high surrogate range.
/// `low_surrogate` must be in the low surrogate range.
///
/// # Panics
///
/// When debug assertions are enabled, panics if the safety invariant
/// is not upheld.
#[inline(always)]
pub unsafe fn surrogate_pair_to_char(high_surrogate: u16, low_surrogate: u16) -> char {
    debug_assert!(is_high_surrogate(high_surrogate));
    debug_assert!(is_low_surrogate(low_surrogate));
    // SAFETY: This formulation results in a value in the `char` range if the
    // safety invariant of this function is upheld.
    unsafe {
        char::from_u32_unchecked(
            (u32::from(high_surrogate) << 10) + u32::from(low_surrogate)
                - (((0xD800u32 << 10) - 0x10000u32) + 0xDC00u32),
        )
    }
}

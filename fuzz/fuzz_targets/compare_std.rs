// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![no_main]
use libfuzzer_sys::fuzz_target;
use utf16_iter::Utf16Chars;
use core::char::{decode_utf16, REPLACEMENT_CHARACTER};

fuzz_target!(|data: &[u8]| {
    // SAFETY: Any bit pattern is valid for `u16`, so viewing arbitrary byte data as `u16` is OK.
    let (_, aligned, _) = unsafe { data.align_to::<u16>() };
    assert!(Utf16Chars::new(aligned).eq(decode_utf16(aligned.iter().copied()).map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))));
});

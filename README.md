# utf16_iter

[![crates.io](https://img.shields.io/crates/v/utf16_iter.svg)](https://crates.io/crates/utf16_iter)
[![docs.rs](https://docs.rs/utf16_iter/badge.svg)](https://docs.rs/utf16_iter/)

utf16_iter provides iteration by `char` over potentially-invalid UTF-16 `&[u16]`
such that UTF-16 errors are replaced with the REPLACEMENT CHARACTER.

This is a `no_std` crate.

## Licensing

TL;DR: `Apache-2.0 OR MIT`

Please see the file named
[COPYRIGHT](https://github.com/hsivonen/utf16_iter/blob/master/COPYRIGHT).

## Documentation

Generated [API documentation](https://docs.rs/utf16_iter/) is available
online.

## Release Notes

### 1.0.2

* Implemented `DoubleEndedIterator`.

### 1.0.1

* Added `as_slice()` method.

### 1.0.0

The initial release.

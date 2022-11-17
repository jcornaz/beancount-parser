# beancount-parser

[![License](https://img.shields.io/badge/license-Unlicense%20OR%20MIT-green)](#License)
[![Crates.io](https://img.shields.io/crates/v/beancount-parser)](https://crates.io/crates/beancount-parser)
[![Docs](https://docs.rs/beancount-parser/badge.svg)](https://docs.rs/beancount-parser)
[![Build](https://img.shields.io/github/workflow/status/jcornaz/beancount-parser/verify)](https://github.com/jcornaz/beancount-parser/actions/workflows/verify.yml)

A beancount file parser library for rust

## Goal

Parse a beancount file into a rust data structure

## Non goals

Do not verify beancount rules, such as "transaction must balance to zero", "account must be open", balance assertions, etc.

Do not provide any "business" logic to analyze or manipulate the ledger. No balance, no currency translation, etc.

## Cargo features

| Feature        | Description                                                                          |
|----------------|--------------------------------------------------------------------------------------|
| `rust_decimal` | Add conversion from `Value` into `Decimal`                                           |
| `unstable`     | New, unfinished and unstable API. <br /> **Unstable API is not considered as part os the public API. It may be broken, or even removed in a minor or patch release!** |

## MSRV

The minimum supported rust version is currently `1.60`.

It can be updated to a newer stable version when required, and that will not be considered a breaking change (it can happen in for a minor or patch release).


## Unlicense

This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org/>

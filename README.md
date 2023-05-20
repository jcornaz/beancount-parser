# beancount_parser_2

[![License](https://img.shields.io/crates/l/beancount_parser_2)](#Unlicense)
[![Crates.io](https://img.shields.io/crates/v/beancount_parser_2)](https://crates.io/crates/beancount_parser_2)
![rustc](https://img.shields.io/badge/rustc-1.69+-blue?logo=rust)
[![Docs](https://docs.rs/beancount_parser_2/badge.svg)](https://docs.rs/beancount_parser_2)

A [beancount] file parser library for rust

[beancount]: https://beancount.github.io/docs/index.html

This crate a a rewrite of [beancount-parser]. Not all features have been ported yet, and some of them may never be.

If you're missing something, please open an issue.

[beancount-parser]: https://github.com/jcornaz/beancount-parser

## Goal

Parse a [beancount file](https://beancount.github.io/docs/beancount_language_syntax.html) into a rust data structure


## Non goals

Do not verify beancount rules, such as "transaction must balance to zero", "account must be open", balance assertions, etc.

Do not provide any "business" logic to analyze or manipulate the ledger. No balance, no currency translation, etc.


## MSRV

The minimum supported rust version is currently `1.69`.

It can be updated to a newer stable version when required, and that will not be considered a breaking change.


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

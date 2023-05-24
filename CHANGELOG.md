# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).


## [Unreleased]

### Breaking changes

* The decimal type must now implement the `beancount_parser_2::Decimal` trait.
  There is a blanket implementation for all types that could be used as a decimal type,
  including `f64` and `rust_decimal::Decimal`


### New syntax supported

* Amount value expressions


### Documentation

* Document `Account` and related directives (`Open`, `Close`, `Balance` and `Pad`)


## [1.0.0-alpha.2] - 2023-05-22


### Breaking changes

* The type of the `price` field in `Posting` has changed to `Option<PostingPrice<'a, D>>`.
* The type `MetadataValue` no longer implements `Eq`

### Added

* Support for total price in posting (`@@` syntax)
* implement `Clone` for all types
* implement `Copy`, `Eq`, `Ord` and `Hash` on `Account` and `Currency`

### Documentation

* Write documentation for the `Transaction` and `Posting` types


## [1.0.0-alpha.1] - 2023-05-21

### Added

* Support for the pad directive
* Line number in error
* Line number in directive
* implement `std::error::Error` for `Error`

### Documentation

* Improve/write documentation for `parser`, `BeancountFile`, `Directive`, `Error` and `Date`


## [1.0.0-alpha.0] - 2023-05-20

### Supported beancount syntax

* Transaction
  * flag
  * payee and description
  * tags
  * postings
    * account
    * amount
    * price
    * cost
      * amount
      * date
* Price directive
* Open and close directives
* Balance assertion
* Commodity declaration
* Events
* Options
* Directive metadata (string values only)

[Unreleased]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.2...HEAD
[1.0.0-alpha.2]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.1...v1.0.0-alpha.2
[1.0.0-alpha.1]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.0...v1.0.0-alpha.1
[1.0.0-alpha.0]: https://github.com/jcornaz/beancount_parser_2/compare/...v1.0.0-alpha.0


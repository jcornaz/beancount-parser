# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).


## [Unreleased]

* Support links (^link) in transaction directive.

## [1.0.0-alpha.7] - 2023-06-11

### Improved

* Don't dump the full end-of-input when debugging the content of an error


## [1.0.0-alpha.6] - 2023-06-05

### Breaking changes

This Release reverts the change made on the last release which made the parser generic over the string type.
The function `parse` and all structs have now one less generic argment, and instead are bound to the lifetime of the input.

The change turned out to not be as beneficial as anticipated for a higher cost in complexity than anticipated.


#### Other breaking changes

* Make private the field `options` from `BeancountFile`. Use the new `option` getter instead.
* Make private the field `includes` from `BeancountFile`. Use the new `includes` iterator instead.
* Make private the field `directives` from `BeancountFile`. Use the new `directives` or `take_directives` methods instead.


### Dependencies

* Rust requirement (MSRV) bumped to 1.70


### Added

* `BeancountFile::option` getter


## [1.0.0-alpha.5] - 2023-05-31

### Breaking changes

The `parse` method and all structs are now generics over the string type `S`.
One must choose how to store strings in the results, with either `&str` or `String`.

With `parse::<&str, D>` the results will contains string slices from the input string.
This is very performant but a bit less convenient to use as one cannot move nor discard the input string until done working with the results.

With `parse::<String, D>` more memory will be allocated to copy strings from the input.
This is less performant but allow continue using the results after discarding the input string.


### Relaxed requirements

* Require less traits for the `Decimal` type, and extend the blanket implementation accordingly


### Documentation

* Document `Event` directive
* Document `Flag` directive


## [1.0.0-alpha.4] - 2023-05-27

### Breaking changes

* `MetadataValue` is now generic over the decimal type `D`
* `currencies` in the `Open` directive is now a `HashSet` instead of `Vec`


### New syntax supported

* `include` directive
* `pushtag` and `poptag` operations
* number and currency as metadata value


### Fixed

* Accept comment on `option` directive


### Documentation

* Document `Price`, `Amount`, `Currency` and `MetadataValue` types


## [1.0.0-alpha.3] - 2023-05-24

### Breaking changes

* The decimal type must now implement the `beancount_parser_2::Decimal` trait.
  There is a blanket implementation for all types that could be used as a decimal type,
  including `f64` and `rust_decimal::Decimal`


### New syntax supported

* Expression for amount value


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

[Unreleased]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.7...HEAD
[1.0.0-alpha.7]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.6...v1.0.0-alpha.7
[1.0.0-alpha.6]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.5...v1.0.0-alpha.6
[1.0.0-alpha.5]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.4...v1.0.0-alpha.5
[1.0.0-alpha.4]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.3...v1.0.0-alpha.4
[1.0.0-alpha.3]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.2...v1.0.0-alpha.3
[1.0.0-alpha.2]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.1...v1.0.0-alpha.2
[1.0.0-alpha.1]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.0...v1.0.0-alpha.1
[1.0.0-alpha.0]: https://github.com/jcornaz/beancount_parser_2/compare/beancount-parser-v1.16.0...v1.0.0-alpha.0


# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).


## [Unreleased]

### Fixed

* Accept negation of grouped expression (example: `-(2 * 3)`)


## [2.0.3] - 2024-01-18

### Fixed

* Accept comma (`,`) as a thousand separator in amounts
* Accept dot (`.`) in transaction links
* Accept escaped backslash (`\\`) in string


## [2.0.2] - 2024-01-17

### Fixed

* Accept escaped double quotes (`\"`) in strings


## [2.0.1] - 2024-01-07

### Fixed

Accept commodities ending with a number ([#63](https://github.com/jcornaz/beancount-parser/pull/63)). Thanks @doriath for the fix.

## [2.0.0] - 2023-07-11

### Breaking changes (since version 1.16)

This is a full rewrite of the parser from scratch.
Most types have been either removed or changed in an incompatible way.

Here are some key differences with the previous API:

* The types no longer have a lifetime parameter, so that they can be manipulated more easily, even after the 
  input string is discarded.
* Struct fields are public, which makes much easier to destruct and pattern-match the results of the parser.
* The directive type is now a struct (not an enum) that contains common directive data (date and metadata).
  And there is a new `DirectiveContent` enum which is roughly equivalent to the previous `Directive` enum.
* The parser is more strict about what beancount syntax is considered valid.

Check the [documentation](https://docs.rs/beancount-parser) to see how the new API looks like.


> **Note**
>
> You may depend on both the version 1 and 2 at the same time like this:
>
> ```toml
> [dependencies]
> # version 1, in rust code `use beancount_parser::...`
> beancount-parser = { version = "1" } 
> # version 2, in rust code `use beancount_parser_2::...`
> beancount-parser-2 = { package = "beancount-parser", version = "2.0.0" }
> ```


### Added (since version 2.0.0-beta.3)

* Cargo feature flag `miette`, which enable implementation of `miette::Diagnostic` for error types


## [2.0.0-beta.3] - 2023-07-09

### Added

* The `Amount` type is now exhaustive
* Support booking method in open account directive
* implement `std::error::Error` for `ConversionError`
* implement `FromStr` for `Directive<D>` where `D: Decimal`


## [2.0.0-beta.2] - 2023-07-08

### Added

* `Entry` enum which is one of `Directive`, `Option`, `Include`
* `parse_iter` which return an iterator over `Result<Entry<D>, Result>`
* implement `Clone` for `Error`
* implement `FromStr` for `BeancountFile<D>` where `D: Decimal`
* `read_files` to read and parse files from disk, following the include directives


## [2.0.0-beta.1] - 2023-07-01

I finally decided to re-use the old crate name `beancount-parser` instead of continuing with [beancount_parser_2](https://github.com/jcornaz/beancount_parser_2).

This release contains exactly the same functionality `beancount_parser_2` version `1.0.0-beta.3`.
`beancount_parser_2` will be discontinued and archived soon.

### Breaking changes

This is a full rewrite of the parser from scratch.
Most types have been either removed or changed in an incompatible way.
This new version is also more strict about what beancount syntax is considered valid.

Check the documentation to see how the new API looks like.

> **Note**
>
> You may depend on both the version 1 and 2 at the same time like this:
>
> ```toml
> [dependencies]
> # version 1, in rust code `use beancount_parser::...`
> beancount-parser = { version = "1" } 
> # version 2, in rust code `use beancount_parser_2::...`
> beancount-parser-2 = { package = "beancount-parser", version = "=2.0.0-beta.1" }
> ```


### Supported beancount syntax

* Transaction
  * flag
  * payee and description
  * tags
  * links
  * metadata
  * postings
    * account
    * amount
    * price
    * cost
      * amount
      * date
    * metadata
* Price directive
* Open and close directives
* Balance assertion
* Commodity declaration
* Events
* Options
* Directive metadata (string values only)


### Thank you

@doriath


## [1.16.2] - 2023-06-20

### Breaking changes *(in unstable API)*

#### remove `pest_parser`

The `pest` experiment was not conclusive. It is much slower than the `nom` implementation and the code is not significantly more maintainable in my opinion.
On top of that, the latest `pest` patch contained some breaking changes, causing new compile errors.
Therefore, this release completely removes the pest parser. 


## [1.16.1] - 2023-05-22

### Documentation

Announce new status of the project in readme: The project is now in "maintenance mode"


## [1.16.0] - 2023-05-14

### Breaking changes *(in unstable API)*

* remove `Error::line_number`

  (Instead, I am working on a wrapper type which would work for locating both errors and successfuly parsed directives)


### Added

* implement `Default` for `transaction::Flag` (The default value is `transaction::Flag::Cleared`)


### Changed

* Accept account types without more components. (e.g. `2023-05-13 open Assets` is now valid)
* Ignore trailing spaces after transaction/posting declarations
* Accept amount values with the unary operator `+` (e.g. `+42`)
* Accept spaces before comma in currency list of open directive


### Unstable API added

> **Warning**
>
> The unstable API requires the `unstable` feature flag. It is not considered part of the public API
> and is subject to breaking changes.

* Parse `option` directive
* Parse `event` directive
* Parse `commodity` directive


## [1.15.0] - 2023-03-07

### Added

* `AccountType` alias for `account::Type`
* **unstable**: `metadata` module

   which makes possible to pattern match the result of `Transaction::metadata`.

### Deprecated

* `Type`

    It has been made public by mistake. Use `AccountType` or `account::Type` instead.


### Documentation

* **readme**: Warning about unsupported feature flags


## [1.14.0] - 2023-02-24


### Features

* ~~provide `AccountType` from root module (equivalent of `account::Type`)~~ ([7602beb](https://github.com/jcornaz/beancount-parser/commit/7602beb03cac02a60b141cbda5c8e67078bc1561))

  By mistake this change re-exported `account::Type` as `Type` instead of `AccountType`.


## [1.13.0] - 2023-02-17


### Features

* pad directive ([e3034e7](https://github.com/jcornaz/beancount-parser/commit/e3034e76979cc7643ebd4dd9164bdf464946adf4))


### Documentation

* fix a broken link ([b943bd7](https://github.com/jcornaz/beancount-parser/commit/b943bd74e48915c397a9fe876cbc799848fda441))

## [1.12.0] - 2023-02-05


### Features

* implement `Display` for `account::Type` ([eb6bf10](https://github.com/jcornaz/beancount-parser/commit/eb6bf10b0717d05aeb43c3099430f58e4e7d6ceb))
* implement `Display` for `Account` ([b890620](https://github.com/jcornaz/beancount-parser/commit/b8906201fad79730604a98ead887f4cd95bba0fc))
* stabilize include directive ([60c7a2b](https://github.com/jcornaz/beancount-parser/commit/60c7a2bb191215d2d98e0f57ab1ca5c23f26fee9))

## [1.11.1] - 2023-02-03


### Features

* **unstable:** Add support for `include` directives ([#24](https://github.com/jcornaz/beancount-parser/issues/24)) ([e39c414](https://github.com/jcornaz/beancount-parser/commit/e39c414dc146ebf075ff1c4bdcfba8e43aaa4556))
* **unstable:** Make the include direcvie return a `&Path` ([480061a](https://github.com/jcornaz/beancount-parser/commit/480061a6a72bbd0eb5f86d3383ac26e331540ec8))

## [1.11.0] - 2023-01-27


### Features

* balance assertion directive ([#22](https://github.com/jcornaz/beancount-parser/issues/22)) ([bf4e5c6](https://github.com/jcornaz/beancount-parser/commit/bf4e5c67e9581417c0ac17a35619a08dbf7d4000))
* **unstable:** transaction metadata ([#23](https://github.com/jcornaz/beancount-parser/issues/23)) ([1a49c92](https://github.com/jcornaz/beancount-parser/commit/1a49c9264c785d7afd8763e0035480f52b75d193))


### Bug Fixes

* postings not parsed when transaction has metadata ([d6be280](https://github.com/jcornaz/beancount-parser/commit/d6be28047b1f30a3d6d8b377b54ab64305e990fa))


### Documentation

* **readme:** fix typo ([04b8693](https://github.com/jcornaz/beancount-parser/commit/04b86939f69899522d8233c07a2fe20e6de2c476))
* **readme:** link to beancount doc and typo fixes ([74872a6](https://github.com/jcornaz/beancount-parser/commit/74872a63ebdfe6bc43c3f669258eb2e074769721))

## [1.10.1] - 2023-01-23


### Bug Fixes

* failure to parse currencies with numbers or special chars ([#21](https://github.com/jcornaz/beancount-parser/issues/21)) ([d5f548e](https://github.com/jcornaz/beancount-parser/commit/d5f548ed4763324d4be7dc09ddb5590675ef721e))
* failure to parse terminal decimal points ([#19](https://github.com/jcornaz/beancount-parser/issues/19)) ([0e31ae4](https://github.com/jcornaz/beancount-parser/commit/0e31ae4687ce10c329872ec0ae5bf1f09712ec26))
* incorrect lot attributes parsing ([#20](https://github.com/jcornaz/beancount-parser/issues/20)) ([04c2d9a](https://github.com/jcornaz/beancount-parser/commit/04c2d9abc0dc5d0b3e88caa823c3d81bb7db7a2a))

## [1.10.0](https://github.com/jcornaz/beancount-parser/compare/v1.9.1...v1.10.0) (2023-01-11)


### Features

* extend the lifetime of items returned by `Account::components()` ([c56d5b1](https://github.com/jcornaz/beancount-parser/commit/c56d5b15e1d2516335e6b92a578715daa6effd44))


### Unstable API removed

* **unstable:** Remove transaction balancing and amount sum logic ([a6fb067](https://github.com/jcornaz/beancount-parser/commit/a6fb0678df8ea4cb0fff1581b00970f6d525c3d2))

## [1.9.1] - 2023-01-09


### Documentation

* **readme:** add rustc badge ([f0becaf](https://github.com/jcornaz/beancount-parser/commit/f0becaf1bf9aad95a0c04ada58122e6670cf65f4))
* **readme:** fix build badge url ([1cbdcc1](https://github.com/jcornaz/beancount-parser/commit/1cbdcc1d46d15fcd32a9c31a4a28a2547c02dae5))
* **readme:** remove build badge ([7df7d5a](https://github.com/jcornaz/beancount-parser/commit/7df7d5acd97970f53eb1023c31cacbe4d16b0ddd))
* **readme:** update rustc badge ([4a9a6c3](https://github.com/jcornaz/beancount-parser/commit/4a9a6c337b2bc31f3c7d69467fcbda8d14eb0edd))

## [1.9.0] - 2022-12-13


### Features

* Implement `Hash` for `Account` and `account::Type` ([#17](https://github.com/jcornaz/beancount-parser/issues/17)) ([84b3df0](https://github.com/jcornaz/beancount-parser/commit/84b3df007303e30cbc944dced683773d89c13581))

## [1.8.5 - 2022-11-21


### Features

* **unstable:** Amount sum ([dc68b38](https://github.com/jcornaz/beancount-parser/commit/dc68b38afae71cfc6a15956187c0f080aa88afe9))
* **unstable:** introduce `BalancedTransaction` type ([988f801](https://github.com/jcornaz/beancount-parser/commit/988f8017254028972949b2c3789c2c28b86011f5))

## [1.8.4] - 2022-11-20


### Bug Fixes

* **unstable:** fix error line-number after a multiline transaction ([93625ba](https://github.com/jcornaz/beancount-parser/commit/93625ba615b56512d9e906788774d83fb388ec6d))

## [1.8.3] - 2022-11-19


### Features

* **unstable:** line number on error type ([b2edd5c](https://github.com/jcornaz/beancount-parser/commit/b2edd5c0f33e6c414e55b23ff91f532e56f01272))


### Documentation

* **changelog:** add changelog file ([83b2800](https://github.com/jcornaz/beancount-parser/commit/83b28004bd10df2ad4b44d3a94e8f2b9f21e9807))
* **readme:** update MSRV ([257a6f2](https://github.com/jcornaz/beancount-parser/commit/257a6f2a048179a29c9c5cc5ec7faec81e99f60f))

## [1.8.2] - 2022-11-13

### Documentation

 - Show API behind feature flags on docs.rs

## [1.8.1] - 2022-11-10

### Documentation

 - Fix typos

## [1.8.0] - 2022-11-08

### New Features

 - `Date::new` constructor

## [1.7.0] - 2022-11-06

### New Features

 - `date` getter on `Directive`

## [1.6.0] - 2022-11-06

### New Features

 - Implement `Ord` for `Date` (fix #8)

## [1.5.0] - 2022-11-06

### New Features

 - Close directive (fix #7)

## [1.4.0] - 2022-11-05

### New Features

 - Currency constraints on open directive
 - Currency constraint on open directive
 - Unstable feature flag

## [1.3.1] - 2022-10-20

### Bug Fixes

 - Poptag being ignored
 - Pushtag being ignored

## [1.3.0] - 2022-10-15

### New Features

 - Make the `Price` type public

## [1.2.0] - 2022-10-14

### New Features

 - Open account directive

### Bug Fixes

 - Failure when parsing cost with date

## [1.1.0] - 2022-10-13

### Documentation

 - Fix small typos

### New Features

 - Parse transaction tags

 - Parse comment on price directive
 - Parse price directive

## [1.0.0] - 2022-10-11

### Documentation

 - Improve wording

### New Features

 - Conversion from `Value` into `rust_decimal::Decimal`
 - `Directive::into_transaction`

### Refactor (BREAKING)

 - Expose the transaction module
 - Expose the `amount` module

## v1.0.0-alpha.4 (2022-09-21)

### Documentation

 - Minor simplification of the root crate example

### Refactor (BREAKING)

 - Make `Transaction::postings` return a slice instead of a `Vec`

## [1.0.0-alpha.3] - 2022-09-16

### Documentation

 - Document the `Directive` type
 - Document the `Account` type
 - Document the `Transacion` and `Posting` types
 - Document the `Date` type

### New Features

 - Include date on `Transaction` type

### Refactor (BREAKING)

 - Iterate directives instead of tuples (date, directive)

## [1.0.0-alpha.2] - 2022-09-15

### Documentation

 - Minor rewordings of the readme
 - Document the main error type
 - Document the amount type
 - documentation of `Parser` type

### New Features

 - Conversion of `Value` into `f32`
 - Conversion of `Value` into `f64`

## [1.0.0-alpha.1] - 2022-09-14

### Documentation

 - Basic documentation

### New Features

 - Expression evaluation
 - directive::as_transaction
 - Ignore unknown directives
 - Ignore comment lines
 - Parse directives
 - `Parser` iterator type


[Unreleased]: https://github.com/jcornaz/beancount-parser/compare/v2.0.3...HEAD
[2.0.3]: https://github.com/jcornaz/beancount-parser/compare/v2.0.2...v2.0.3
[2.0.2]: https://github.com/jcornaz/beancount-parser/compare/v2.0.1...v2.0.2
[2.0.1]: https://github.com/jcornaz/beancount-parser/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/jcornaz/beancount-parser/compare/v2.0.0-beta.3...v2.0.0
[2.0.0-beta.3]: https://github.com/jcornaz/beancount-parser/compare/v2.0.0-beta.2...v2.0.0-beta.3
[2.0.0-beta.2]: https://github.com/jcornaz/beancount-parser/compare/v2.0.0-beta.1...v2.0.0-beta.2
[2.0.0-beta.1]: https://github.com/jcornaz/beancount-parser/compare/v1...v2.0.0-beta.1
[1.16.2]: https://github.com/jcornaz/beancount-parser/compare/v1.16.1...v1.16.2
[1.16.1]: https://github.com/jcornaz/beancount-parser/compare/v1.16.0...v1.16.1
[1.16.0]: https://github.com/jcornaz/beancount-parser/compare/v1.15.0...v1.16.0
[1.15.0]: https://github.com/jcornaz/beancount-parser/compare/v1.14.0...v1.15.0
[1.14.0]: https://github.com/jcornaz/beancount-parser/compare/v1.13.0...v1.14.0
[1.13.0]: https://github.com/jcornaz/beancount-parser/compare/v1.12.0...v1.13.0
[1.12.0]: https://github.com/jcornaz/beancount-parser/compare/v1.11.1...v1.12.0
[1.11.1]: https://github.com/jcornaz/beancount-parser/compare/v1.11.0...v1.11.1
[1.11.0]: https://github.com/jcornaz/beancount-parser/compare/v1.10.1...v1.11.0
[1.10.1]: https://github.com/jcornaz/beancount-parser/compare/v1.10.0...v1.10.1
[1.10.0]: https://github.com/jcornaz/beancount-parser/compare/v1.9.1...v1.10.0
[1.9.1]: https://github.com/jcornaz/beancount-parser/compare/v1.9.0...v1.9.1
[1.9.0]: https://github.com/jcornaz/beancount-parser/compare/v1.8.5...v1.9.0
[1.8.5]: https://github.com/jcornaz/beancount-parser/compare/v1.8.4...v1.8.5
[1.8.4]: https://github.com/jcornaz/beancount-parser/compare/v1.8.3...v1.8.4
[1.8.3]: https://github.com/jcornaz/beancount-parser/compare/v1.8.2...v1.8.3
[1.8.2]: https://github.com/jcornaz/beancount-parser/compare/v1.8.1...v1.8.2
[1.8.1]: https://github.com/jcornaz/beancount-parser/compare/v1.8.0...v1.8.1
[1.8.0]: https://github.com/jcornaz/beancount-parser/compare/v1.7.0...v1.8.0
[1.7.0]: https://github.com/jcornaz/beancount-parser/compare/v1.6.0...v1.7.0
[1.6.0]: https://github.com/jcornaz/beancount-parser/compare/v1.5.0...v1.6.0
[1.5.0]: https://github.com/jcornaz/beancount-parser/compare/v1.4.0...v1.5.0
[1.4.0]: https://github.com/jcornaz/beancount-parser/compare/v1.3.1...v1.4.0
[1.3.1]: https://github.com/jcornaz/beancount-parser/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/jcornaz/beancount-parser/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/jcornaz/beancount-parser/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/jcornaz/beancount-parser/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/jcornaz/beancount-parser/tree/v1.0.0

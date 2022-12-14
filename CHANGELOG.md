## [1.10.0](https://github.com/jcornaz/beancount-parser/compare/v1.9.1...v1.10.0) (2023-01-11)


### Features

* extend the lifetime of items returned by `Account::components()` ([c56d5b1](https://github.com/jcornaz/beancount-parser/commit/c56d5b15e1d2516335e6b92a578715daa6effd44))


### Unstable API removed

* **unstable:** Remove transaction balancing and amount sum logic ([a6fb067](https://github.com/jcornaz/beancount-parser/commit/a6fb0678df8ea4cb0fff1581b00970f6d525c3d2))

## [1.9.1](https://github.com/jcornaz/beancount-parser/compare/v1.9.0...v1.9.1) (2023-01-09)


### Documentation

* **readme:** add rustc badge ([f0becaf](https://github.com/jcornaz/beancount-parser/commit/f0becaf1bf9aad95a0c04ada58122e6670cf65f4))
* **readme:** fix build badge url ([1cbdcc1](https://github.com/jcornaz/beancount-parser/commit/1cbdcc1d46d15fcd32a9c31a4a28a2547c02dae5))
* **readme:** remove build badge ([7df7d5a](https://github.com/jcornaz/beancount-parser/commit/7df7d5acd97970f53eb1023c31cacbe4d16b0ddd))
* **readme:** update rustc badge ([4a9a6c3](https://github.com/jcornaz/beancount-parser/commit/4a9a6c337b2bc31f3c7d69467fcbda8d14eb0edd))

## [1.9.0](https://github.com/jcornaz/beancount-parser/compare/v1.8.5...v1.9.0) (2022-12-13)


### Features

* Implement `Hash` for `Account` and `account::Type` ([#17](https://github.com/jcornaz/beancount-parser/issues/17)) ([84b3df0](https://github.com/jcornaz/beancount-parser/commit/84b3df007303e30cbc944dced683773d89c13581))

## [1.8.5](https://github.com/jcornaz/beancount-parser/compare/v1.8.4...v1.8.5) (2022-11-21)


### Features

* **unstable:** Amount sum ([dc68b38](https://github.com/jcornaz/beancount-parser/commit/dc68b38afae71cfc6a15956187c0f080aa88afe9))
* **unstable:** introduce `BalancedTransaction` type ([988f801](https://github.com/jcornaz/beancount-parser/commit/988f8017254028972949b2c3789c2c28b86011f5))

## [1.8.4](https://github.com/jcornaz/beancount-parser/compare/v1.8.3...v1.8.4) (2022-11-20)


### Bug Fixes

* **unstable:** fix error line-number after a multiline transaction ([93625ba](https://github.com/jcornaz/beancount-parser/commit/93625ba615b56512d9e906788774d83fb388ec6d))

## [1.8.3](https://github.com/jcornaz/beancount-parser/compare/v1.8.2...v1.8.3) (2022-11-19)


### Features

* **unstable:** line number on error type ([b2edd5c](https://github.com/jcornaz/beancount-parser/commit/b2edd5c0f33e6c414e55b23ff91f532e56f01272))


### Documentation

* **changelog:** add changelog file ([83b2800](https://github.com/jcornaz/beancount-parser/commit/83b28004bd10df2ad4b44d3a94e8f2b9f21e9807))
* **readme:** update MSRV ([257a6f2](https://github.com/jcornaz/beancount-parser/commit/257a6f2a048179a29c9c5cc5ec7faec81e99f60f))

# Changelog

## v1.8.2 (2022-11-13)

### Documentation

 - Show API behind feature flags on docs.rs

## v1.8.1 (2022-11-10)

### Documentation

 - Fix typos

## v1.8.0 (2022-11-08)

### New Features

 - `Date::new` constructor

## v1.7.0 (2022-11-06)

### New Features

 - `date` getter on `Directive`

## v1.6.0 (2022-11-06)

### New Features

 - Implement `Ord` for `Date` (fix #8)

## v1.5.0 (2022-11-06)

### New Features

 - Close directive (fix #7)

## v1.4.0 (2022-11-05)

### New Features

 - Currency constraints on open directive
 - Currency constraint on open directive
 - Unstable feature flag

## v1.3.1 (2022-10-20)

### Bug Fixes

 - Poptag being ignored
 - Pushtag being ignored

## v1.3.0 (2022-10-15)

### New Features

 - Make the `Price` type public

## v1.2.0 (2022-10-14)

### New Features

 - Open account directive

### Bug Fixes

 - Failure when parsing cost with date

## v1.1.0 (2022-10-13)

### Documentation

 - Fix small typos

### New Features

 - Parse transaction tags

 - Parse comment on price directive
 - Parse price directive

## v1.0.0 (2022-10-11)

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

## v1.0.0-alpha.3 (2022-09-16)

### Documentation

 - Document the `Directive` type
 - Document the `Account` type
 - Document the `Transacion` and `Posting` types
 - Document the `Date` type

### New Features

 - Include date on `Transaction` type

### Refactor (BREAKING)

 - Iterate directives instead of tuples (date, directive)

## v1.0.0-alpha.2 (2022-09-15)

### Documentation

 - Minor rewordings of the readme
 - Document the main error type
 - Document the amount type
 - documentation of `Parser` type

### New Features

 - Conversion of `Value` into `f32`
 - Conversion of `Value` into `f64`

## v1.0.0-alpha.1 (2022-09-14)

### Documentation

 - Basic documentation

### New Features

 - Expression evaluation
 - directive::as_transaction
 - Ignore unknown directives
 - Ignore comment lines
 - Parse directives
 - `Parser` iterator type

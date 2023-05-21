# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).


## [Unreleased]

### Added

* Support for the pad directive
* Line number in error
* Line number in directive
* implement `std::error::Error` for `Error`


### Documentation improvements

* Some typo fixes and wordings improved


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

[Unreleased]: https://github.com/jcornaz/beancount_parser_2/compare/v1.0.0-alpha.0...HEAD
[1.0.0-alpha.0]: https://github.com/jcornaz/beancount_parser_2/compare/...v1.0.0-alpha.0


# How to contribute

## Ask for help, propose a feature request a feature or report a bug

Use the [discussions](https://github.com/jcornaz/beancount-parser/discussions) to ask questions, share/discuss idea of features and even show-case what cool think you made with this project!

Use the [issues](https://github.com/jcornaz/beancount-parser/issues) to report any issue you have (bug or missing feature). Make sure to explain why you need something.


## Work with the sources

1. Make sure you have latest stable rust toolchain installed (https://rustup.rs)
2. Make sure you have [just](https://just.systems/man/en/chapter_4.html) installed
3. Run `just -l` to see the list of available recipes

## Coding standards

### Tests

***This is a test-driven project!*** Every new feature and bug fixes must come with tests.

### Stable API

***Do not break public API!***

(See https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md to understand what constitutes a breaking change)

Favor, creating new types/functions, that could be used in place of the old ones.
Eventually we may deprecate the old abstractions and hide it from the doc.

When extending the API, make sure it can last. In particular:
* Don't make anything public unless you actually need it (and **Never expose struct fields!**)
* Use `#[non_exhaustive]` for public enums and unit structs
* Don't eagerly implement traits that are not yet needed or related to the use-case
* Don't leek private dependencies in the API
* Public dependencies crates must be optional

New, unstable or incomplete features may be gated behind a `unstable` cargo flag until stabilized/finished.

> **Note**
>
> The API may eventually be broken. (in a new major version)
> But only if the breanking change makes the API significantly better than any non-breaking change we can think of.


## Open a pull request

Don't be afraid of small steps. I'd rather review 5 tiny pull-requests than 1 big. It is fine to have a PR that only partilally implement a feature. We can gate the feature behind the `unstable` feature flag until it is complete.

But the no matter how small the PR is, it must have automated tests!

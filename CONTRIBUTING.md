# How to contribute

## Ask for help, propose a feature request a feature or report a bug

Feel free to create an [issue](https://github.com/jcornaz/beancount-parser/issues) or open a [discussion](https://github.com/jcornaz/beancount-parser/discussions)


## Choose an issue to work on

You don't *need* to find an open issue to contribute a PR. But it is better to make sure the change is actually desirable.

I assign myself to issues when I am working on them. So you can safely pick any
[unassigned issue](https://github.com/jcornaz/beancount-parser/issues?utf8=%E2%9C%93&q=is%3Aissue+is%3Aopen+no%3Aassignee+).

You may (but don't have to) write a message in the issue to say you are working on it.

## Build from source

1. Make sure you have latest stable rust toolchain installed (https://rustup.rs)
2. Make sure you have [just](https://just.systems/man/en/chapter_4.html) installed
3. Run `just install-dev-deps`
4. Look at the list of `just` recipes `just -l`

## Coding standards

### Tests

**This is a test-driven project**. Every new feature and bug fixes must come with tests.
If you need help to write a test, ask me.

When contributing, you should see the tests you write as the most important and valuable part of your contribution.
When I review, most of my attention is for the tests. If the tests are good and exhaustive, then it doesn't matter much how clean the implementation is, because it already has the two most important poroperties: 1) It works, 2) It can safely be refactored later.

### API stability

#### Add stable API

When writting new code make sure you don't expose any unecessary technical details:
* Never expose struct fields!
* Use `#[non_exhaustive]` for enums and unit structs
* Be very carefull with public enums. In doubt keep them private.
* Don't expose types and functions that do no need to be public
* Don't eagerly implement traits that are not yet needed or related to the use-case
  * except for `Debug`, `Clone`, `Eq`, `PartialEq` and `Default` that may be implemented eagerly when it makes sense
  * note that if a `new()` constuctor does not make sense, `Default` should **not** be implemented
  * in case of doubt, don't implement what you don't need. Don't worry, we can add it later.
* Avoid promissing too much in return types. (e.g. `&[T]` are better that `&Vec<T>`)
* Don't leek private dependencies in the API.
* Integration with other crates must be optional.

New, unstable or incomplete features may be gated behind a `unstable-` cargo flag until stabilized.

#### Do not break existing API

Do not break public API. (See https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md to understand what constitutes a breaking change)

Instead, create a new API. Eventually we may deprecate the old one and hide it from the doc.

The API *may* eventually be broken. But I want to avoid that for as long as possible (forever would be perfect).
I see a breaking change as good only if it makes future breaking changes less likely to be needed. 
For example, to make struct field privates is a an acceptable breaking change.

If you don't see how to improve an API without breaking it, ask me.

## Open a pull request

Don't be afraid of small steps. I'd rather review 5 tiny pull-requests than 1 big.

But to be merged a pull-request needs to be in state ready for release:
* New features and Bug fixes must comes with automated tests
* The build must pass
* The documentation must be up-to-date

It is fine to have a PR that only partilally implement a feature.
But the implemented part must be tested, and the feature can be either hidden or gated behind an `unstable-*` feature flag.

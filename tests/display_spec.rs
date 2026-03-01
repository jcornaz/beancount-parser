#![allow(missing_docs)]

use rstest::rstest;

use beancount_parser::parse;

#[rstest]
#[case(r#"2020-01-01 ! "" "narration""#)]
#[case(r#"2020-01-01 ! "payee" "narration""#)]
#[case(r#"2020-01-01 ! "narration""#)]
#[case(r#"2020-01-01 * "Store" "Groceries" #food ^receipt"#)]
#[case(
    r#"2020-01-01 * "" "Test"
  Assets:Cash 100 USD
  Income:Salary"#
)]
#[case("2023-01-01 price USD 0.92 EUR")]
#[case("2020-01-01 balance Assets:Cash 123 EUR")]
#[case("2020-01-01 balance Assets:Cash 123 ~ 0.1 EUR")]
#[case("2020-01-01 open Assets:Cash USD")]
// #[case("2020-01-01 open Assets:Cash USD,EUR")] disabled due to non-stable output
#[case("2020-01-01 open Assets:Cash USD \"method\"")]
#[case("2020-01-01 open Assets:Cash")]
#[case("2023-12-31 close Assets:Cash")]
#[case("2020-01-01 pad Assets:Cash Equity:Opening")]
#[case("2020-01-01 commodity USD")]
#[case("2020-01-01 event \"location\" \"home\"")]
/*#[case(
    r#"2020-01-01 close Assets:Cash
  note: "Account closed"
  count: 42
  currency: USD"#
)] disabled due to non-stable output*/
#[case(
    r#"2020-01-01 * "Store" "Groceries" #food ^receipt
  Assets:Cash -50 USD
  Expenses:Groceries 50 USD
    category: "essentials""#
)]
fn display_roundtrip(#[case] input: &str) {
    let parsed = parse::<f64>(input).unwrap_or_else(|_| panic!("Failed to parse:\n  {}", input));
    let directive = &parsed.directives[0];
    let displayed = directive.to_string();

    assert_eq!(input, displayed, "Round-trip failed");
}

#[rstest]
#[case(
    // floats get normalized
    "2020-01-01 balance Assets:Cash 100.50 USD",
    "2020-01-01 balance Assets:Cash 100.5 USD"
)]
// transactions syntax is normalized
#[case(r#"2020-01-01 txn "Narration""#, r#"2020-01-01 txn "Narration""#)]
#[case(
    r#"2020-01-01   balance    Assets:Cash  10  USD"#,
    r#"2020-01-01 balance Assets:Cash 10 USD"#
)]
fn directive_display_changes(#[case] input: &str, #[case] expected: &str) {
    let result = parse::<f64>(input).unwrap();
    let directive = &result.directives[0];
    assert_eq!(directive.to_string(), expected);
}

#[rstest]
#[case(
    r#"2020-01-01 * ""
  Assets:Cash   100 USD"#,
    "Assets:Cash 100 USD"
)]
#[case(
    r#"2020-01-01 * ""
  Income:Salary"#,
    "Income:Salary"
)]
#[case(
    r#"2020-01-01 * ""
  ! Assets:Cash   100 USD"#,
    "! Assets:Cash 100 USD"
)]
#[case(
    r#"2020-01-01 * ""
  Assets:Cash   10 STOCK @ 50.00 USD"#,
    "Assets:Cash 10 STOCK @ 50 USD"
)]
#[case(
    r#"2020-01-01 * ""
  Assets:Cash   10 STOCK @@ 500.00 USD"#,
    "Assets:Cash 10 STOCK @@ 500 USD"
)]
#[case(
    r#"2020-01-01 * ""
  Assets:Cash   10 STOCK {50.00 USD}"#,
    "Assets:Cash 10 STOCK {50 USD}"
)]
#[case(
    r#"2020-01-01 * ""
  Assets:Cash   10 STOCK {2022-01-01, 50.00 USD}"#,
    "Assets:Cash 10 STOCK {2022-01-01, 50 USD}"
)]
fn posting_display(#[case] input: &str, #[case] expected: &str) {
    let result = parse::<f64>(input).unwrap();
    let directive = &result.directives[0];
    let output = directive.to_string();
    let lines: Vec<&str> = output.lines().collect();
    let posting_line = lines[1].trim_start();
    assert_eq!(posting_line, expected);
}

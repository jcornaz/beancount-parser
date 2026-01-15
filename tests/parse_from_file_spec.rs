#![allow(missing_docs)]

use std::path::PathBuf;

use rstest::rstest;

use beancount_parser::{BeancountFile, Entry};

#[rstest]
#[case("comments.beancount", 0)]
#[case("simple.beancount", 16)]
#[case("official.beancount", 1714)]
#[case("includes.beancount", 1731)]
fn can_parse_example_files(#[case] file_name: &str, #[case] expected_directive_count: usize) {
    let mut path: PathBuf = "./tests/samples".into();
    path.push(file_name);
    {
        let mut file = BeancountFile::<f64>::default();
        beancount_parser::read_files_v2([path.clone()], |entry| file.extend(Some(entry))).unwrap();
        assert_eq!(file.directives.len(), expected_directive_count);
    }
    {
        let mut file = BeancountFile::<f64>::default();
        file.extend(beancount_parser::read_files_to_vec([path]).unwrap());
        assert_eq!(file.directives.len(), expected_directive_count);
    }
}

#[rstest]
#[case("comments.beancount", 0)]
#[case("simple.beancount", 16)]
#[case("official.beancount", 1714 + 2)] // include entries are emitted as well
#[case("includes.beancount", 1731 + 2)]
fn can_parse_example_files_iter(#[case] file_name: &str, #[case] expected_directive_count: usize) {
    let mut path: PathBuf = "./tests/samples".into();
    path.push(file_name);
    let directives = beancount_parser::read_files_iter([path])
        .collect::<Result<Vec<Entry<f64>>, _>>()
        .unwrap();
    assert_eq!(directives.len(), expected_directive_count);
}

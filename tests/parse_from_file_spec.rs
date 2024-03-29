use std::path::PathBuf;

use rstest::rstest;

use beancount_parser::BeancountFile;

#[rstest]
#[case("comments.beancount", 0)]
#[case("simple.beancount", 16)]
#[case("official.beancount", 2154)]
fn can_parse_example_files(#[case] file_name: &str, #[case] expected_directive_count: usize) {
    let mut path: PathBuf = "./tests/samples".into();
    path.push(file_name);
    let mut file = BeancountFile::<f64>::default();
    beancount_parser::read_files([path], |entry| file.extend(Some(entry))).unwrap();
    assert_eq!(file.directives.len(), expected_directive_count);
}

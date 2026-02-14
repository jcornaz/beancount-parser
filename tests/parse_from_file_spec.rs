#![allow(missing_docs)]

use std::path::PathBuf;

use rstest::rstest;

use beancount_parser::BeancountFile;

#[rstest]
#[case("comments.beancount", 0, 0)]
#[case("simple.beancount", 16, 0)]
#[case("official.beancount", 1714, 0)]
#[case("includes.beancount", 1731, 6)]
fn can_parse_example_files(
    #[case] file_name: &str,
    #[case] expected_directive_count: usize,
    #[case] expected_include_count: usize,
) {
    let mut path: PathBuf = "./tests/samples".into();
    path.push(file_name);
    {
        let mut file = BeancountFile::<f64>::default();
        beancount_parser::read_files_v2([path.clone()], |entry| file.extend(Some(entry))).unwrap();
        assert_eq!(file.directives.len(), expected_directive_count);
        assert_eq!(file.includes.len(), expected_include_count);
    }
    {
        let mut file = BeancountFile::<f64>::default();
        file.extend(beancount_parser::read_files_to_vec([path.clone()]).unwrap());
        assert_eq!(file.directives.len(), expected_directive_count);
        assert_eq!(file.includes.len(), expected_include_count);
    }
    {
        let file = BeancountFile::<f64>::read_files([path]).unwrap();
        assert_eq!(file.directives.len(), expected_directive_count);
        assert_eq!(file.includes.len(), expected_include_count);
    }
}

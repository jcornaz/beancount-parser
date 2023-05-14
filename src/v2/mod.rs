pub fn parse(_input: &str) -> Result<Vec<Directive>, Error> {
    Ok(Vec::new())
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Error;

#[derive(Debug)]
#[non_exhaustive]
pub struct Directive;

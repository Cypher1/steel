#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Symbol<'a> {
    // TODO: Intern strings
    // TODO: Locations
    pub name: &'a str,
}

/*
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Node<'a> {
    Symbol(Symbol<'a>),
}
*/

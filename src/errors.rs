pub enum MetaCommandError {
    Unrecognized,
}

pub enum ParseError {
    UnrecognizedStatement,
    SyntaxError,
    NegativeId,
    StringTooLong,
}

pub enum ExecuteError {
    TableFull,
    DuplicateKey,
}

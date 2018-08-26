use std;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {         

    }

    foreign_links {
        IoError(std::io::Error);
    }
}
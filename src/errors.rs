use std::io::Error as ioError;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {         

    }

    foreign_links {
        IoError(ioError);
    }
}
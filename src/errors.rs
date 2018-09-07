use pcap::Error as pcapError;
use std::io::Error as ioError;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IoError(ioError);
        PacpError(pcapError);
    }   
}
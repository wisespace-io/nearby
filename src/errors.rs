use serde_json;
use reqwest;
use ctrlc::Error as ctrlcError;
use pcap::Error as pcapError;
use std::io::Error as ioError;
use std::num::ParseIntError as parseIntError;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IoError(ioError);
        PacpError(pcapError);
        Json(serde_json::Error);
        ReqError(reqwest::Error);
        ParseIntError(parseIntError);
        CtrlcError(ctrlcError);
    }   
}
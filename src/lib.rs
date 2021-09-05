use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use lazy_static::lazy_static;
use regex::Regex;
use urlencoding::{decode, encode};

lazy_static! {
    // We don't want to percent encode the colon on a Windows drive letter.
    static ref WINDOWS_DRIVE: Regex = Regex::new(r"[a-zA-Z]:").unwrap();
    static ref SEPARATOR: Regex = Regex::new(r"[/\\]").unwrap();
}

static FORWARD_SLASH: &str = "/";

#[derive(Debug)]
pub struct UTFDecodeError {
    details: String,
}

impl UTFDecodeError {
    fn new(msg: &str) -> UTFDecodeError {
        UTFDecodeError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for UTFDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for UTFDecodeError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub fn encode_file_component(path_part: &str) -> String {
    // If it's a separator char or a Windows drive return
    // as-is.
    if SEPARATOR.is_match(path_part) || WINDOWS_DRIVE.is_match(path_part) {
        path_part.to_owned()
    } else {
        encode(path_part).to_string()
    }
}

pub trait PathBufUrlExt {
    fn to_file_url(&self) -> Result<String, UTFDecodeError>;
    fn from_file_url(file_url: &str) -> Result<PathBuf, FromUtf8Error>;
}

impl PathBufUrlExt for PathBuf {
    fn to_file_url(&self) -> Result<String, UTFDecodeError> {
        let path_parts: Result<PathBuf, UTFDecodeError> = self
            .components()
            .map(|part| match part.as_os_str().to_str() {
                Some(part) => Ok(encode_file_component(part)),
                None => Err(UTFDecodeError::new("File path not UTF-8 compatible!")),
            })
            .collect();

        match path_parts {
            // Unwrap shouldn't fail here since everything should be properly decoded.
            Ok(parts) => Ok(format!("file://{}", parts.to_str().unwrap())),
            Err(e) => Err(e),
        }
    }

    fn from_file_url(file_url: &str) -> Result<PathBuf, FromUtf8Error> {
        SEPARATOR
            .split(file_url)
            .enumerate()
            .map(|(i, url_piece)| {
                if i == 0 && url_piece == "file:" {
                    // File url should always be abspath
                    Ok(String::from(FORWARD_SLASH))
                } else {
                    let dec_str = decode(url_piece);
                    match dec_str {
                        Ok(decoded) => Ok(decoded.into_owned()),
                        Err(e) => Err(e),
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::PathBufUrlExt;
    use std::path::PathBuf;

    #[test]
    fn basic_pathbuf_to_url() {
        let p = PathBuf::from("/some/file.txt");
        let url = p.to_file_url().unwrap();
        let s = url.as_str();
        assert_eq!(s, "file:///some/file.txt");
    }

    #[test]
    fn oddball_pathbuf_to_url() {
        let p = PathBuf::from("/gi>/some & what.whtvr");
        let url = p.to_file_url().unwrap();
        let s = url.as_str();
        assert_eq!(s, "file:///gi%3E/some%20%26%20what.whtvr");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_pathbuf_to_url() {
        let p = PathBuf::from(r"c:\WINDOWS\clock.avi");
        let url = p.to_file_url().unwrap();
        let s = url.as_str();
        assert_eq!(s, "file:///c:/WINDOWS/clock.avi");
    }

    #[test]
    fn basic_pathbuf_from_url() {
        let one = PathBuf::from("/some/file.txt");
        let two = PathBuf::from_file_url("file:///some/file.txt").unwrap();
        assert_eq!(one, two);
    }

    #[test]
    fn oddball_pathbuf_from_url() {
        let one = PathBuf::from_file_url("file:///gi%3E/some%20%26%20what.whtvr").unwrap();
        let two = PathBuf::from("/gi>/some & what.whtvr");
        assert_eq!(one, two);
    }
}

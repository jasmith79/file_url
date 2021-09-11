//! file_url
//!
//! Makes it easier to Path/PathBuf to/from file URLs.
//!
//! Author: Jared Adam Smith
//! license: MIT
//! © 2021
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;
use urlencoding::{decode, encode};

lazy_static! {
    // We don't want to percent encode the colon on a Windows drive letter.
    static ref WINDOWS_DRIVE: Regex = Regex::new(r"[a-zA-Z]:").unwrap();
    static ref SEPARATOR: Regex = Regex::new(r"[/\\]").unwrap();
}

static FORWARD_SLASH: &str = "/";

/// Error for file paths that don't decode to
/// valid UTF-8 strings.
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

/// Percent-encodes the path component. Ignores
/// Microsoft Windows drive letters and separator
/// characters.
///
/// # Example:
/// ```
/// use file_url::encode_file_component;
///
/// let enc = encode_file_component("some file.txt");
/// assert_eq!(enc, "some%20file.txt");
///
/// let windows_drive = encode_file_component("C:");
/// assert_eq!(windows_drive, "C:");
/// ```
pub fn encode_file_component(path_part: &str) -> Cow<str> {
    // If it's a separator char or a Windows drive return
    // as-is.
    if SEPARATOR.is_match(path_part) || WINDOWS_DRIVE.is_match(path_part) {
        Cow::from(path_part)
    } else {
        encode(path_part)
    }
}

/// Turns a file URL into a PathBuf. Note that because
/// `std::path::PathBuf` is backed by a `std::ffi::OsString`
/// the result is platform-dependent, i.e. Microsoft Windows
/// paths will not be properly processed on Unix-like systems
/// and vice-versa. Also note that because the bytes of a
/// valid file path can be non-UTF8 we have to return a
/// Result in case the string decode fails.
///
/// # Examples:
/// ```
/// use std::path::PathBuf;
/// use file_url::file_url_to_pathbuf;
///
/// let p_buf = file_url_to_pathbuf("file:///foo/bar%20baz.txt").unwrap();
/// assert_eq!(p_buf, PathBuf::from("/foo/bar baz.txt"));
/// ```
pub fn file_url_to_pathbuf(file_url: &str) -> Result<PathBuf, FromUtf8Error> {
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

/// Method for converting std::path::PathBuf and
/// `std::path::Path` to a file URL.
pub trait PathFileUrlExt {
    /// Assuming a PathBuf or Path is valid UTF8, converts
    /// to a file URL as an owned String.
    fn to_file_url(&self) -> Result<String, UTFDecodeError>;
}

/// Method for constructing a `std::path::PathBuf` from a file URL.
pub trait PathFromFileUrlExt<PathBuf> {
    /// Constructs a PathBuf from the supplied &str.
    fn from_file_url(file_url: &str) -> Result<PathBuf, FromUtf8Error>;
}

impl PathFileUrlExt for Path {
    fn to_file_url(&self) -> Result<String, UTFDecodeError> {
        let path_parts: Result<PathBuf, UTFDecodeError> = self
            .components()
            .map(|part| match part.as_os_str().to_str() {
                Some(part) => Ok(encode_file_component(part).to_string()),
                None => Err(UTFDecodeError::new("File path not UTF-8 compatible!")),
            })
            .collect();

        match path_parts {
            // Unwrap shouldn't fail here since everything should be properly decoded.
            Ok(parts) => Ok(format!("file://{}", parts.to_str().unwrap())),
            Err(e) => Err(e),
        }
    }
}

impl PathFromFileUrlExt<PathBuf> for PathBuf {
    fn from_file_url(file_url: &str) -> Result<PathBuf, FromUtf8Error> {
        file_url_to_pathbuf(file_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn basic_path_to_url() {
        let one = Path::new("/foo/bar.txt").to_file_url().unwrap();
        let two = "file:///foo/bar.txt";
        assert_eq!(one, two);
    }
}

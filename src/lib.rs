//! file_url
//!
//! Makes it easier to Path/PathBuf to/from file URLs.
//!
//! Author: Jared Adam Smith
//! license: MIT
//! © 2021
use std::ffi::OsString;
#[cfg(target_family = "unix")]
use std::os::unix::ffi::OsStringExt;
#[cfg(target_family = "windows")]
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;

mod os_str_from_bytes;
mod percent_ops;

use crate::percent_ops::{decode_path_component, encode_path_component};

lazy_static! {
    static ref SEPARATOR: Regex = Regex::new(r"[/\\]").unwrap();
}

static FORWARD_SLASH: &str = "/";

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
/// let p_buf = file_url_to_pathbuf("file:///foo/bar%20baz.txt");
/// assert_eq!(p_buf, PathBuf::from("/foo/bar baz.txt"));
/// ```
pub fn file_url_to_pathbuf(file_url: &str) -> PathBuf {
    SEPARATOR
        .split(file_url)
        .enumerate()
        .map(|(i, url_piece)| {
            if i == 0 && url_piece == "file:" {
                // File url should always be abspath
                OsString::from(FORWARD_SLASH)
            } else {
                let decoded = decode_path_component(url_piece).as_os_str();
                decoded
            }
        })
        .collect()
}

/// Method for converting std::path::PathBuf and
/// `std::path::Path` to a file URL.
pub trait PathFileUrlExt {
    /// Assuming a PathBuf or Path is valid UTF8, converts
    /// to a file URL as an owned String.
    fn to_file_url(&self) -> String;
}

/// Method for constructing a `std::path::PathBuf` from a file URL.
pub trait PathFromFileUrlExt<PathBuf> {
    /// Constructs a PathBuf from the supplied &str.
    fn from_file_url(file_url: &str) -> PathBuf;
}

impl PathFileUrlExt for Path {
    fn to_file_url(&self) -> String {
        let p_buff = self
            .components()
            .into_iter()
            .enumerate()
            .map(|(i, component)| {
                let s: OsString = component.as_os_str().to_owned();
                if i == 0 && s == FORWARD_SLASH {
                    String::from(FORWARD_SLASH)
                } else {
                    encode_path_component(&s.into_vec())
                }
            })
            .collect::<PathBuf>();

        // The unwrap is safe here, we constructed the PathBuf from Strings.
        let x = p_buff.to_str().unwrap();
        println!("{}", x);
        format!("file://{}", x)
    }
}

impl PathFromFileUrlExt<PathBuf> for PathBuf {
    fn from_file_url(file_url: &str) -> PathBuf {
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
        let url = p.to_file_url();
        let s = url.as_str();
        assert_eq!(s, "file:///some/file.txt");
    }

    #[test]
    fn oddball_pathbuf_to_url() {
        let p = PathBuf::from("/gi>/some & what.whtvr");
        let url = p.to_file_url();
        let s = url.as_str();
        assert_eq!(s, "file:///gi%3E/some%20%26%20what.whtvr");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_pathbuf_to_url() {
        let p = PathBuf::from(r"c:\WINDOWS\clock.avi");
        let url = p.to_file_url();
        let s = url.as_str();
        assert_eq!(s, "file:///c:/WINDOWS/clock.avi");
    }

    #[test]
    fn basic_pathbuf_from_url() {
        let one = PathBuf::from("/some/file.txt");
        let two = PathBuf::from_file_url("file:///some/file.txt");
        assert_eq!(one, two);
    }

    #[test]
    fn oddball_pathbuf_from_url() {
        let one = PathBuf::from_file_url("file:///gi%3E/some%20%26%20what.whtvr");
        let two = PathBuf::from("/gi>/some & what.whtvr");
        assert_eq!(one, two);
    }

    #[test]
    fn basic_path_to_url() {
        let one = Path::new("/foo/bar.txt").to_file_url();
        let two = "file:///foo/bar.txt";
        assert_eq!(one, two);
    }

    #[test]
    fn unicode_path_to_url() {
        let one = Path::new("/tmp/😀/#{}^.txt").to_file_url();
        let two = "file:///tmp/%F0%9F%98%80/%23%7B%7D%5E.txt";
        assert_eq!(one, two);
    }

    #[test]
    fn unicode_url_to_path() {
        let one = PathBuf::from_file_url("file:///tmp/%F0%9F%98%80/%23%7B%7D%5E.txt");
        let two = PathBuf::from("/tmp/😀/#{}^.txt");
        assert_eq!(one, two);
    }
}

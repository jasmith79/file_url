//! file_url
//!
//! Makes it easier to Path/PathBuf to/from file URLs.
//!
//! Author: Jared Adam Smith
//! license: MIT
//! © 2021
use lazy_static::{__Deref, lazy_static};
use regex::Regex;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

#[cfg(target_family = "unix")]
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
/// and vice-versa.
///
/// # Example:
/// ```rust
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
                // File url should always be fully qualified
                OsString::from(FORWARD_SLASH)
            } else {
                decode_path_component(url_piece)
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
pub trait PathFromFileUrlExt: private::Sealed {
    /// Constructs a PathBuf from the supplied &str.
    fn from_file_url(file_url: &str) -> PathBuf;
}

impl PathFileUrlExt for Path {
    /// Method to convert a std::path::Path into a file URL.
    /// **NOTE:** on Windows systems the path must be valid
    /// UTF-8 because std::path::Path is backed by a
    /// platform-dependent std::ffi::OsString and there are
    /// difficulties dealing with the byte representation of
    /// platform string on Windows. If the path is not valid
    /// UTF-8 then any erroneous bytes will be replaced with
    /// �. Unix-like operating systems do not have this
    /// restriction.
    ///
    /// # Example:
    /// ```rust
    /// use std::path::Path;
    /// use file_url::PathFileUrlExt;
    ///
    /// let p = Path::new("/foo/bar baz.txt");
    /// assert_eq!(p.to_file_url(), "file:///foo/bar%20baz.txt");
    /// ```
    fn to_file_url(&self) -> String {
        #[cfg(target_family = "windows")]
        let (p, cmp): (Vec<Component>, Vec<Component>) =
            self.components()
                .into_iter()
                .partition(|component| match component {
                    Component::Prefix(_) => true,
                    _ => false,
                });

        #[cfg(target_family = "windows")]
        let pref = p.first();
        #[cfg(target_family = "windows")]
        let component_iter = cmp.iter();

        #[cfg(target_family = "unix")]
        let component_iter = self.components().into_iter();
        #[cfg(target_family = "unix")]
        let pref: Option<Component> = Option::None;

        let cs;
        if self.has_root() {
            cs = component_iter.skip(1);
        } else {
            cs = component_iter.skip(0);
        }

        let encoded = cs
            .map(|component| match component {
                Component::CurDir | Component::ParentDir => {
                    component.as_os_str().to_string_lossy().to_string()
                }
                Component::Normal(s) => encode_path_component(s.deref().to_owned()),
                _ => panic!("Unexpected path component."),
            })
            .collect::<Vec<_>>()
            .join("/");

        match pref {
            None => format!("file:///{}", encoded),
            Some(p) => format!("file:///{}/{}", p.as_os_str().to_string_lossy(), encoded),
        }
    }
}

impl PathFromFileUrlExt for PathBuf {
    /// Creates a std::path::PathBuf from a given file URL.
    ///
    /// # Example:
    /// ```rust
    /// use std::path::PathBuf;
    /// use file_url::PathFromFileUrlExt;
    ///
    /// let p = PathBuf::from("/foo/bar baz.txt");
    /// assert_eq!(p, PathBuf::from_file_url("file:///foo/bar%20baz.txt"));
    /// ```
    fn from_file_url(file_url: &str) -> PathBuf {
        file_url_to_pathbuf(file_url)
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::PathBuf {}
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

#[cfg(target_os = "windows")]
#[cfg(test)]
mod windows_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn windows_pathbuf_to_url() {
        let p = PathBuf::from(r"c:\WINDOWS\clock.avi");
        let url = p.to_file_url();
        let s = url.as_str();
        assert_eq!(s, "file:///c:/WINDOWS/clock.avi");
    }
}

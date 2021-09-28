use lazy_static::lazy_static;
#[cfg(target_family = "unix")]
use percent_encoding::percent_encode;
#[cfg(target_family = "windows")]
use percent_encoding::utf8_percent_encode;
use percent_encoding::{percent_decode_str, AsciiSet, CONTROLS};
use std::ffi::OsString;

#[cfg(target_family = "unix")]
use crate::os_str_from_bytes::{OsStringExt, OsStringFromByteArrExt};

pub struct ControlByteWrapper {
    controls: AsciiSet,
}

lazy_static! {
    static ref FILE_URL_BYTES: ControlByteWrapper = ControlByteWrapper {
        // RFC 3986 section 2.2 Reserved Characters, except ':'
        // because of Windows drive designators.
        controls: CONTROLS
            .add(b'/') // needs escaped inside paths.
            .add(b' ')
            .add(b'#')
            .add(b'$')
            .add(b'&')
            .add(b'+')
            .add(b',')
            .add(b';')
            .add(b'=')
            .add(b'?')
            .add(b'@')
            .add(b'[')
            .add(b']')
            .add(b'{')
            .add(b'}')
            .add(b'`')
            .add(b'<')
            .add(b'>')
            .add(b'^')
            .add(b'!')
            .add(b'\'')
            .add(b'(')
            .add(b')')
            .add(b'*')
    };
}

/// Percent-encodes a std::ffi::OsString from a std::path::Component.
pub fn encode_path_component(path_component: OsString) -> String {
    #[cfg(target_family = "unix")]
    {
        let b = path_component.into_vec();
        percent_encode(&b, &FILE_URL_BYTES.controls).collect()
    }
    #[cfg(target_family = "windows")]
    {
        let s = path_component.to_string_lossy();
        utf8_percent_encode(&s, &FILE_URL_BYTES.controls).to_string()
    }
}

/// Decodes a percent-encoded &str into a DecodeResult.
pub fn decode_path_component(encoded_path_compenent: &str) -> OsString {
    #[cfg(target_family = "unix")]
    {
        let b: Vec<u8> = percent_decode_str(encoded_path_compenent).collect();
        OsString::from_byte_vec(&b)
    }
    #[cfg(target_family = "windows")]
    {
        let x = percent_decode_str(encoded_path_compenent)
            .decode_utf8_lossy()
            .into_owned();
        OsString::from(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf; // Easy way to get some OsStrings

    #[test]
    fn encode_test() {
        let first = PathBuf::from("/😀#{}^some & what.whtvr")
            .components()
            .into_iter()
            .last()
            .unwrap()
            .as_os_str()
            .to_owned();

        let enc = encode_path_component(first);
        assert_eq!(enc, "%F0%9F%98%80%23%7B%7D%5Esome%20%26%20what.whtvr");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn decode_test() {
        let b = OsString::from_byte_vec(&"😀#{}^some & what.whtvr".as_bytes().to_vec());
        let dec = decode_path_component("%F0%9F%98%80%23%7B%7D%5Esome%20%26%20what.whtvr");
        assert_eq!(b, dec);
    }
}

use crate::os_str_from_bytes::{OsStringExt, OsStringFromByteArrExt};
use lazy_static::lazy_static;
use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, CONTROLS};
use std::ffi::OsString;
use std::ops::Deref;

pub struct ControlByteWrapper {
    controls: AsciiSet,
}

#[derive(Debug, Clone)]
pub struct DecodeResult {
    buff: Vec<u8>,
}

impl DecodeResult {
    pub fn to_os_string(&mut self) -> OsString {
        OsString::from_byte_vec(&self.buff)
    }
}

impl Deref for DecodeResult {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.buff
    }
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

pub fn encode_path_component(path_component: OsString) -> String {
    let b = path_component.into_vec();
    percent_encode(&b, &FILE_URL_BYTES.controls).collect()
}

pub fn decode_path_component(encoded_path_compenent: &str) -> DecodeResult {
    let b: Vec<u8> = percent_decode_str(encoded_path_compenent).collect();
    DecodeResult { buff: b }
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
    fn decode_test() {
        let b = "😀#{}^some & what.whtvr".as_bytes().to_vec();
        let dec = decode_path_component("%F0%9F%98%80%23%7B%7D%5Esome%20%26%20what.whtvr");
        assert_eq!(b, *dec);
    }
}

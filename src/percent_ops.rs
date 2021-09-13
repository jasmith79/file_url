use crate::os_str_from_bytes::OsStringFromByteArrExt;
use lazy_static::lazy_static;
use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, CONTROLS};
use std::ffi::OsString;

pub struct ControlByteWrapper {
    controls: AsciiSet,
}

#[derive(Debug, Clone)]
pub struct DecodeResult {
    buff: Vec<u8>,
}

impl DecodeResult {
    pub fn as_os_str(&mut self) -> OsString {
        OsString::from_byte_vec(&self.buff)
    }

    pub fn to_vec(&mut self) -> &[u8] {
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

pub fn encode_path_component(c: &[u8]) -> String {
    percent_encode(c, &FILE_URL_BYTES.controls).collect()
}

pub fn decode_path_component(c: &str) -> DecodeResult {
    let b: Vec<u8> = percent_decode_str(c).collect();
    DecodeResult { buff: b }
}

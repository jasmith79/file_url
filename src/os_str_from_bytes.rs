use std::ffi::OsString;
pub use std::os::unix::ffi::OsStringExt;

pub trait OsStringFromByteArrExt: private::Sealed {
    fn from_byte_vec(b: &[u8]) -> OsString;
}

impl OsStringFromByteArrExt for OsString {
    fn from_byte_vec(b: &[u8]) -> OsString {
        return OsString::from_vec(b.to_vec());
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::OsString {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn convert_bytes_to_osstring() {
        let b: &[u8] = &[b'a', b'b', b'c'];
        let s = OsString::from_byte_vec(b);
        assert_eq!(s, "abc");
    }
}

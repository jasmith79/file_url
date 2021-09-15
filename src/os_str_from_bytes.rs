use std::ffi::OsString;
#[cfg(target_family = "unix")]
pub use std::os::unix::ffi::OsStringExt;
#[cfg(target_family = "windows")]
pub use std::os::windows::ffi::OsStringExt;

pub trait OsStringFromByteArrExt {
    fn from_byte_vec(b: &[u8]) -> OsString;
}

impl OsStringFromByteArrExt for OsString {
    fn from_byte_vec(b: &[u8]) -> OsString {
        #[cfg(target_family = "unix")]
        return OsString::from_vec(b.to_vec());
        #[cfg(target_family = "windows")]
        return OsString::from_wide(
            b.chunks_exact(2)
                .into_iter()
                .map(|a, b| u16::from_ne_bytes(a, b))
                .collect::<Vec<u16>>(),
        );
    }
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

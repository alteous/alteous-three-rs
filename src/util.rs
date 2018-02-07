//! Internal utility functions.

#![allow(dead_code)]

use std::{ffi, fs, io, path};
use std::io::Read;

/// Creates an `ffi::CStr` slice from a byte string literal.
///
/// ```
/// let foo = cstr(b"foo\0");
/// ```
pub fn cstr<'a, T>(bytes: &'a T) -> &'a ffi::CStr
    where T: AsRef<[u8]> + ?Sized
{
    ffi::CStr::from_bytes_with_nul(bytes.as_ref()).expect("missing NUL byte")
}

/// Reads the entire contents of a file into a `Vec<u8>`.
pub fn read_file_to_end<P>(path: P) -> ::std::io::Result<Vec<u8>>
    where P: AsRef<::std::path::Path>
{
    match fs::File::open(&path) {
        Ok(file) => {
            let len = file.metadata().ok().map_or(0, |x| x.len() as usize);
            let mut reader = io::BufReader::new(file);
            let mut contents = Vec::with_capacity(len);
            let _ = reader.read_to_end(&mut contents)?;
            Ok(contents)
        }
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                panic!("file not found: {:?}", path.as_ref());
            } else {
                Err(err)
            }
        }
    }
}

/// Reads the entire contents of a file into a `String`.
pub fn read_file_to_string<P>(path: P) -> io::Result<String>
    where P: AsRef<path::Path>
{
    match fs::File::open(&path) {
        Ok(file) => {
            let len = file.metadata().ok().map_or(0, |x| x.len() as usize);
            let mut contents = String::with_capacity(len);
            let _ = io::BufReader::new(file).read_to_string(&mut contents)?;
            Ok(contents)
        }
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                panic!("file not found: {:?}", path.as_ref());
            } else {
                Err(err)
            }
        }
    }
}

/// Reads the entire contents of a file into a `CString`.
pub fn read_file_to_cstring<P>(path: P) -> io::Result<ffi::CString>
    where P: AsRef<path::Path>
{
    read_file_to_end(path).map(|vec| ffi::CString::new(vec).unwrap())
}

use std::mem;
use std::slice;
use std::io::{Read, Write, Error};

#[macro_export]
macro_rules! offset_of {
    ($ty: ty, $field: ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}

pub enum SearchDirection2D {
    Up,
    Left,
    Down,
    Right,
}

pub fn read_struct<T, R: Read>(reader: &mut R) -> Result<T, Error> {
    let num_bytes = mem::size_of::<T>();

    unsafe {
        let mut s = mem::uninitialized::<T>();
        let buffer = slice::from_raw_parts_mut(&mut s as *mut T as *mut u8, num_bytes);

        match reader.read_exact(buffer) {
            Ok(_) => Ok(s),
            Err(e) => {
                mem::forget(s);
                Err(e)
            }
        }
    }
}

pub fn write_struct<T, W: Write>(writer: &mut W, value: &mut T) -> Result<usize, Error> {
    let num_bytes = mem::size_of::<T>();

    unsafe {
        let buffer = slice::from_raw_parts_mut(value as *mut T as *mut u8, num_bytes);

        match writer.write(buffer) {
            Ok(s) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

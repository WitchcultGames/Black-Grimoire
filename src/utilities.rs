use std::io::{Error, Read, Write};
use std::mem;
use std::slice;

//#[macro_export]
//macro_rules! offset_of {
//    ($ty: ty, $field: ident) => {
//        &(*(std::ptr::null() as *const $ty)).$field as *const _ as usize
//    }
//}

//#[macro_export]
//macro_rules! offset_of {
//    ($ty: ty, $field: ident) => {
//        unsafe {
//            //  Create correctly sized storage.
//            //
//            //  Note: `let zeroed: $ty = ::std::mem::zeroed();` is incorrect,
//            //        a zero pattern is not always a valid value.
//            let buffer = ::std::mem::MaybeUninit::<$ty>::uninit();
//
//            //  Create a Raw reference to the storage:
//            //  - Alignment does not matter, though is correct here.
//            //  - It safely refers to uninitialized storage.
//            //
//            //  Note: using `&raw const *(&buffer as *const _ as *const $ty)`
//            //        is incorrect, it creates a temporary non-raw reference.
//            let uninit: &raw const $ty = ::std::mem::transmute(&buffer);
//
//            //  Create a Raw reference to the field:
//            //  - Alignment does not matter, though is correct here.
//            //  - It points within the memory area.
//            //  - It safely refers to uninitialized storage.
//            let field = &raw const uninit.$field;
//
//            //  Compute the difference between pointers.
//            (field as *const _ as usize) - (uninit as *const_ as usize)
//        }
//    }
//}

#[macro_export]
macro_rules! offset_of {
    ($type:ty, $field:tt) => {{
        let dummy = ::core::mem::MaybeUninit::<$type>::uninit();
        let dummy_ptr = dummy.as_ptr();
        let member_ptr = ::core::ptr::addr_of!((*dummy_ptr).$field);
        member_ptr as usize - dummy_ptr as usize
    }};
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
        let mut s = mem::MaybeUninit::<T>::uninit().assume_init();
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

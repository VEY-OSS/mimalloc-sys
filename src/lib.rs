#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod bindgen {
    include!(concat!(env!("OUT_DIR"), "/bindgen.rs"));
}
pub use bindgen::*;

//  LIB.rs
//    by Lut99
//
//  Created:
//    18 Nov 2023, 12:57:56
//  Last edited:
//    19 Nov 2023, 19:43:27
//  Auto updated?
//    Yes
//
//  Description:
//!   A (suite of) Rust procedural macro(s) that can be used to compile a
//!   schema- or specification-like struct to multiple versions of itself.
//

mod versioning;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DataStruct};

// TODO: make custom parse type for "statements" in our little DSL


/***** MACROS *****/
#[inline]
#[proc_macro]
pub fn versioning(input: TokenStream) -> TokenStream {
    // Parse the input as a data thing
    let data: DataStruct = parse_macro_input!(input);

    match versioning::call(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.abort(),
    }
}

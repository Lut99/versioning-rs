//  LIB.rs
//    by Lut99
//
//  Created:
//    18 Nov 2023, 12:57:56
//  Last edited:
//    20 Nov 2023, 13:26:45
//  Auto updated?
//    Yes
//
//  Description:
//!   A (suite of) Rust procedural macro(s) that can be used to compile a
//!   schema- or specification-like struct to multiple versions of itself.
//

mod spec;
mod versioned;

use proc_macro::TokenStream;


/***** MACROS *****/
/// Defines the attribute macro that declares a particular module as having versioned definitions inside of it.
///
/// # Arguments
/// - `attr`: The tokens given in the attribute, i.e., the stuff in between the brackets in `#[versioned(...)]`.
/// - `input`: The tokens that are being attributed. This defines the versioned region.
///
/// # Returns
/// A new [`TokenStream`] replacing the `input`.
#[inline]
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn versioned(attr: TokenStream, input: TokenStream) -> TokenStream {
    match versioned::call(attr.into(), input.into()) {
        Ok(res) => res.into(),
        Err(err) => err.abort(),
    }
}

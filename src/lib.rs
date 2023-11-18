//  LIB.rs
//    by Lut99
//
//  Created:
//    18 Nov 2023, 12:57:56
//  Last edited:
//    18 Nov 2023, 13:17:39
//  Auto updated?
//    Yes
//
//  Description:
//!   A (suite of) Rust procedural macro(s) that can be used to compile a
//!   schema- or specification-like struct to multiple versions of itself.
//

use std::cell::RefCell;
use std::thread_local;

use proc_macro::TokenStream;


/***** GLOBALS *****/
thread_local! {
    static TEST: RefCell<Option<TokenStream>> = RefCell::new(None);
}





/***** MACROS *****/
#[proc_macro_attribute]
pub fn versioning(_attr: TokenStream, item: TokenStream) -> TokenStream {
    TEST.with(|test| *test.borrow_mut() = Some(item));
    TokenStream::new()
}

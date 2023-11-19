//  VERSIONING.rs
//    by Lut99
//
//  Created:
//    19 Nov 2023, 19:25:25
//  Last edited:
//    19 Nov 2023, 19:36:12
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the toplevel of the `versioning!`-macro.
//

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data};


/***** LIBRARY *****/
/// Handles the toplevel `versioning!(...)` call.
///
/// # Arguments
/// - `input`: The input [`TokenStream2`] to parse.
///
/// # Returns
/// An output [`TokenStream2`] containing the versioned versions of the given input.
///
/// # Errors
/// This function may error if it failed to correctly understand the input.
pub fn call(input: TokenStream) -> Result<TokenStream2, Diagnostic> {
    let data: Data = parse_macro_input!(input as Data);

    // Alright let's parse either a struct/enum/union or a function (to begin with)
    if let Ok(data) = syn::parse::<Data>(input) {
        Ok(quote! { #data })
    } else {
        Err(Diagnostic::spanned(input.span(), Level::Error, "Only structs or enums allowed as input to `versioning!`".into()))
    }
}

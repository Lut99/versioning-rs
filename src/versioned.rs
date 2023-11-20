//  VERSIONED.rs
//    by Lut99
//
//  Created:
//    19 Nov 2023, 19:25:25
//  Last edited:
//    20 Nov 2023, 14:02:12
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the toplevel of the `#[versioned(...)]`-macro.
//

use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Visibility};

use crate::spec::{Body, BodyItem, Version, VersionFilterList, VersionList};


/***** HELPER FUNCTIONS *****/
/// Attempts to read the `#[version(...)]`-attribute from the given list of attributes.
///
/// # Arguments
/// - `attrs`: The attributes to read.
///
/// # Returns
/// The [`VersionFilterList`] specified in the `#[version(...)]`-macro if it was found, or else [`None`] if the macro wasn't given.
pub fn get_version_attr(attrs: &[Attribute]) -> Result<Option<VersionFilterList>, Diagnostic> { Ok(None) }

/// Filters the given body item in accordance to the list of versions.
///
/// # Arguments
/// - `item`: The [`BodyItem`] to filter.
/// - `versions`: The list of versions in total (allows us to define order)
/// - `version`: The current version to filter for.
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
pub fn filter_item(item: &BodyItem, versions: &VersionList, version: &Version) -> Result<TokenStream2, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    let attrs: &[Attribute] = match item {
        BodyItem::Enum(e) => &e.attrs,
        BodyItem::Struct(s) => &s.attrs,
    };
    if let Some(version) = get_version_attr(attrs)? {}

    // Done
    Ok(quote! {})
}





/***** LIBRARY *****/
/// Handles the toplevel `#[versioned(...)]` call.
///
/// # Arguments
/// - `attrs`: The given attributes to parse.
/// - `input`: The input [`TokenStream2`] to parse.
///
/// # Returns
/// An output [`TokenStream2`] containing the versioned versions of the given input.
///
/// # Errors
/// This function may error if it failed to correctly understand the input.
pub fn call(attrs: TokenStream2, input: TokenStream2) -> Result<TokenStream2, Diagnostic> {
    // Parse the attributes first as a list of versions
    let versions: VersionList = match syn::parse2(attrs) {
        Ok(versions) => versions,
        Err(err) => return Err(Diagnostic::spanned(err.span(), Level::Error, err.to_string())),
    };

    // Next, parse the input as a module
    let input_span: Span = input.span();
    let body: Body = match syn::parse2(input) {
        Ok(body) => body,
        Err(_) => {
            return Err(Diagnostic::spanned(input_span, Level::Error, "Can only use `#[versioned(...)]` macro on modules, structs or enums".into()));
        },
    };
    println!("{body:?}");

    // Now examine all of the body items to serialize them appropriately
    let mut impls: Vec<TokenStream2> = Vec::with_capacity(versions.0.len());
    for version in &versions.0 {
        // Collect the filtered editions of all the items
        let mut filtered_items: Vec<TokenStream2> = Vec::with_capacity(body.items.len());
        for item in &body.items {
            filtered_items.push(filter_item(item, &versions, version)?);
        }

        // Generate a module for this version
        let vis: &Visibility = &body.vis;
        let ident: Ident = Ident::new(&version.0.value(), version.0.span());
        impls.push(quote! {
            #vis mod #ident {
                #(#filtered_items)*
            }
        });
    }

    // Done
    Ok(quote! {
        #(#impls)*
    })
}

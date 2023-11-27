//  VERSIONING.rs
//    by Lut99
//
//  Created:
//    19 Nov 2023, 19:25:25
//  Last edited:
//    27 Nov 2023, 15:55:55
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
use syn::{Attribute, Ident, Meta, Visibility};

use crate::spec::BodyItem;
use crate::version::{Version, VersionFilter, VersionList};


/***** HELPER FUNCTIONS *****/
/// Attempts to read the `#[version(...)]`-attribute from the given list of attributes.
///
/// # Arguments
/// - `attrs`: The attributes to read.
///
/// # Returns
/// The [`VersionFilterList`] specified in the `#[version(...)]`-macro if it was found, or else [`None`] if the macro wasn't given.
pub fn get_version_attr(attrs: &[Attribute]) -> Result<Option<VersionFilter>, Diagnostic> {
    // Iterate over the attributes
    for attr in attrs {
        match &attr.meta {
            Meta::List(l) => {
                if l.path.is_ident("version") {
                    return match l.parse_args() {
                        Ok(filter) => Ok(Some(filter)),
                        Err(err) => Err(Diagnostic::spanned(err.span(), Level::Error, err.to_string())),
                    };
                } else {
                    // Not ours, ignore
                    continue;
                }
            },
            // Not ours, ignore
            Meta::NameValue(_) => continue,
            Meta::Path(_) => continue,
        }
    }
    Ok(None)
}

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
    println!("Filtering item @ {:?} for version {:?}", item.span(), version);

    // First, check the item's attributes to see if it has been version filtered
    if let Some(version) = get_version_attr(item.attrs())? {
        println!("{version:#?}");
    }

    // Then serialize to quote
    match item {
        BodyItem::Module(attrs, vis, name, items, _) => {
            // Recurse into the contents first
            let mut filtered: TokenStream2 = TokenStream2::new();
            for item in items {
                filtered.extend(filter_item(item, versions, version)?);
            }

            // Now make the module
            Ok(quote! {
                #(#attrs)*
                #vis mod #name {
                    #filtered
                }
            })
        },

        BodyItem::Enum(e) => Ok(quote! { #e }),
        BodyItem::Struct(s) => Ok(quote! { #s }),
    }
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
    let body: BodyItem = match syn::parse2(input) {
        Ok(body) => body,
        Err(_) => {
            return Err(Diagnostic::spanned(input_span, Level::Error, "Can only use `#[versioned(...)]` macro on modules, structs or enums".into()));
        },
    };
    println!("{body:?}");

    // Now examine all of the body items to serialize them appropriately
    let mut impls: Vec<TokenStream2> = Vec::with_capacity(versions.0.len());
    for version in &versions.0 {
        // Collect the filtered editions of this item and all its contents
        let filtered: TokenStream2 = filter_item(&body, &versions, version)?;

        // Generate a module for this version
        let vis: &Visibility = body.vis();
        let ident: Ident = Ident::new(&version.0.value(), version.0.span());
        impls.push(quote! {
            #vis mod #ident {
                #filtered
            }
        });
    }

    // Done
    Ok(quote! {
        #(#impls)*
    })
}

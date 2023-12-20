//  VERSIONING.rs
//    by Lut99
//
//  Created:
//    19 Nov 2023, 19:25:25
//  Last edited:
//    20 Dec 2023, 15:39:55
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
use syn::token::Pub;
use syn::{Attribute, Ident, Meta, Visibility};

use crate::spec::BodyItem;
use crate::version::{Filter as _, Version, VersionFilter, VersionList};


/***** HELPER FUNCTIONS *****/
/// Attempts to read the `#[version(...)]`-attribute from the given list of attributes.
///
/// Then removes it from the given list if found.
///
/// # Arguments
/// - `attrs`: The attributes to read.
///
/// # Returns
/// The [`VersionFilterList`] specified in the `#[version(...)]`-macro if it was found, or else [`None`] if the macro wasn't given.
pub fn remove_version_attr(attrs: &mut Vec<Attribute>) -> Result<Option<(Attribute, VersionFilter)>, Diagnostic> {
    // Iterate over the attributes
    for (i, attr) in attrs.into_iter().enumerate() {
        match &attr.meta {
            Meta::List(l) => {
                if l.path.is_ident("version") {
                    return match l.parse_args() {
                        Ok(filter) => {
                            // Before we return, remove from the list
                            Ok(Some((attrs.remove(i), filter)))
                        },
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
pub fn filter_item(item: &mut BodyItem, versions: &VersionList, version: &Version) -> Result<TokenStream2, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    let filter: Option<(Attribute, VersionFilter)> = remove_version_attr(item.attrs_mut())?;
    if let Some((attr, filter)) = &filter {
        // Next, see if this matches the current version
        if !filter.matches(version) {
            // Don't forget to restore the attribute
            item.attrs_mut().push(attr.clone());

            // Now skip it
            return Ok(quote! {});
        }
    }

    // Then serialize to quote
    let res: TokenStream2 = match item {
        BodyItem::Module(attrs, vis, unsafety, name, items, mod_token, semi_token, _) => {
            // Recurse into the contents first
            let mut filtered: TokenStream2 = TokenStream2::new();
            for item in items {
                filtered.extend(filter_item(item, versions, version)?);
            }

            // Now make the module
            quote! {
                #(#attrs)*
                #vis #unsafety #mod_token #name {
                    #filtered
                } #semi_token
            }
        },

        BodyItem::Enum(e) => quote! { #e },
        BodyItem::Struct(s) => quote! { #s },
    };

    // Now restore the attribute to the list
    if let Some((attr, _)) = filter {
        item.attrs_mut().push(attr);
    }

    // Done!
    Ok(res)
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
    let mut body: BodyItem = match syn::parse2(input) {
        Ok(body) => body,
        Err(_) => {
            return Err(Diagnostic::spanned(input_span, Level::Error, "Can only use `#[versioned(...)]` macro on modules, structs or enums".into()));
        },
    };

    // Set the toplevel item's visibility to public, so it's always accessible in our generated module
    let old_vis: Visibility = body.vis().clone();
    *body.vis_mut() = Visibility::Public(Pub { span: old_vis.span() });

    // Now examine all of the body items to serialize them appropriately
    let mut impls: Vec<TokenStream2> = Vec::with_capacity(versions.0.len());
    for version in &versions.0 {
        // Collect the filtered editions of this item and all its contents
        let filtered: TokenStream2 = filter_item(&mut body, &versions, version)?;

        // Generate a module for this version
        let ident: Ident = Ident::new(&version.0.value(), version.0.span());
        impls.push(quote! {
            #old_vis mod #ident {
                #filtered
            }
        });
    }

    // Done
    Ok(quote! {
        #(#impls)*
    })
}

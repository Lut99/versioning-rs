//  VERSIONING.rs
//    by Lut99
//
//  Created:
//    19 Nov 2023, 19:25:25
//  Last edited:
//    20 Dec 2023, 16:33:35
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the toplevel of the `#[versioned(...)]`-macro.
//

use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Comma, Pub};
use syn::{Attribute, Field, Fields, Ident, ImplItem, ImplItemConst, ImplItemFn, ImplItemMacro, ImplItemType, Meta, Variant, Visibility};

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
pub fn remove_version_attr(attrs: &mut Vec<Attribute>) -> Result<Option<VersionFilter>, Diagnostic> {
    // Iterate over the attributes
    for (i, attr) in attrs.into_iter().enumerate() {
        match &attr.meta {
            Meta::List(l) => {
                if l.path.is_ident("version") {
                    return match l.parse_args() {
                        Ok(filter) => {
                            // Before we return, remove from the list
                            attrs.remove(i);
                            Ok(Some(filter))
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
pub fn filter_item(mut item: BodyItem, versions: &VersionList, version: &Version) -> Result<TokenStream2, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    if let Some(filter) = remove_version_attr(item.attrs_mut())? {
        // Next, see if this matches the current version
        filter.verify(versions)?;
        if !filter.matches(versions, version) {
            return Ok(quote! {});
        }
    }

    // Then serialize to quote
    match item {
        BodyItem::Module(attrs, vis, unsafety, name, items, mod_token, semi_token, _) => {
            // Recurse into the contents first
            let mut filtered: TokenStream2 = TokenStream2::new();
            for item in items {
                filtered.extend(filter_item(item, versions, version)?);
            }

            // Now make the module
            Ok(quote! {
                #(#attrs)*
                #vis #unsafety #mod_token #name {
                    #filtered
                } #semi_token
            })
        },

        BodyItem::Enum(mut e) => {
            // Filter variants next
            let mut fvariants: Punctuated<Variant, Comma> = Punctuated::new();
            for mut variant in e.variants {
                // See if this variant has the attribute
                if let Some(filter) = remove_version_attr(&mut variant.attrs)? {
                    filter.verify(versions)?;
                    if !filter.matches(versions, version) {
                        // Don't add it!
                        continue;
                    }
                }

                // Otherwise, examine the variant's fields
                variant.fields = match variant.fields {
                    Fields::Named(mut fields) => {
                        let mut ffields: Punctuated<Field, Comma> = Punctuated::new();
                        for mut field in fields.named {
                            // See if this field has the attribute
                            if let Some(filter) = remove_version_attr(&mut field.attrs)? {
                                filter.verify(versions)?;
                                if !filter.matches(versions, version) {
                                    // Don't add it!
                                    continue;
                                }
                            }
                            ffields.push(field);
                        }
                        fields.named = ffields;
                        Fields::Named(fields)
                    },
                    Fields::Unnamed(mut fields) => {
                        let mut ffields: Punctuated<Field, Comma> = Punctuated::new();
                        for mut field in fields.unnamed {
                            // See if this field has the attribute
                            if let Some(filter) = remove_version_attr(&mut field.attrs)? {
                                filter.verify(versions)?;
                                if !filter.matches(versions, version) {
                                    // Don't add it!
                                    continue;
                                }
                            }
                            ffields.push(field);
                        }
                        fields.unnamed = ffields;
                        Fields::Unnamed(fields)
                    },

                    // Nothing to do here
                    Fields::Unit => Fields::Unit,
                };

                // Add it
                fvariants.push(variant);
            }
            e.variants = fvariants;

            // Then serialize
            Ok(quote! { #e })
        },
        BodyItem::Struct(mut s) => {
            // Filter the struct's fields
            s.fields = match s.fields {
                Fields::Named(mut fields) => {
                    let mut ffields: Punctuated<Field, Comma> = Punctuated::new();
                    for mut field in fields.named {
                        // See if this field has the attribute
                        if let Some(filter) = remove_version_attr(&mut field.attrs)? {
                            filter.verify(versions)?;
                            if !filter.matches(versions, version) {
                                // Don't add it!
                                continue;
                            }
                        }
                        ffields.push(field);
                    }
                    fields.named = ffields;
                    Fields::Named(fields)
                },
                Fields::Unnamed(mut fields) => {
                    let mut ffields: Punctuated<Field, Comma> = Punctuated::new();
                    for mut field in fields.unnamed {
                        // See if this field has the attribute
                        if let Some(filter) = remove_version_attr(&mut field.attrs)? {
                            filter.verify(versions)?;
                            if !filter.matches(versions, version) {
                                // Don't add it!
                                continue;
                            }
                        }
                        ffields.push(field);
                    }
                    fields.unnamed = ffields;
                    Fields::Unnamed(fields)
                },

                // Nothing to do here
                Fields::Unit => Fields::Unit,
            };

            // Then serialize
            Ok(quote! { #s })
        },

        BodyItem::Impl(mut i) => {
            // Go over the nested items
            let mut fitems: Vec<ImplItem> = Vec::new();
            for mut item in i.items {
                // Check if we want to keep this based on the item's attributes
                let keep: bool = match &mut item {
                    ImplItem::Const(ImplItemConst { attrs, .. })
                    | ImplItem::Fn(ImplItemFn { attrs, .. })
                    | ImplItem::Macro(ImplItemMacro { attrs, .. })
                    | ImplItem::Type(ImplItemType { attrs, .. }) => {
                        if let Some(filter) = remove_version_attr(attrs)? {
                            filter.verify(versions)?;
                            filter.matches(versions, version)
                        } else {
                            true
                        }
                    },

                    // Rest is always kept
                    _ => true,
                };

                // Then keep it if so
                if keep {
                    fitems.push(item);
                }
            }
            i.items = fitems;

            // Jep, done here, serialize!
            Ok(quote! { #i })
        },
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
    let mut body: BodyItem = match syn::parse2(input) {
        Ok(body) => body,
        Err(_) => {
            return Err(Diagnostic::spanned(input_span, Level::Error, "Can only use `#[versioned(...)]` macro on modules, structs or enums".into()));
        },
    };

    // Set the toplevel item's visibility to public, so it's always accessible in our generated module
    let old_vis: Option<Visibility> = body.vis_mut().map(|vis| {
        let old_vis: Visibility = vis.clone();
        *vis = Visibility::Public(Pub { span: old_vis.span() });
        old_vis
    });

    // Now examine all of the body items to serialize them appropriately
    let mut impls: Vec<TokenStream2> = Vec::with_capacity(versions.0.len());
    for version in &versions.0 {
        // Collect the filtered editions of this item and all its contents
        let filtered: TokenStream2 = filter_item(body.clone(), &versions, version)?;

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

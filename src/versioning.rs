//  VERSIONING.rs
//    by Lut99
//
//  Created:
//    19 Nov 2023, 19:25:25
//  Last edited:
//    21 Dec 2023, 10:03:59
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
use syn::{
    Attribute, Expr, ExprLit, Field, Fields, Ident, ImplItem, ImplItemConst, ImplItemFn, ImplItemMacro, ImplItemType, Lit, LitBool, Meta, Variant,
    Visibility,
};

use crate::spec::BodyItem;
use crate::version::{Filter as _, Version, VersionFilter, VersionList};


/***** HELPERS *****/
/// Defines the configurable options to the `#[versioning(...)]`-macro.
#[derive(Debug)]
struct Options {
    /// Whether to inject `#[cfg(feature = "...")]` when generating code or not.
    features: bool,
    /// Whether modules starting with an underscore (`_`) are omitted from the generated code or not.
    invisible_modules: bool,
}
impl Default for Options {
    #[inline]
    fn default() -> Self { Self { features: false, invisible_modules: true } }
}





/***** HELPER FUNCTIONS *****/
/// Parses macro input as a [`VersionList`] and any options given as a key/value pair.
///
/// # Arguments
/// - `tokens`: The [`TokenStream2`] that contains the input to the attribute macro.
///
/// # Returns
/// A tuple with a [`VersionList`], containing the given versions, and an [`Options`], containing other configuration.
///
/// # Errors
/// This function can error if the input is not valid.
fn parse_input(tokens: TokenStream2) -> Result<(VersionList, Options), Diagnostic> {
    // Parse the tokens as attributes first
    let metas: Punctuated<Meta, Comma> = match syn::parse::Parser::parse2(Punctuated::parse_terminated, tokens) {
        Ok(metas) => metas,
        Err(err) => return Err(Diagnostic::spanned(err.span(), Level::Error, err.to_string())),
    };

    // Parse them
    let mut versions: VersionList = VersionList(vec![]);
    let mut opts: Options = Options::default();
    for meta in metas {
        // Match the meta
        match meta {
            // We assume paths are version identifiers
            Meta::Path(p) => match p.get_ident() {
                Some(ident) => versions.0.push(Version(ident.clone())),
                None => return Err(Diagnostic::spanned(p.span(), Level::Error, format!("Given version number is not a valid identifier"))),
            },

            // Key/Value pairs are settings
            Meta::NameValue(nv) => {
                if nv.path.is_ident("features") {
                    // Parse the value as a boolean literal
                    let val: bool = if let Expr::Lit(ExprLit { lit: Lit::Bool(LitBool { value, .. }), .. }) = nv.value {
                        value
                    } else {
                        return Err(Diagnostic::spanned(
                            nv.value.span(),
                            Level::Error,
                            "'features' option must be given a boolean (true/false)".into(),
                        ));
                    };

                    // Store it
                    opts.features = val;
                } else if nv.path.is_ident("invisible_modules") {
                    // Parse the value as a boolean literal
                    let val: bool = if let Expr::Lit(ExprLit { lit: Lit::Bool(LitBool { value, .. }), .. }) = nv.value {
                        value
                    } else {
                        return Err(Diagnostic::spanned(
                            nv.value.span(),
                            Level::Error,
                            "'invisible_modules' option must be given a boolean (true/false)".into(),
                        ));
                    };

                    // Store it
                    opts.invisible_modules = val;
                } else {
                    return Err(Diagnostic::spanned(
                        nv.path.span(),
                        Level::Error,
                        format!("Unknown configuration parameter '{}'", nv.path.span().source_text().unwrap_or_else(|| "???".into())),
                    ));
                }
            },

            // Others we ignore
            Meta::List(l) => return Err(Diagnostic::spanned(l.span(), Level::Error, "Not a valid options to the `#[versioning(...)]`-macro".into())),
        }
    }

    // Alright return the lot
    Ok((versions, opts))
}

/// Attempts to read the `#[version(...)]`-attribute from the given list of attributes.
///
/// Then removes it from the given list if found.
///
/// # Arguments
/// - `attrs`: The attributes to read.
///
/// # Returns
/// The [`VersionFilterList`] specified in the `#[version(...)]`-macro if it was found, or else [`None`] if the macro wasn't given.
fn remove_version_attr(attrs: &mut Vec<Attribute>) -> Result<Option<VersionFilter>, Diagnostic> {
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
/// - `opts`: Any options active.
/// - `is_toplevel`: If true, assumes we're working on the toplevel. This means that:
///    - Modules starting with `_` are not generated, but only their contents (i.e., invisible top modules).
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
fn filter_item(mut item: BodyItem, versions: &VersionList, version: &Version, opts: &Options, is_toplevel: bool) -> Result<TokenStream2, Diagnostic> {
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
                filtered.extend(filter_item(item, versions, version, opts, false)?);
            }

            // Now make the module - but only if given!
            if opts.invisible_modules && is_toplevel && name.to_string().starts_with('_') {
                Ok(quote! {
                    #filtered
                })
            } else {
                Ok(quote! {
                    #(#attrs)*
                    #vis #unsafety #mod_token #name {
                        #filtered
                    } #semi_token
                })
            }
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
    let (versions, opts): (VersionList, Options) = parse_input(attrs)?;

    // Next, parse the input as a module
    let input_span: Span = input.span();
    let mut body: BodyItem = match syn::parse2(input) {
        Ok(body) => body,
        Err(err) => {
            return Err(Diagnostic::spanned(input_span, Level::Error, err.to_string()));
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
        let filtered: TokenStream2 = filter_item(body.clone(), &versions, version, &opts, true)?;

        // Check if we need to inject the feature gate or not
        let cfg: Option<TokenStream2> = if opts.features {
            let feature: String = version.0.to_string();
            Some(quote! {
                #[cfg(feature = #feature)]
            })
        } else {
            None
        };

        // Generate a module for this version
        let ident: &Ident = &version.0;
        impls.push(quote! {
            #cfg
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

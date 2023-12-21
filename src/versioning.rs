//  VERSIONING.rs
//    by Lut99
//
//  Created:
//    19 Nov 2023, 19:25:25
//  Last edited:
//    21 Dec 2023, 10:47:08
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
use syn::token::{Brace, Comma, Mod, Pub};
use syn::{
    Attribute, Expr, ExprLit, Field, Fields, ForeignItem, ForeignItemFn, ForeignItemMacro, ForeignItemStatic, ForeignItemType, ImplItem,
    ImplItemConst, ImplItemFn, ImplItemMacro, ImplItemType, Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro,
    ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, Lit, LitBool, Meta, TraitItem, TraitItemConst, TraitItemFn, TraitItemMacro,
    TraitItemType, Variant, Visibility,
};

// use crate::spec::BodyItem;
use crate::version::{Filter as _, Version, VersionFilter, VersionList};


/***** HELPERS *****/
/// Defines the configurable options to the `#[versioning(...)]`-macro.
#[derive(Debug)]
struct Options {
    /// Whether to inject `#[cfg(feature = "...")]` when generating code or not.
    features: bool,
    /// Whether toplevel modules are wrapped or renamed.
    nest_toplevel_modules: bool,
}
impl Default for Options {
    #[inline]
    fn default() -> Self { Self { features: false, nest_toplevel_modules: false } }
}





/***** HELPER FUNCTIONS *****/
/// Gets the attributes of an [`Item`], mutably.
///
/// # Arguments
/// - `item`: A(n) (mutable reference to the) [`Item`] of which to return the attributes.
///
/// # Returns
/// A mutable list of [`Attribute`]s.
#[inline]
fn item_attrs_mut(item: &mut Item) -> &mut Vec<Attribute> {
    match item {
        // All the ones we know
        Item::Const(ItemConst { attrs, .. })
        | Item::Enum(ItemEnum { attrs, .. })
        | Item::ExternCrate(ItemExternCrate { attrs, .. })
        | Item::Fn(ItemFn { attrs, .. })
        | Item::ForeignMod(ItemForeignMod { attrs, .. })
        | Item::Impl(ItemImpl { attrs, .. })
        | Item::Macro(ItemMacro { attrs, .. })
        | Item::Mod(ItemMod { attrs, .. })
        | Item::Static(ItemStatic { attrs, .. })
        | Item::Struct(ItemStruct { attrs, .. })
        | Item::Trait(ItemTrait { attrs, .. })
        | Item::TraitAlias(ItemTraitAlias { attrs, .. }) => attrs,

        // And any others, 'cuz non-exhaustive ;(
        _ => unimplemented!(),
    }
}
/// Gets the visibility of an [`Item`], mutably.
///
/// # Arguments
/// - `item`: A(n) (mutable reference to the) [`Item`] of which to return the attributes.
///
/// # Returns
/// A mutable reference to the [`Visibility`], or [`None`] if this variant does not have any.
#[inline]
fn item_vis_mut(item: &mut Item) -> Option<&mut Visibility> {
    match item {
        // All the ones we know and have visibility
        Item::Const(ItemConst { vis, .. })
        | Item::Enum(ItemEnum { vis, .. })
        | Item::ExternCrate(ItemExternCrate { vis, .. })
        | Item::Fn(ItemFn { vis, .. })
        | Item::Mod(ItemMod { vis, .. })
        | Item::Static(ItemStatic { vis, .. })
        | Item::Struct(ItemStruct { vis, .. })
        | Item::Trait(ItemTrait { vis, .. })
        | Item::TraitAlias(ItemTraitAlias { vis, .. }) => Some(vis),

        // All the ones we know that _don't_ have visibility
        Item::ForeignMod(ItemForeignMod { .. }) | Item::Impl(ItemImpl { .. }) | Item::Macro(ItemMacro { .. }) => None,

        // And any others, 'cuz non-exhaustive ;(
        _ => unimplemented!(),
    }
}

/// Gets the attributes of a [`TraitItem`], mutably.
///
/// # Arguments
/// - `item`: A (mutable reference to the) [`TraitItem`] of which to return the attributes.
///
/// # Returns
/// A mutable list of [`Attribute`]s, or [`None`] if this variant does not have any.
#[inline]
fn trait_item_attrs_mut(item: &mut TraitItem) -> Option<&mut Vec<Attribute>> {
    match item {
        // All the ones we know
        TraitItem::Const(TraitItemConst { attrs, .. })
        | TraitItem::Fn(TraitItemFn { attrs, .. })
        | TraitItem::Macro(TraitItemMacro { attrs, .. })
        | TraitItem::Type(TraitItemType { attrs, .. }) => Some(attrs),

        // Except for the vertabim; that one doesn't have any attrs
        TraitItem::Verbatim(_) => None,

        // And any others, 'cuz non-exhaustive ;(
        _ => unimplemented!(),
    }
}

/// Gets the attributes of a [`ForeignItem`], mutably.
///
/// # Arguments
/// - `item`: A (mutable reference to the) [`ForeignItem`] of which to return the attributes.
///
/// # Returns
/// A mutable list of [`Attribute`]s, or [`None`] if this variant does not have any.
#[inline]
fn foreign_item_attrs_mut(item: &mut ForeignItem) -> Option<&mut Vec<Attribute>> {
    match item {
        // All the ones we know
        ForeignItem::Fn(ForeignItemFn { attrs, .. })
        | ForeignItem::Macro(ForeignItemMacro { attrs, .. })
        | ForeignItem::Static(ForeignItemStatic { attrs, .. })
        | ForeignItem::Type(ForeignItemType { attrs, .. }) => Some(attrs),

        // Except for the vertabim; that one doesn't have any attrs
        ForeignItem::Verbatim(_) => None,

        // And any others, 'cuz non-exhaustive ;(
        _ => unimplemented!(),
    }
}

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
                } else if nv.path.is_ident("nest_toplevel_modules") {
                    // Parse the value as a boolean literal
                    let val: bool = if let Expr::Lit(ExprLit { lit: Lit::Bool(LitBool { value, .. }), .. }) = nv.value {
                        value
                    } else {
                        return Err(Diagnostic::spanned(
                            nv.value.span(),
                            Level::Error,
                            "'nest_toplevel_modules' option must be given a boolean (true/false)".into(),
                        ));
                    };

                    // Store it
                    opts.nest_toplevel_modules = val;
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
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
fn filter_item(mut item: Item, versions: &VersionList, version: &Version) -> Result<Option<Item>, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    if let Some(filter) = remove_version_attr(item_attrs_mut(&mut item))? {
        // Next, see if this matches the current version
        filter.verify(versions)?;
        if !filter.matches(versions, version) {
            // Filtered oot!
            return Ok(None);
        }
    }

    // Then recurse if necessary
    match item {
        Item::Mod(mut m) => {
            // Recursively only keep OK modules
            if let Some((brace, items)) = m.content {
                let mut fitems: Vec<Item> = Vec::with_capacity(items.len());
                for item in items {
                    // Only keep OK ones
                    if let Some(item) = filter_item(item, versions, version)? {
                        fitems.push(item);
                    }
                }
                m.content = Some((brace, fitems));
            }

            // OK, let's return!
            Ok(Some(Item::Mod(m)))
        },

        Item::Enum(mut e) => {
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
            Ok(Some(Item::Enum(e)))
        },
        Item::Struct(mut s) => {
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
            Ok(Some(Item::Struct(s)))
        },
        Item::Trait(mut t) => {
            // Filter the trait's items
            let mut fitems: Vec<TraitItem> = Vec::with_capacity(t.items.len());
            for mut item in t.items {
                // If the items have any attributes, see if it must be filtered
                if let Some(attrs) = trait_item_attrs_mut(&mut item) {
                    if let Some(filter) = remove_version_attr(attrs)? {
                        filter.verify(versions)?;
                        if !filter.matches(versions, version) {
                            // Don't add it!
                            continue;
                        }
                    }
                }

                // Otherwise, if we got here, keep it
                fitems.push(item);
            }
            t.items = fitems;

            // OK, continue
            Ok(Some(Item::Trait(t)))
        },

        Item::ForeignMod(mut f) => {
            // Filter the nested items
            let mut fitems: Vec<ForeignItem> = Vec::with_capacity(f.items.len());
            for mut item in f.items {
                // If the items have any attributes, see if it must be filtered
                if let Some(attrs) = foreign_item_attrs_mut(&mut item) {
                    if let Some(filter) = remove_version_attr(attrs)? {
                        filter.verify(versions)?;
                        if !filter.matches(versions, version) {
                            // Don't add it!
                            continue;
                        }
                    }
                }

                // Otherwise, if we got here, keep it
                fitems.push(item);
            }
            f.items = fitems;

            // OK, continue
            Ok(Some(Item::ForeignMod(f)))
        },
        Item::Impl(mut i) => {
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
                    _ => unimplemented!(),
                };

                // Then keep it if so
                if keep {
                    fitems.push(item);
                }
            }
            i.items = fitems;

            // Jep, done here, serialize!
            Ok(Some(Item::Impl(i)))
        },

        // The rest of the defined ones we just pass as-is...
        item if matches!(item, Item::Const(_))
            && matches!(item, Item::ExternCrate(_))
            && matches!(item, Item::Fn(_))
            && matches!(item, Item::Macro(_))
            && matches!(item, Item::Static(_))
            && matches!(item, Item::TraitAlias(_)) =>
        {
            Ok(Some(item))
        },

        // ...and undefined ones are errors, if ever
        _ => unimplemented!(),
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
    let item: Item = match syn::parse2(input) {
        Ok(item) => item,
        Err(err) => {
            return Err(Diagnostic::spanned(input_span, Level::Error, err.to_string()));
        },
    };

    // Generate new impls from the parsed one for every version in the `versions`
    let mut impls: Vec<TokenStream2> = Vec::with_capacity(versions.0.len());
    for version in &versions.0 {
        // Collect the filtered version of the implementation
        let mut item: Item = match filter_item(item.clone(), &versions, version)? {
            Some(item) => item,
            // Filtered out
            None => continue,
        };

        // Update the toplevel item to be wrapped in a version thing
        let item: Item = if matches!(item, Item::Mod(_)) && !opts.nest_toplevel_modules {
            // Special case where the toplevel module is transformed instead of wrapped; so update the name and return
            if let Item::Mod(mut m) = item {
                m.ident = version.0.clone();
                Item::Mod(m)
            } else {
                unreachable!();
            }
        } else {
            // "Steal" the visibility of the wrapped item
            let module_vis: Visibility = if let Some(vis) = item_vis_mut(&mut item) {
                let old_vis: Visibility = vis.clone();
                *vis = Visibility::Public(Pub { span: old_vis.span() });
                old_vis
            } else {
                Visibility::Inherited
            };

            // Wrap it in a new module
            Item::Mod(ItemMod {
                attrs: vec![],
                vis: module_vis,
                unsafety: None,
                mod_token: Mod { span: Span::call_site() },
                ident: version.0.clone(),
                content: Some((Brace::default(), vec![item])),
                semi: None,
            })
        };

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
        impls.push(quote! {
            #cfg
            #item
        });
    }

    // Done
    Ok(quote! {
        #(#impls)*
    })
}

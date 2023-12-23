//  VERSIONING.rs
//    by Lut99
//
//  Created:
//    19 Nov 2023, 19:25:25
//  Last edited:
//    23 Dec 2023, 15:46:37
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the toplevel of the `#[versioned(...)]`-macro.
//

use std::borrow::Cow;

use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Comma, Pub};
use syn::{
    Attribute, Expr, ExprLit, Field, Fields, FieldsNamed, FieldsUnnamed, ForeignItem, ForeignItemFn, ForeignItemMacro, ForeignItemStatic,
    ForeignItemType, Ident, ImplItem, ImplItemConst, ImplItemFn, ImplItemMacro, ImplItemType, Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn,
    ItemForeignMod, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Lit, LitBool,
    Meta, TraitItem, TraitItemConst, TraitItemFn, TraitItemMacro, TraitItemType, Variant, Visibility,
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
/// Gets the attributes of an [`Item`].
///
/// # Arguments
/// - `item`: A(n) (reference to the) [`Item`] of which to return the attributes.
///
/// # Returns
/// A list of [`Attribute`]s.
#[inline]
fn item_attrs(item: &Item) -> Option<&[Attribute]> {
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
        | Item::TraitAlias(ItemTraitAlias { attrs, .. })
        | Item::Type(ItemType { attrs, .. })
        | Item::Union(ItemUnion { attrs, .. })
        | Item::Use(ItemUse { attrs, .. }) => Some(attrs),

        // Vertabim doesn't have attrs, unfortunately
        Item::Verbatim(_) => None,

        // And any others, 'cuz non-exhaustive ;(
        other => panic!("Encountered unknown Item variant '{other:?}'"),
    }
}
/// Gets the visibility of an [`Item`].
///
/// # Arguments
/// - `item`: A(n) (reference to the) [`Item`] of which to return the attributes.
///
/// # Returns
/// A reference to the [`Visibility`], or [`None`] if this variant does not have any.
#[inline]
fn item_vis(item: &Item) -> Option<&Visibility> {
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
        | Item::TraitAlias(ItemTraitAlias { vis, .. })
        | Item::Type(ItemType { vis, .. })
        | Item::Union(ItemUnion { vis, .. })
        | Item::Use(ItemUse { vis, .. }) => Some(vis),

        // All the ones we know that _don't_ have visibility
        Item::ForeignMod(ItemForeignMod { .. }) | Item::Impl(ItemImpl { .. }) | Item::Macro(ItemMacro { .. }) | Item::Verbatim(_) => None,

        // And any others, 'cuz non-exhaustive ;(
        other => panic!("Encountered unknown Item variant '{other:?}'"),
    }
}

/// Gets the attributes of a [`TraitItem`].
///
/// # Arguments
/// - `item`: A (reference to the) [`TraitItem`] of which to return the attributes.
///
/// # Returns
/// A list of [`Attribute`]s, or [`None`] if this variant does not have any.
#[inline]
fn trait_item_attrs(item: &TraitItem) -> Option<&[Attribute]> {
    match item {
        // All the ones we know
        TraitItem::Const(TraitItemConst { attrs, .. })
        | TraitItem::Fn(TraitItemFn { attrs, .. })
        | TraitItem::Macro(TraitItemMacro { attrs, .. })
        | TraitItem::Type(TraitItemType { attrs, .. }) => Some(attrs),

        // Except for the vertabim; that one doesn't have any attrs
        TraitItem::Verbatim(_) => None,

        // And any others, 'cuz non-exhaustive ;(
        other => panic!("Encountered unknown TraitItem variant '{other:?}'"),
    }
}

/// Gets the attributes of a [`ForeignItem`].
///
/// # Arguments
/// - `item`: A (reference to the) [`ForeignItem`] of which to return the attributes.
///
/// # Returns
/// A list of [`Attribute`]s, or [`None`] if this variant does not have any.
#[inline]
fn foreign_item_attrs(item: &ForeignItem) -> Option<&[Attribute]> {
    match item {
        // All the ones we know
        ForeignItem::Fn(ForeignItemFn { attrs, .. })
        | ForeignItem::Macro(ForeignItemMacro { attrs, .. })
        | ForeignItem::Static(ForeignItemStatic { attrs, .. })
        | ForeignItem::Type(ForeignItemType { attrs, .. }) => Some(attrs),

        // Except for the vertabim; that one doesn't have any attrs
        ForeignItem::Verbatim(_) => None,

        // And any others, 'cuz non-exhaustive ;(
        other => panic!("Encountered unknown ForeignItem variant '{other:?}'"),
    }
}

/// Gets the attributes of an [`ImplItem`].
///
/// # Arguments
/// - `item`: A (reference to the) [`ForeignItem`] of which to return the attributes.
///
/// # Returns
/// A list of [`Attribute`]s, or [`None`] if this variant does not have any.
#[inline]
fn impl_item_attrs(item: &ImplItem) -> Option<&[Attribute]> {
    match item {
        // All the ones we know
        ImplItem::Const(ImplItemConst { attrs, .. })
        | ImplItem::Fn(ImplItemFn { attrs, .. })
        | ImplItem::Macro(ImplItemMacro { attrs, .. })
        | ImplItem::Type(ImplItemType { attrs, .. }) => Some(attrs),

        // Except for the vertabim; that one doesn't have any attrs
        ImplItem::Verbatim(_) => None,

        // And any others, 'cuz non-exhaustive ;(
        other => panic!("Encountered unknown ImplItem variant '{other:?}'"),
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
/// # Arguments
/// - `attrs`: The attributes to read.
///
/// # Returns
/// The [`VersionFilterList`] specified in the `#[version(...)]`-macro if it was found, or else [`None`] if the macro wasn't given.
fn get_version_attr(attrs: &[Attribute]) -> Result<Option<VersionFilter>, Diagnostic> {
    // Iterate over the attributes
    for attr in attrs {
        match &attr.meta {
            Meta::List(l) => {
                if l.path.is_ident("version") {
                    return match l.parse_args() {
                        Ok(filter) => {
                            // Before we return, remove from the list
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



/// Filters the given attributes minus the `#[version(...)]`-attribute and compiles it to a [`TokenStream2`].
///
/// # Arguments
/// - `attrs`: The list of [`Attribute`]s to filter.
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
fn generate_attrs(attrs: &[Attribute]) -> TokenStream2 {
    // Serialize them all, except the ones we don't like
    let mut stream: TokenStream2 = TokenStream2::new();
    for attr in attrs {
        // See if it's a match
        if !attr.path().is_ident("version") {
            stream.extend(quote! { #attr });
        }
    }
    stream
}

/// Filters the given field in accordance to the list of versions and compiles it to a [`TokenStream2`].
///
/// # Arguments
/// - `field`: The [`Field`] to filter.
/// - `versions`: The list of versions in total (allows us to define order)
/// - `version`: The current version to filter for.
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
fn generate_filtered_field(field: &Field, versions: &VersionList, version: &Version) -> Result<Option<TokenStream2>, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    if let Some(filter) = get_version_attr(&field.attrs)? {
        // Next, see if this matches the current version
        filter.verify(versions)?;
        if !filter.matches(versions, version) {
            // Filtered oot!
            return Ok(None);
        }
    }

    // Otherwise, serialize with adapted attributes
    let Field { attrs, vis, mutability: _, ident, colon_token, ty } = field;
    let mut stream: TokenStream2 = generate_attrs(attrs);
    stream.extend(quote! { #vis #ident #colon_token #ty });
    Ok(Some(stream))
}

/// Filters the given enum variant in accordance to the list of versions and compiles it to a [`TokenStream2`].
///
/// # Arguments
/// - `variant`: The [`Variant`] to filter.
/// - `versions`: The list of versions in total (allows us to define order)
/// - `version`: The current version to filter for.
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
fn generate_filtered_variant(variant: &Variant, versions: &VersionList, version: &Version) -> Result<Option<TokenStream2>, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    if let Some(filter) = get_version_attr(&variant.attrs)? {
        // Next, see if this matches the current version
        filter.verify(versions)?;
        if !filter.matches(versions, version) {
            // Filtered oot!
            return Ok(None);
        }
    }

    // Serialize the attributes as first part of the module
    let Variant { attrs, ident, fields, discriminant } = variant;
    let mut stream: TokenStream2 = generate_attrs(attrs);
    stream.extend(quote! { #ident });
    // Serialize the fields, if any
    match fields {
        Fields::Named(named) => {
            // Serialize the field as a whole
            let FieldsNamed { named, brace_token } = named;
            let mut children: TokenStream2 = TokenStream2::new();
            for pair in named.pairs() {
                // Only add filtered ones too
                let (field, comma): (&Field, Option<&Comma>) = pair.into_tuple();
                if let Some(stream) = generate_filtered_field(field, versions, version)? {
                    children.extend(stream);
                    children.extend(quote! { #comma });
                }
            }
            // Add the whole thing
            brace_token.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));
        },
        Fields::Unnamed(unnamed) => {
            // Serialize the field as a whole
            let FieldsUnnamed { unnamed, paren_token } = unnamed;
            let mut children: TokenStream2 = TokenStream2::new();
            for pair in unnamed.pairs() {
                // Only add filtered ones too
                let (field, comma): (&Field, Option<&Comma>) = pair.into_tuple();
                if let Some(stream) = generate_filtered_field(field, versions, version)? {
                    children.extend(stream);
                    children.extend(quote! { #comma });
                }
            }
            // Add the whole thing
            paren_token.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));
        },
        Fields::Unit => {},
    }
    // Serialize the discriminant, if any
    if let Some((eq, expr)) = discriminant {
        stream.extend(quote! { #eq #expr });
    }

    // Done!
    Ok(Some(stream))
}

/// Filters the given trait item in accordance to the list of versions and compiles it to a [`TokenStream2`].
///
/// # Arguments
/// - `item`: The [`TraitItem`] to filter.
/// - `versions`: The list of versions in total (allows us to define order)
/// - `version`: The current version to filter for.
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
fn generate_filtered_trait_item(item: &TraitItem, versions: &VersionList, version: &Version) -> Result<Option<TokenStream2>, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    if let Some(attrs) = trait_item_attrs(item) {
        if let Some(filter) = get_version_attr(attrs)? {
            // Next, see if this matches the current version
            filter.verify(versions)?;
            if !filter.matches(versions, version) {
                // Filtered oot!
                return Ok(None);
            }
        }
    }

    // Match after all (third time we're writing this) to filter oot some attributes
    match item {
        TraitItem::Const(TraitItemConst { attrs, const_token, ident, generics, colon_token, ty, default, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #const_token #ident #generics #colon_token #ty });
            if let Some((eq_token, expr)) = default {
                stream.extend(quote! { #eq_token #expr })
            }
            stream.extend(quote! { #semi_token });
            Ok(Some(stream))
        },
        TraitItem::Fn(TraitItemFn { attrs, sig, default, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #sig #default #semi_token });
            Ok(Some(stream))
        },
        TraitItem::Macro(TraitItemMacro { attrs, mac, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #mac #semi_token });
            Ok(Some(stream))
        },
        TraitItem::Type(TraitItemType { attrs, type_token, ident, generics, colon_token, bounds, default, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #type_token #ident #generics #colon_token #bounds });
            if let Some((eq_token, expr)) = default {
                stream.extend(quote! { #eq_token #expr })
            }
            stream.extend(quote! { #semi_token });
            Ok(Some(stream))
        },

        // Vertabim is passed as-is
        TraitItem::Verbatim(v) => Ok(Some(quote! { #v })),

        // And any others, 'cuz non-exhaustive ;(
        other => panic!("Encountered unknown TraitItem variant '{other:?}'"),
    }
}

/// Filters the given foreign item in accordance to the list of versions and compiles it to a [`TokenStream2`].
///
/// # Arguments
/// - `item`: The [`ForeignItem`] to filter.
/// - `versions`: The list of versions in total (allows us to define order)
/// - `version`: The current version to filter for.
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
fn generate_filtered_foreign_item(item: &ForeignItem, versions: &VersionList, version: &Version) -> Result<Option<TokenStream2>, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    if let Some(attrs) = foreign_item_attrs(item) {
        if let Some(filter) = get_version_attr(attrs)? {
            // Next, see if this matches the current version
            filter.verify(versions)?;
            if !filter.matches(versions, version) {
                // Filtered oot!
                return Ok(None);
            }
        }
    }

    // Also needs to be manually matches to skip attributes
    match item {
        ForeignItem::Fn(ForeignItemFn { attrs, vis, sig, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #vis #sig #semi_token });
            Ok(Some(stream))
        },
        ForeignItem::Macro(ForeignItemMacro { attrs, mac, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #mac #semi_token });
            Ok(Some(stream))
        },
        ForeignItem::Static(ForeignItemStatic { attrs, vis, static_token, mutability, ident, colon_token, ty, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #vis #static_token #mutability #ident #colon_token #ty #semi_token });
            Ok(Some(stream))
        },
        ForeignItem::Type(ForeignItemType { attrs, vis, type_token, ident, generics, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #vis #type_token #ident #generics #semi_token });
            Ok(Some(stream))
        },

        // Vertabim is passed as-is
        ForeignItem::Verbatim(v) => Ok(Some(quote! { #v })),

        // And any others, 'cuz non-exhaustive ;(
        other => panic!("Encountered unknown ForeignItem variant '{other:?}'"),
    }
}

/// Filters the given impl item in accordance to the list of versions and compiles it to a [`TokenStream2`].
///
/// # Arguments
/// - `item`: The [`ImplItem`] to filter.
/// - `versions`: The list of versions in total (allows us to define order)
/// - `version`: The current version to filter for.
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
fn generate_filtered_impl_item(item: &ImplItem, versions: &VersionList, version: &Version) -> Result<Option<TokenStream2>, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    if let Some(attrs) = impl_item_attrs(item) {
        if let Some(filter) = get_version_attr(attrs)? {
            // Next, see if this matches the current version
            filter.verify(versions)?;
            if !filter.matches(versions, version) {
                // Filtered oot!
                return Ok(None);
            }
        }
    }

    // Also needs to be manually matches to skip attributes
    match item {
        ImplItem::Const(ImplItemConst { attrs, vis, defaultness, const_token, ident, generics, colon_token, ty, eq_token, expr, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #vis #defaultness #const_token #ident #generics #colon_token #ty #eq_token #expr #semi_token });
            Ok(Some(stream))
        },
        ImplItem::Fn(ImplItemFn { attrs, vis, defaultness, sig, block }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #vis #defaultness #sig #block });
            Ok(Some(stream))
        },
        ImplItem::Macro(ImplItemMacro { attrs, mac, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #mac #semi_token });
            Ok(Some(stream))
        },
        ImplItem::Type(ImplItemType { attrs, vis, defaultness, type_token, ident, generics, eq_token, ty, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #vis #defaultness #type_token #ident #generics #eq_token #ty #semi_token });
            Ok(Some(stream))
        },

        // Vertabim is passed as-is
        ImplItem::Verbatim(v) => Ok(Some(quote! { #v })),

        // And any others, 'cuz non-exhaustive ;(
        other => panic!("Encountered unknown ImplItem variant '{other:?}'"),
    }
}

/// Filters the given body item in accordance to the list of versions and compiles it to a [`TokenStream2`].
///
/// # Arguments
/// - `item`: The [`BodyItem`] to filter.
/// - `versions`: The list of versions in total (allows us to define order)
/// - `version`: The current version to filter for.
/// - `toplevel`: Only true for the first depth of recursion.
/// - `force_public`: If given, always writes a `pub` for his item (in case it's nested in a version module). Note that nested modules are always hardcoded to `false`.
///
/// # Returns
/// A new [`TokenStream2`] that encodes the body item but without certain components if filtered out by the version.
fn generate_filtered_item(
    item: &Item,
    versions: &VersionList,
    version: &Version,
    toplevel: bool,
    force_public: bool,
) -> Result<Option<TokenStream2>, Diagnostic> {
    // First, check the item's attributes to see if it has been version filtered
    if let Some(attrs) = item_attrs(item) {
        if let Some(filter) = get_version_attr(attrs)? {
            // Next, see if this matches the current version
            filter.verify(versions)?;
            if !filter.matches(versions, version) {
                // Filtered oot!
                return Ok(None);
            }
        }
    }

    // Then recurse if necessary
    match item {
        // These all have some kind of recursion going on
        Item::Mod(ItemMod { attrs, vis, unsafety, mod_token, ident, content, semi }) => {
            // Serialize the attributes as first part of the module
            let mut stream: TokenStream2 = generate_attrs(attrs);
            // Serialize the visibility
            if force_public {
                let vis: Visibility = Visibility::Public(Pub { span: vis.span() });
                stream.extend(quote! { #vis });
            } else {
                stream.extend(quote! { #vis });
            }
            // Serialize some other parts
            stream.extend(quote! {
                #unsafety #mod_token
            });
            // Serialize the indent, which is overridden is we _are_ toplevel but _not_ wrapping
            if toplevel && !force_public {
                let ident: &Ident = &version.0;
                stream.extend(quote! { #ident });
            } else {
                stream.extend(quote! { #ident });
            }
            // Serialize content if there is any
            if let Some((brace, items)) = content {
                // Serialize all children
                let mut children: TokenStream2 = TokenStream2::new();
                for item in items {
                    // Only keep OK ones
                    if let Some(stream) = generate_filtered_item(item, versions, version, false, false)? {
                        children.extend(stream);
                    }
                }
                // Serialize them with braces
                brace.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));
            };
            // Serialize the possible brace
            stream.extend(quote! {
                #semi
            });

            // OK, let's return!
            Ok(Some(stream))
        },

        Item::Enum(ItemEnum { attrs, vis, enum_token, ident, generics, brace_token, variants }) => {
            // First, serialize the attributes
            let mut stream: TokenStream2 = generate_attrs(attrs);
            // Serialize the visibility
            if force_public {
                let vis: Visibility = Visibility::Public(Pub { span: vis.span() });
                stream.extend(quote! { #vis });
            } else {
                stream.extend(quote! { #vis });
            }
            // Serialize some other parts
            stream.extend(quote! {
                #enum_token #ident #generics
            });
            // Add the variants wrapped in braces
            let mut children: TokenStream2 = TokenStream2::new();
            for pair in variants.pairs() {
                let (variant, comma): (&Variant, Option<&Comma>) = pair.into_tuple();
                if let Some(stream) = generate_filtered_variant(variant, versions, version)? {
                    children.extend(stream);
                    children.extend(quote! { #comma });
                }
            }
            brace_token.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));

            // Done
            Ok(Some(stream))
        },
        Item::Struct(ItemStruct { attrs, vis, struct_token, ident, generics, fields, semi_token }) => {
            // First, serialize the attributes
            let mut stream: TokenStream2 = generate_attrs(attrs);
            // Serialize the visibility
            if force_public {
                let vis: Visibility = Visibility::Public(Pub { span: vis.span() });
                stream.extend(quote! { #vis });
            } else {
                stream.extend(quote! { #vis });
            }
            // Serialize some other parts
            stream.extend(quote! {
                #struct_token #ident #generics
            });
            // Serialize the fields, if any
            match fields {
                Fields::Named(named) => {
                    // Serialize the field as a whole
                    let FieldsNamed { named, brace_token } = named;
                    let mut children: TokenStream2 = TokenStream2::new();
                    for pair in named.pairs() {
                        // Only add filtered ones too
                        let (field, comma): (&Field, Option<&Comma>) = pair.into_tuple();
                        if let Some(stream) = generate_filtered_field(field, versions, version)? {
                            children.extend(stream);
                            children.extend(quote! { #comma });
                        }
                    }
                    // Add the whole thing
                    brace_token.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));
                },
                Fields::Unnamed(unnamed) => {
                    // Serialize the field as a whole
                    let FieldsUnnamed { unnamed, paren_token } = unnamed;
                    let mut children: TokenStream2 = TokenStream2::new();
                    for pair in unnamed.pairs() {
                        // Only add filtered ones too
                        let (field, comma): (&Field, Option<&Comma>) = pair.into_tuple();
                        if let Some(stream) = generate_filtered_field(field, versions, version)? {
                            children.extend(stream);
                            children.extend(quote! { #comma });
                        }
                    }
                    // Add the whole thing
                    paren_token.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));
                },
                Fields::Unit => {},
            }
            // Generate the remaining
            stream.extend(quote! { #semi_token });

            // OK, return
            Ok(Some(stream))
        },
        Item::Union(ItemUnion { attrs, vis, union_token, ident, generics, fields }) => {
            // First, serialize the attributes
            let mut stream: TokenStream2 = generate_attrs(attrs);
            // Serialize the visibility
            if force_public {
                let vis: Visibility = Visibility::Public(Pub { span: vis.span() });
                stream.extend(quote! { #vis });
            } else {
                stream.extend(quote! { #vis });
            }
            // Serialize some other parts
            stream.extend(quote! {
                #union_token #ident #generics
            });
            // Serialize the named fields as a whole
            let FieldsNamed { named, brace_token } = fields;
            let mut children: TokenStream2 = TokenStream2::new();
            for pair in named.pairs() {
                // Only add filtered ones too
                let (field, comma): (&Field, Option<&Comma>) = pair.into_tuple();
                if let Some(stream) = generate_filtered_field(field, versions, version)? {
                    children.extend(stream);
                    children.extend(quote! { #comma });
                }
            }
            // Add the whole thing
            brace_token.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));

            // Done!
            Ok(Some(stream))
        },
        Item::Trait(ItemTrait {
            attrs,
            vis,
            unsafety,
            auto_token,
            restriction,
            trait_token,
            ident,
            generics,
            colon_token,
            supertraits,
            brace_token,
            items,
        }) => {
            // First, serialize the attributes
            let mut stream: TokenStream2 = generate_attrs(attrs);
            // Serialize the visibility
            if force_public {
                let vis: Visibility = Visibility::Public(Pub { span: vis.span() });
                stream.extend(quote! { #vis });
            } else {
                stream.extend(quote! { #vis });
            }
            // Serialize some other parts
            stream.extend(quote! { #unsafety #auto_token });
            // Serialize the restriction(?)
            if restriction.is_some() {
                unimplemented!();
            }
            // Finally serialize 'trait ...'
            stream.extend(quote! { #trait_token #ident #generics #colon_token #supertraits });
            // Serialize the items in the trait
            let mut children: TokenStream2 = TokenStream2::new();
            for item in items {
                // Only serialize those that match the filter test
                if let Some(stream) = generate_filtered_trait_item(item, versions, version)? {
                    children.extend(stream);
                }
            }
            brace_token.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));

            // Done!
            Ok(Some(stream))
        },

        Item::ForeignMod(ItemForeignMod { attrs, unsafety, abi, brace_token, items }) => {
            // Since it doesn't have visibility, error if we would be forced to update it
            if force_public {
                return Err(Diagnostic::spanned(
                    item.span(),
                    Level::Error,
                    "Cannot use `#[versioning(...)]` on a foreign block; instead, wrap it in a module to decide internal visibility yourself".into(),
                ));
            }

            // First, serialize the attributes
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! {
                #unsafety #abi
            });
            // Serialize the foreign items in the trait
            let mut children: TokenStream2 = TokenStream2::new();
            for item in items {
                // Only serialize those that match the filter test
                if let Some(stream) = generate_filtered_foreign_item(item, versions, version)? {
                    children.extend(stream);
                }
            }
            brace_token.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));

            // Done!
            Ok(Some(stream))
        },
        Item::Impl(ItemImpl { attrs, defaultness, unsafety, impl_token, generics, trait_, self_ty, brace_token, items }) => {
            // Since it doesn't have visibility, error if we would be forced to update it
            if force_public {
                return Err(Diagnostic::spanned(
                    item.span(),
                    Level::Error,
                    "Cannot use `#[versioning(...)]` on an impl block; instead, wrap it in a module to decide internal visibility yourself".into(),
                ));
            }

            // Serialize as far as we can before it gets gnarly
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! {
                #defaultness #unsafety #impl_token #generics
            });
            // Serialize the 'for trait' part, if any
            if let Some((not, name, for_token)) = trait_ {
                stream.extend(quote! { #not #name #for_token });
            }
            // Serialize the type
            stream.extend(quote! { #self_ty });
            // Serialize the items
            let mut children: TokenStream2 = TokenStream2::new();
            for item in items {
                // Keep only non-filtered items
                if let Some(stream) = generate_filtered_impl_item(item, versions, version)? {
                    children.extend(stream);
                }
            }
            brace_token.surround(&mut stream, |stream: &mut TokenStream2| stream.extend(children));

            // Done!
            Ok(Some(stream))
        },

        // For these, just mod the visibility if told to do so
        Item::Const(ItemConst { attrs, vis, const_token, ident, generics, colon_token, ty, eq_token, expr, semi_token }) => {
            let vis: Cow<Visibility> = if force_public { Cow::Owned(Visibility::Public(Pub { span: vis.span() })) } else { Cow::Borrowed(vis) };
            let mut stream = generate_attrs(attrs);
            stream.extend(quote! {
                #vis #const_token #ident #generics #colon_token #ty #eq_token #expr #semi_token
            });
            Ok(Some(stream))
        },
        Item::ExternCrate(ItemExternCrate { attrs, vis, extern_token, crate_token, ident, rename, semi_token }) => {
            let vis: Cow<Visibility> = if force_public { Cow::Owned(Visibility::Public(Pub { span: vis.span() })) } else { Cow::Borrowed(vis) };
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! {
                #vis #extern_token #crate_token #ident
            });
            if let Some((as_token, name)) = rename {
                stream.extend(quote! { #as_token, #name });
            }
            stream.extend(quote! { #semi_token });
            Ok(Some(stream))
        },
        Item::Fn(ItemFn { attrs, vis, sig, block }) => {
            // For now, function bodies are not yet nested
            let vis: Cow<Visibility> = if force_public { Cow::Owned(Visibility::Public(Pub { span: vis.span() })) } else { Cow::Borrowed(vis) };
            let mut stream = generate_attrs(attrs);
            stream.extend(quote! {
                #vis #sig #block
            });
            Ok(Some(stream))
        },
        Item::Macro(ItemMacro { attrs, ident, mac, semi_token }) => {
            let mut stream: TokenStream2 = generate_attrs(attrs);
            stream.extend(quote! { #ident #mac #semi_token });
            Ok(Some(stream))
        },
        Item::Static(ItemStatic { attrs, vis, static_token, mutability, ident, colon_token, ty, eq_token, expr, semi_token }) => {
            let vis: Cow<Visibility> = if force_public { Cow::Owned(Visibility::Public(Pub { span: vis.span() })) } else { Cow::Borrowed(vis) };
            let mut stream = generate_attrs(attrs);
            stream.extend(quote! {
                #vis #static_token #mutability #ident #colon_token #ty #eq_token #expr #semi_token
            });
            Ok(Some(stream))
        },
        Item::TraitAlias(ItemTraitAlias { attrs, vis, trait_token, ident, generics, eq_token, bounds, semi_token }) => {
            let vis: Cow<Visibility> = if force_public { Cow::Owned(Visibility::Public(Pub { span: vis.span() })) } else { Cow::Borrowed(vis) };
            let mut stream = generate_attrs(attrs);
            stream.extend(quote! {
                #vis #trait_token #ident #generics #eq_token #bounds #semi_token
            });
            Ok(Some(stream))
        },
        Item::Type(ItemType { attrs, vis, type_token, ident, generics, eq_token, ty, semi_token }) => {
            let vis: Cow<Visibility> = if force_public { Cow::Owned(Visibility::Public(Pub { span: vis.span() })) } else { Cow::Borrowed(vis) };
            let mut stream = generate_attrs(attrs);
            stream.extend(quote! {
                #vis #type_token #ident #generics #eq_token #ty #semi_token
            });
            Ok(Some(stream))
        },
        Item::Use(ItemUse { attrs, vis, use_token, leading_colon, tree, semi_token }) => {
            let vis: Cow<Visibility> = if force_public { Cow::Owned(Visibility::Public(Pub { span: vis.span() })) } else { Cow::Borrowed(vis) };
            let mut stream = generate_attrs(attrs);
            stream.extend(quote! {
                #vis #use_token #leading_colon #tree #semi_token
            });
            Ok(Some(stream))
        },

        // These can be passed as-is
        Item::Verbatim(v) => Ok(Some(quote! { #v })),

        // ...and undefined ones are errors, if ever
        other => panic!("Encountered unknown Item variant '{other:?}'"),
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
        let wrap_in_mod: bool = !matches!(item, Item::Mod(_)) || opts.nest_toplevel_modules;
        let old_vis: Option<&Visibility> = item_vis(&item);
        let mut stream: TokenStream2 = match generate_filtered_item(&item, &versions, version, true, wrap_in_mod)? {
            Some(item) => item,
            // Filtered out
            None => continue,
        };

        // If we are wrapping, then do so
        if wrap_in_mod {
            // Resolve the input
            let ident: &Ident = &version.0;
            let vis: Cow<Visibility> = if let Some(old_vis) = old_vis { Cow::Borrowed(old_vis) } else { Cow::Owned(Visibility::Inherited) };
            // Wrap
            stream = quote! {
                #vis mod #ident {
                    #stream
                }
            };
        }

        // Check if we need to inject the feature gate or not
        if opts.features {
            let feature: String = version.0.to_string();
            stream = quote! {
                #[cfg(feature = #feature)]
                #stream
            };
        }

        // Epic, store it!
        impls.push(stream);
    }

    // Done
    Ok(quote! {
        #(#impls)*
    })
}

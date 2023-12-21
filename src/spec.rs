//  SPEC.rs
//    by Lut99
//
//  Created:
//    20 Nov 2023, 13:02:02
//  Last edited:
//    21 Dec 2023, 10:03:23
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the structure of things we're parsing from the source code.
//

use proc_macro2::Span;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::spanned::Spanned as _;
use syn::token::{Mod, Semi, Unsafe};
use syn::{Attribute, Ident, Item, ItemEnum, ItemImpl, ItemMod, ItemStruct, Visibility};


/***** LIBRARY *****/
/// Defines a single "item" (i.e., statement) in the versionable [`Body`].
#[derive(Clone, Debug)]
pub enum BodyItem {
    /// It's a module
    Module(Vec<Attribute>, Visibility, Option<Unsafe>, Ident, Vec<Self>, Mod, Option<Semi>, Span),
    /// It's an enum
    Enum(ItemEnum),
    /// It's a struct
    Struct(ItemStruct),
    /// It's an impl block
    Impl(ItemImpl),
}
impl BodyItem {
    /// Returns the attributes of this item.
    ///
    /// # Returns
    /// A mutable vector of [`Attribute`]s part of this body.
    pub fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        match self {
            Self::Module(attrs, _, _, _, _, _, _, _) => attrs,
            Self::Enum(e) => &mut e.attrs,
            Self::Struct(s) => &mut s.attrs,
            Self::Impl(i) => &mut i.attrs,
        }
    }

    /// Returns the visibility of this item.
    ///
    /// # Returns
    /// A mutable [`Visibility`] determining how public this item is.
    pub fn vis_mut(&mut self) -> Option<&mut Visibility> {
        match self {
            Self::Module(_, vis, _, _, _, _, _, _) => Some(vis),
            Self::Enum(e) => Some(&mut e.vis),
            Self::Struct(s) => Some(&mut s.vis),
            Self::Impl(_) => None,
        }
    }
}
impl Parse for BodyItem {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> { input.call(|buf: &ParseBuffer| -> syn::Result<Self> { buf.parse::<Item>()?.try_into() }) }
}
impl TryFrom<Item> for BodyItem {
    type Error = syn::Error;

    #[inline]
    fn try_from(value: Item) -> Result<Self, Self::Error> {
        match value {
            Item::Mod(m) => m.try_into(),
            Item::Enum(e) => e.try_into(),
            Item::Struct(s) => s.try_into(),
            Item::Impl(i) => i.try_into(),
            other => return Err(syn::Error::new(other.span(), "Expected mode, struct, enum or impl definition")),
        }
    }
}
impl TryFrom<ItemMod> for BodyItem {
    type Error = syn::Error;

    #[inline]
    fn try_from(value: ItemMod) -> Result<Self, Self::Error> {
        let span: Span = value.span();
        let ItemMod { attrs, vis, unsafety, ident, content, mod_token, semi } = value;

        // Recursively convert the contents tho
        let mut contents: Vec<Self> = Vec::with_capacity(content.as_ref().map(|contents| contents.1.len()).unwrap_or(0));
        if let Some((_, items)) = content {
            for item in items {
                contents.push(item.try_into()?);
            }
        }

        // Done
        Ok(Self::Module(attrs, vis, unsafety, ident, contents, mod_token, semi, span))
    }
}
impl TryFrom<ItemEnum> for BodyItem {
    type Error = syn::Error;

    #[inline]
    fn try_from(value: ItemEnum) -> Result<Self, Self::Error> { Ok(Self::Enum(value)) }
}
impl TryFrom<ItemStruct> for BodyItem {
    type Error = syn::Error;

    #[inline]
    fn try_from(value: ItemStruct) -> Result<Self, Self::Error> { Ok(Self::Struct(value)) }
}
impl TryFrom<ItemImpl> for BodyItem {
    type Error = syn::Error;

    #[inline]
    fn try_from(value: ItemImpl) -> Result<Self, Self::Error> { Ok(Self::Impl(value)) }
}

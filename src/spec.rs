//  SPEC.rs
//    by Lut99
//
//  Created:
//    20 Nov 2023, 13:02:02
//  Last edited:
//    27 Nov 2023, 16:34:57
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the structure of things we're parsing from the source code.
//

use proc_macro2::Span;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::spanned::Spanned as _;
use syn::{Attribute, Ident, Item, ItemEnum, ItemMod, ItemStruct, Visibility};


/***** LIBRARY *****/
// /// Defines the toplevel module that we parsed, which lists all the structs and whatnot to version.
// #[derive(Clone, Debug)]
// pub struct Body {
//     /// The publicity of this module.
//     pub vis:   Visibility,
//     /// The list of BodyItems to parse.
//     pub items: Vec<BodyItem>,
// }
// impl Parse for Body {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         // Parse as a module
//         let input_span: Span = input.span();
//         if let Ok(m) = input.parse::<ItemMod>() {
//             // Extract the valid items from it, if any
//             if let Some(content) = m.content {
//                 let mut items: Vec<BodyItem> = Vec::with_capacity(content.1.len());
//                 for item in content.1 {
//                     match item {
//                         Item::Mod(m) =>
//                         Item::Enum(e) => items.push(BodyItem::Enum(e)),
//                         Item::Struct(s) => items.push(BodyItem::Struct(s)),
//                         other => return Err(syn::Error::new(other.span(), "Expected struct or enum definition")),
//                     }
//                 }
//                 Ok(Self { vis: m.vis, items })
//             } else {
//                 Ok(Self { vis: m.vis, items: vec![] })
//             }
//         } else {
//             Err(syn::Error::new(input_span, "Expected module"))
//         }
//     }
// }

/// Defines a single "item" (i.e., statement) in the versionable [`Body`].
#[derive(Clone, Debug)]
pub enum BodyItem {
    /// It's a module
    Module(Vec<Attribute>, Visibility, Ident, Vec<Self>, ModSpan),
    /// It's an enum
    Enum(ItemEnum),
    /// It's a struct
    Struct(ItemStruct),
}
impl BodyItem {
    /// Returns the attributes of this item.
    ///
    /// # Returns
    /// A slice of [`Attribute`]s part of this body.
    pub fn attrs(&self) -> &[Attribute] {
        match self {
            Self::Module(attrs, _, _, _, _) => attrs,
            Self::Enum(e) => &e.attrs,
            Self::Struct(s) => &s.attrs,
        }
    }

    /// Returns the visibility of this item.
    ///
    /// # Returns
    /// A [`Visibility`] determining how public this item is.
    pub fn vis(&self) -> &Visibility {
        match self {
            Self::Module(_, vis, _, _, _) => vis,
            Self::Enum(e) => &e.vis,
            Self::Struct(s) => &s.vis,
        }
    }

    /// Returns the span of this item.
    ///
    /// # Returns
    /// A [`Span`] with the source location.
    pub fn span(&self) -> Span {
        match self {
            Self::Module(_, _, _, _, span) => *span,
            Self::Enum(e) => e.span(),
            Self::Struct(s) => s.span(),
        }
    }
}
impl Parse for BodyItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.call(|buf: &ParseBuffer| -> syn::Result<Self> {
            // Lookahead for the token
            if let Ok(m) = buf.parse::<ItemMod>() {
                m.try_into()
            } else if let Ok(e) = buf.parse::<ItemEnum>() {
                e.try_into()
            } else if let Ok(s) = buf.parse::<ItemStruct>() {
                s.try_into()
            } else {
                Err(buf.error("Expected module, struct or enum"))
            }
        })
    }
}
impl TryFrom<Item> for BodyItem {
    type Error = syn::Error;

    #[inline]
    fn try_from(value: Item) -> Result<Self, Self::Error> {
        match value {
            Item::Mod(m) => m.try_into(),
            Item::Enum(e) => e.try_into(),
            Item::Struct(s) => s.try_into(),
            other => return Err(syn::Error::new(other.span(), "Expected struct or enum definition")),
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

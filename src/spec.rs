//  SPEC.rs
//    by Lut99
//
//  Created:
//    20 Nov 2023, 13:02:02
//  Last edited:
//    21 Nov 2023, 22:14:16
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the structure of things we're parsing from the source code.
//

use proc_macro2::Span;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::spanned::Spanned as _;
use syn::{Item, ItemEnum, ItemMod, ItemStruct, Visibility};


/***** LIBRARY *****/
/// Defines the toplevel module that we parsed, which lists all the structs and whatnot to version.
#[derive(Clone, Debug)]
pub struct Body {
    /// The publicity of this module.
    pub vis:   Visibility,
    /// The list of BodyItems to parse.
    pub items: Vec<BodyItem>,
}
impl Parse for Body {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse as a module, struct or enum first
        let input_span: Span = input.span();
        if let Ok(m) = input.parse::<ItemMod>() {
            // Extract the valid items from it, if any
            if let Some(content) = m.content {
                let mut items: Vec<BodyItem> = Vec::with_capacity(content.1.len());
                for item in content.1 {
                    match item {
                        Item::Enum(e) => items.push(BodyItem::Enum(e)),
                        Item::Struct(s) => items.push(BodyItem::Struct(s)),
                        other => return Err(syn::Error::new(other.span(), "Expected struct or enum definition")),
                    }
                }
                Ok(Self { vis: m.vis, items })
            } else {
                Ok(Self { vis: m.vis, items: vec![] })
            }
        } else if let Ok(e) = input.parse::<ItemEnum>() {
            Ok(Self { vis: e.vis.clone(), items: vec![BodyItem::Enum(e)] })
        } else if let Ok(s) = input.parse::<ItemStruct>() {
            Ok(Self { vis: s.vis.clone(), items: vec![BodyItem::Struct(s)] })
        } else {
            Err(syn::Error::new(input_span, "Expected struct or enum"))
        }
    }
}

/// Defines a single "item" (i.e., statement) in the versionable [`Body`].
#[derive(Clone, Debug)]
pub enum BodyItem {
    /// It's an enum
    Enum(ItemEnum),
    /// It's a struct
    Struct(ItemStruct),
}
impl Parse for BodyItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.call(|buf: &ParseBuffer| -> syn::Result<Self> {
            // Lookahead for the token
            if let Ok(e) = buf.parse::<ItemEnum>() {
                Ok(Self::Enum(e))
            } else if let Ok(s) = buf.parse::<ItemStruct>() {
                Ok(Self::Struct(s))
            } else {
                Err(buf.error("Expected struct or enum"))
            }
        })
    }
}

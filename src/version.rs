//  VERSION.rs
//    by Lut99
//
//  Created:
//    21 Nov 2023, 22:07:03
//  Last edited:
//    21 Nov 2023, 22:30:09
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines a small expression language that can be used to filter
//!   versions.
//

use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, Ident, LitStr, Token};


/***** LIBRARY *****/
/// A version string that matches (part of) a filter.
///
/// Implemented as a string that matches the prefix of the version string in question, e.g.,
/// ```
/// "1.0" 
/// ```
/// matches all versions starting with `1.0`.
#[derive(Clone, Debug)]
pub struct Version(pub LitStr);
impl Parse for Version {
    fn parse(input: ParseStream) -> syn::Result<Self> { Ok(Self(input.parse()?)) }
}

/// A list of [`Version`]s that are defined.
///
/// Used in the [`#[versioning(...)]`](crate::versioning())-macro to define the list (and order) of versions.
#[derive(Clone, Debug)]
pub struct VersionList(pub Vec<Version>);
impl Parse for VersionList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.call(|buf: &ParseBuffer| -> syn::Result<Vec<Version>> {
            let mut res: Vec<Version> = vec![];
            while !buf.is_empty() {
                // Attempt to parse the next
                res.push(buf.parse()?);
                // Pop at least one comma if the buffer isn't empty
                if !buf.is_empty() {
                    let mut at_least_one: bool = false;
                    while buf.parse::<Token![,]>().is_ok() {
                        at_least_one = true;
                    }
                    if !at_least_one {
                        return Err(buf.error("Expected comma"));
                    }
                }
            }
            Ok(res)
        })?))
    }
}

/// A filter of versions, which implements a little expression tree that allows us to parse them.
///
/// Used in the `#[version(...)]` macro to match versions for which to implement an item.
#[derive(Clone, Debug)]
pub enum VersionFilter {
    /// It's a version string.
    Version(Version),
    /// It's a negation of a filter (i.e., anything _but_...)
    Not(Box<Self>),
    /// It's a disjunction between nested filters
    Any(Vec<Self>),
    /// It's a conjunction between nested filters
    All(Vec<Self>),
}
impl Parse for VersionFilter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // We can use a lookahead here
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            Ok(Self::Version(Version(input.parse()?)))
        } else if lookahead.peek(Ident) {
            // Check _which_ identifier
            let ident: Ident = input.parse()?;
            if ident == "not" {
                // Parse brackets, with a new version filter in between them
                let contents;
                parenthesized!(contents in input);
                let filter: VersionFilter = contents.parse()?;
                Ok(Self::Not(Box::new(filter)))
            } else if ident == "any" {
                // Parse brackets, with any number of version filters tokens in between them
                let contents;
                parenthesized!(contents in input);
                let filters: Punctuated<VersionFilter, Token![,]> = contents.parse_terminated(VersionFilter::parse, Token![,])?;
                Ok(Self::Any(filters.into_iter().collect()))
            } else if ident == "all" {
                // Parse brackets, with any number of version filters tokens in between them
                let contents;
                parenthesized!(contents in input);
                let filters: Punctuated<VersionFilter, Token![,]> = contents.parse_terminated(VersionFilter::parse, Token![,])?;
                Ok(Self::All(filters.into_iter().collect()))
            } else {
                Err(input.error(format!("Unknown operator function '{ident}' (expected `not`, `any` or `all`)")))
            }
        } else {
            Err(input.error("Expected string or operator function (e.g., `all(...)`, `not(...)`, etc)"))
        }
    }
}

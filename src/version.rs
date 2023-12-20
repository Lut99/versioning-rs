//  VERSION.rs
//    by Lut99
//
//  Created:
//    21 Nov 2023, 22:07:03
//  Last edited:
//    20 Dec 2023, 19:18:13
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines a small expression language that can be used to filter
//!   versions.
//

use proc_macro2::Span;
use proc_macro_error::{Diagnostic, Level};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, Ident, LitStr, Token};


/***** INTERFACE *****/
/// Defines that something can filter out [`Version`]s.
pub trait Filter {
    /// Examines if this filter would allow the given version.
    ///
    /// # Arguments
    /// - `list`: A [`VersionList`] that, when ordered filters are used (e.g., [`VersionFilter::AtMost`]), determines the version order.
    /// - `version`: The [`Version`] to match with this filter.
    ///
    /// # Returns
    /// True if the version _does_ match the filter, or false if it _doesn't_.
    fn matches(&self, list: &VersionList, version: &Version) -> bool;
}





/***** LIBRARY *****/
/// A version string that matches (part of) a filter.
///
/// Implemented as a string that matches the prefix of the version string in question, e.g.,
/// ```
/// "1.0" 
/// ```
/// matches all versions starting with `1.0`.
#[derive(Clone, Debug)]
pub struct Version(pub Ident);
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
    Version(LitStr),

    /// It's a `>` (ordered by appearance in `#[versioning]`)
    AtLeastExcl(LitStr),
    /// It's a `>=` (ordered by appearance in `#[versioning]`)
    AtLeast(LitStr),
    /// It's a `<` (ordered by appearance in `#[versioning]`)
    AtMostExcl(LitStr),
    /// It's a `<=` (ordered by appearance in `#[versioning]`)
    AtMost(LitStr),

    /// It's a negation of a filter (i.e., anything _but_...)
    Not(Box<Self>),
    /// It's a disjunction between nested filters
    Any(Vec<Self>),
    /// It's a conjunction between nested filters
    All(Vec<Self>),
}
impl VersionFilter {
    /// Verifies if all versions are known, then emits errors if they aren't.
    ///
    /// # Arguments
    /// - `list`: A [`VersionList`] that determines known versions.
    ///
    /// # Errors
    /// This function emits a [`Diagnostic`] if a [`Version`] in this filter did not exist.
    pub fn verify(&self, list: &VersionList) -> Result<(), Diagnostic> {
        // Match on the operation
        let unknown_span: Option<(String, Span)> = match self {
            Self::Version(ver) => {
                if !list.0.iter().any(|v| v.0.to_string().starts_with(&ver.value())) {
                    Some((ver.value(), ver.span()))
                } else {
                    None
                }
            },

            Self::AtLeastExcl(ver) => {
                if !list.0.iter().any(|v| v.0.to_string() == ver.value()) {
                    Some((ver.value(), ver.span()))
                } else {
                    None
                }
            },
            Self::AtLeast(ver) => {
                if !list.0.iter().any(|v| v.0.to_string() == ver.value()) {
                    Some((ver.value(), ver.span()))
                } else {
                    None
                }
            },
            Self::AtMostExcl(ver) => {
                if !list.0.iter().any(|v| v.0.to_string() == ver.value()) {
                    Some((ver.value(), ver.span()))
                } else {
                    None
                }
            },
            Self::AtMost(ver) => {
                if !list.0.iter().any(|v| v.0.to_string() == ver.value()) {
                    Some((ver.value(), ver.span()))
                } else {
                    None
                }
            },

            Self::Not(filter) => {
                filter.verify(list)?;
                None
            },
            Self::Any(vers) => {
                for filter in vers {
                    filter.verify(list)?;
                }
                None
            },
            Self::All(vers) => {
                for filter in vers {
                    filter.verify(list)?;
                }
                None
            },
        };

        // If we found any known ones, emit the diagnostic
        match unknown_span {
            Some((ver, span)) => Err(Diagnostic::spanned(
                span,
                Level::Error,
                format!("Unknown version string '{ver}' (add it to your `#[versioning(...)]` list of known versions)"),
            )),
            None => Ok(()),
        }
    }
}
impl Filter for VersionFilter {
    #[inline]
    fn matches(&self, list: &VersionList, version: &Version) -> bool {
        // Match on the operation
        match self {
            Self::Version(ver) => version.0.to_string().starts_with(&ver.value()),

            Self::AtLeastExcl(ver) => {
                // Find the index of both versions
                let ver_i: usize =
                    list.0.iter().position(|v| ver.value() == v.0.to_string()).unwrap_or_else(|| panic!("Encountered unknown version '{ver:?}'"));
                let version_i: usize = list
                    .0
                    .iter()
                    .position(|v| version.0.to_string() == v.0.to_string())
                    .unwrap_or_else(|| panic!("Encountered unknown version '{version:?}'"));

                // Compare
                version_i > ver_i
            },
            Self::AtLeast(ver) => {
                // Find the index of both versions
                let ver_i: usize =
                    list.0.iter().position(|v| ver.value() == v.0.to_string()).unwrap_or_else(|| panic!("Encountered unknown version '{ver:?}'"));
                let version_i: usize = list
                    .0
                    .iter()
                    .position(|v| version.0.to_string() == v.0.to_string())
                    .unwrap_or_else(|| panic!("Encountered unknown version '{version:?}'"));

                // Compare
                version_i >= ver_i
            },
            Self::AtMostExcl(ver) => {
                // Find the index of both versions
                let ver_i: usize =
                    list.0.iter().position(|v| ver.value() == v.0.to_string()).unwrap_or_else(|| panic!("Encountered unknown version '{ver:?}'"));
                let version_i: usize = list
                    .0
                    .iter()
                    .position(|v| version.0.to_string() == v.0.to_string())
                    .unwrap_or_else(|| panic!("Encountered unknown version '{version:?}'"));

                // Compare
                version_i < ver_i
            },
            Self::AtMost(ver) => {
                // Find the index of both versions
                let ver_i: usize =
                    list.0.iter().position(|v| ver.value() == v.0.to_string()).unwrap_or_else(|| panic!("Encountered unknown version '{ver:?}'"));
                let version_i: usize = list
                    .0
                    .iter()
                    .position(|v| version.0.to_string() == v.0.to_string())
                    .unwrap_or_else(|| panic!("Encountered unknown version '{version:?}'"));

                // Compare
                version_i <= ver_i
            },

            Self::Not(filter) => !filter.matches(list, version),
            Self::Any(vers) => {
                // Only one needs to match
                let mut res: bool = false;
                for filter in vers {
                    res = res || filter.matches(list, version);
                }
                res
            },
            Self::All(vers) => {
                // Catch empty lists
                if vers.is_empty() {
                    return false;
                }

                // Otherwise, require all to match
                let mut res: bool = true;
                for filter in vers {
                    res = res && filter.matches(list, version);
                }
                res
            },
        }
    }
}
impl Parse for VersionFilter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // We can use a lookahead here
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            Ok(Self::Version(input.parse()?))
        } else if lookahead.peek(Ident) {
            // Check _which_ identifier
            let ident: Ident = input.parse()?;
            if ident == "mne" {
                // Parse brackets, with a new version filter in between them
                let contents;
                parenthesized!(contents in input);
                let version: LitStr = contents.parse()?;
                Ok(Self::AtLeastExcl(version))
            } else if ident == "min" {
                // Parse brackets, with a new version filter in between them
                let contents;
                parenthesized!(contents in input);
                let version: LitStr = contents.parse()?;
                Ok(Self::AtLeast(version))
            } else if ident == "mxe" {
                // Parse brackets, with a new version filter in between them
                let contents;
                parenthesized!(contents in input);
                let version: LitStr = contents.parse()?;
                Ok(Self::AtMostExcl(version))
            } else if ident == "max" {
                // Parse brackets, with a new version filter in between them
                let contents;
                parenthesized!(contents in input);
                let version: LitStr = contents.parse()?;
                Ok(Self::AtMost(version))
            } else if ident == "not" {
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

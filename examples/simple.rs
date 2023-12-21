//  SIMPLE.rs
//    by Lut99
//
//  Created:
//    18 Nov 2023, 13:06:17
//  Last edited:
//    21 Dec 2023, 09:50:38
//  Auto updated?
//    Yes
//
//  Description:
//!   Showcases a simple usage of the [`versioning`] crate.
//!
//!   Tip: use `cargo expand --example simple` to see what this generates :)
//


/***** SCHEMA *****/
pub mod take1 {
    use versioning::versioning;

    #[versioning(v1_0_0, v2_0_0)]
    pub(crate) mod defs {
        #[version("v1_0_0")]
        pub struct FileDefinition {}
    }
}

pub mod take2 {
    use versioning::versioning;

    #[versioning(v1_0_0, v2_0_0)]
    #[version(any("v1_0_0", "v2_0_0"))]
    pub enum FileDefinition2 {}
}

pub mod take3 {
    use versioning::versioning;

    #[versioning(v1_0_0, v2_0_0)]
    #[version(all("v1_0_0", not("v2_0_0")))]
    pub struct FileDefinition3 {}
}

pub mod take4 {
    use versioning::versioning;

    #[versioning(v1_0_0, v2_0_0, v3_0_0)]
    pub(super) mod _defs {
        #[version(any("v1_0_0", "v2_0_0"))]
        mod private {
            #[version("v1_0_0")]
            pub struct Nested {}
        }

        #[version("v1_0_0")]
        pub struct FileDefinition4a {
            #[allow(dead_code)]
            nested: private::Nested,
        }
        #[version("v1_0_0")]
        impl FileDefinition4a {
            pub fn new() -> Self { Self { nested: private::Nested {} } }
        }

        #[version("v2_0_0")]
        pub struct FileDefinition4b {}
    }
}





/***** ENTRYPOINT *****/
fn main() {
    // We can now use the generated modules!
    let _a = take1::v1_0_0::defs::FileDefinition {};

    // Also works with impls, if annotated correctly :)
    let _b = take4::v1_0_0::FileDefinition4a::new();
    let _c = take4::v2_0_0::FileDefinition4b {};
}

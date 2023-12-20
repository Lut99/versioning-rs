//  SIMPLE.rs
//    by Lut99
//
//  Created:
//    18 Nov 2023, 13:06:17
//  Last edited:
//    20 Dec 2023, 15:35:27
//  Auto updated?
//    Yes
//
//  Description:
//!   Showcases a simple usage of the [`versioning`] crate.
//


/***** SCHEMA *****/
pub mod take1 {
    use versioning::versioning;

    #[versioning("v1_0_0", "v2_0_0")]
    mod defs {
        #[version("v1_0_0")]
        pub struct FileDefinition {}
    }
}

pub mod take2 {
    use versioning::versioning;

    #[versioning("v1_0_0", "v2_0_0")]
    #[version(any("v1_0_0", "v2_0_0"))]
    pub enum FileDefinition2 {}
}

pub mod take3 {
    use versioning::versioning;

    #[versioning("v1_0_0", "v2_0_0")]
    #[version(all("v1_0_0", not("v2_0_0")))]
    pub struct FileDefinition3 {}
}

pub mod take4 {
    use versioning::versioning;

    #[versioning("v1_0_0", "v2_0_0", "v3_0_0")]
    mod defs {
        #[version(any("v1_0_0", "v2_0_0"))]
        mod private {
            #[version("v1_0_0")]
            pub struct Nested {}
        }

        #[version("v1_0_0")]
        pub struct FileDefinition4a {}

        #[version("v2_0_0")]
        pub struct FileDefinition4b {}
    }
}





/***** ENTRYPOINT *****/
fn main() {}

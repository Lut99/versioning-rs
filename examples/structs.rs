//  STRUCTS.rs
//    by Lut99
//
//  Created:
//    20 Dec 2023, 15:54:25
//  Last edited:
//    20 Dec 2023, 15:59:19
//  Auto updated?
//    Yes
//
//  Description:
//!   Showcases generating multiple versions of a struct.
//!
//!   Tip: use `cargo expand --example structs` to see what this generates :)
//


/***** LIBRARY *****/
mod defs1 {
    use versioning::versioning;

    /// Shows named structs
    #[versioning("v1_0_0", "v2_0_0")]
    struct Example1 {
        #[version("v1_0_0")]
        foo: String,
        #[version("v2_0_0")]
        bar: u64,
    }
}

mod defs2 {
    use versioning::versioning;

    /// Shows unnamed structs
    #[versioning("v1_0_0", "v2_0_0")]
    struct Example2(#[version("v1_0_0")] String, #[version("v2_0_0")] u64);
}





/***** ENTRYPOINT *****/
fn main() {}

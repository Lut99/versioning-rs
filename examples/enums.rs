//  ENUMS.rs
//    by Lut99
//
//  Created:
//    20 Dec 2023, 15:46:07
//  Last edited:
//    20 Dec 2023, 19:14:48
//  Auto updated?
//    Yes
//
//  Description:
//!   Showcases generating multiple versions of an enum.
//!
//!   Tip: use `cargo expand --example enums` to see what this generates :)
//

use versioning::versioning;


/***** LIBRARY *****/
/// We create four variants of this enum
#[versioning(v1_0_0, v2_0_0, v3_0_0, v4_0_0, v5_0_0)]
enum Example {
    #[version("v1_0_0")]
    Variant1,

    #[version(any("v2_0_0", "v3_0_0"))]
    Variant2(#[version("v2_0_0")] String, #[version("v3_0_0")] u64),

    #[version(any("v4_0_0", "v5_0_0"))]
    Variant3 {
        #[version("v4_0_0")]
        foo: String,
        #[version("v5_0_0")]
        bar: u64,
    },
}





/***** ENTRYPOINT *****/
fn main() {}

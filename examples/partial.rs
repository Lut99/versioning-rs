//  PARTIAL.rs
//    by Lut99
//
//  Created:
//    20 Dec 2023, 16:45:55
//  Last edited:
//    21 Dec 2023, 10:47:29
//  Auto updated?
//    Yes
//
//  Description:
//!   Shows partial version string matching.
//

use versioning::versioning;


/***** LIBRARY *****/
/// The order of versions is determined by this order
#[versioning(v1_0_0, v1_0_1, v1_1_0, v2_0_0)]
mod defs {
    pub struct Example {
        /// Note that the string is actually matched as a prefix to the version
        #[version("v1")]
        pub foo: String,

        // So this is another way of matching *everything*
        #[version("")]
        pub bar: u64,
    }
}





/***** ENTRYPOINT *****/
fn main() {
    // This is how it works now
    let _a = v1_0_0::Example { foo: "Hello, world!".into(), bar: 42 };
    let _b = v1_0_1::Example { foo: "Hello, world!".into(), bar: 42 };
    let _c = v1_1_0::Example { foo: "Hello, world!".into(), bar: 42 };
    let _d = v2_0_0::Example { bar: 42 };
}

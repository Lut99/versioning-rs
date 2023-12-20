//  ORDERED.rs
//    by Lut99
//
//  Created:
//    20 Dec 2023, 16:23:38
//  Last edited:
//    20 Dec 2023, 16:43:17
//  Auto updated?
//    Yes
//
//  Description:
//!   Showcases the usage of ordered version filters.
//

use versioning::versioning;


/***** LIBRARY *****/
/// The order of versions is determined by this order
#[versioning("v1_0_0", "v1_0_1", "v1_1_0", "v2_0_0")]
mod defs {
    pub struct Example {
        /// This field is for the lower half...
        #[version(max("v1_0_1"))]
        pub foo: String,
        /// ...and this field is for the upper half!
        #[version(min("v1_1_0"))]
        pub bar: u64,

        /// The same but showing exclusive bounds
        #[version(max_excl("v1_1_0"))]
        pub baz: String,
        #[version(min_excl("v1_0_1"))]
        pub quz: u64,
    }
}





/***** ENTRYPOINT *****/
fn main() {
    // This is how it works now
    let _a = v1_0_0::defs::Example { foo: "Hello, world!".into(), baz: "Goodbye, world!".into() };
    let _b = v1_0_1::defs::Example { foo: "Hello, world!".into(), baz: "Goodbye, world!".into() };
    let _c = v1_1_0::defs::Example { bar: 42, quz: 84 };
    let _d = v2_0_0::defs::Example { bar: 42, quz: 84 };
}

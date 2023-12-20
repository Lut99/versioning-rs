//  IMPLS.rs
//    by Lut99
//
//  Created:
//    20 Dec 2023, 16:10:34
//  Last edited:
//    20 Dec 2023, 16:14:07
//  Auto updated?
//    Yes
//
//  Description:
//!   Some more elaborate `impl`-block showcase.
//

use versioning::versioning;


/***** LIBRARY *****/
#[versioning("v1_0_0", "v2_0_0")]
mod defs {
    pub struct Example1 {
        #[version("v1_0_0")]
        foo: String,
        #[version("v2_0_0")]
        bar: u64,
    }

    #[version("v1_0_0")]
    impl Example1 {
        pub fn new() -> Self { Self { foo: "Foo!".into() } }
    }
    #[version("v2_0_0")]
    impl Example1 {
        pub fn new() -> Self { Self { bar: 42 } }
    }

    impl Example1 {
        #[version("v1_0_0")]
        pub fn foo(&self) -> &str { &self.foo }

        #[version("v2_0_0")]
        pub fn bar(&self) -> u64 { self.bar }
    }
}





/***** ENTRYPOINT *****/
fn main() {
    // We have to commit to a version at the [`versioning`]-tag level
    {
        use v1_0_0::defs::Example1;

        // This is version 1 space!
        let example: Example1 = Example1::new();
        println!("{}", example.foo());
    }
    {
        use v2_0_0::defs::Example1;

        // This is version 2 space!
        let example: Example1 = Example1::new();
        println!("{}", example.bar());
    }
}

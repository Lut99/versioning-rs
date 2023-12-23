//  GENERICS.rs
//    by Lut99
//
//  Created:
//    23 Dec 2023, 15:56:48
//  Last edited:
//    23 Dec 2023, 16:04:53
//  Auto updated?
//    Yes
//
//  Description:
//!   It also works with generics
//

use versioning::versioning;


#[versioning(v1_0_0, v2_0_0)]
mod defs {
    /// A generic list thing
    pub struct List<I> {
        #[version("v1_0_0")]
        pub data:     Vec<I>,
        #[version("v2_0_0")]
        pub contents: Vec<I>,
    }
    impl<I> List<I>
    where
        I: Clone,
    {
        #[version("v1_0_0")]
        pub fn clone(&self) -> Self { Self { data: self.data.clone() } }

        #[version("v2_0_0")]
        pub fn clone(&self) -> Self { Self { contents: self.contents.clone() } }
    }
}


/***** ENTRYPOINT *****/
fn main() {
    // We can use the version 1...
    let data: v1_0_0::List<u64> = v1_0_0::List { data: vec![1, 2, 3] };
    let _data2: v1_0_0::List<u64> = data.clone();

    // Or version 2!
    let contents: v2_0_0::List<u64> = v2_0_0::List { contents: vec![1, 2, 3] };
    let _contents2: v2_0_0::List<u64> = contents.clone();
}

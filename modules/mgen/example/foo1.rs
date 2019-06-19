//#![no_std]

//extern crate std;
//extern crate core;

pub mod foo6 {
    pub struct Foo2 {
        pub foo3: i8,
    }
}

use foo6::Foo2;
pub enum Foo4 {
    Foo5(Foo2),
}

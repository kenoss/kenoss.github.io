pub mod a {
    pub struct Rect {
        width: usize,
        height: usize,
    }

    #[enum_dispatch::enum_dispatch]
    pub enum Shape {
        Rect(Rect),
    }

    #[enum_dispatch::enum_dispatch(Shape)]
    pub trait ShapeI {
        fn area(&self) -> usize;
    }

    impl ShapeI for Rect {
        fn area(&self) -> usize {
            self.width * self.height
        }
    }
}

mod b {
    use crate::a::{ShapeI, Rect};

    // This should define the enum and generate `impl ShapeI for Shape` once, but it actually does twice.
    #[enum_dispatch::enum_dispatch(ShapeI)]
    enum Shape {
        Rect(Rect),
    }
}

fn main() {}

// The result of `cargo expand`:
//
// #![feature(prelude_import)]
// #[prelude_import]
// use std::prelude::rust_2021::*;
// #[macro_use]
// extern crate std;
// pub mod a {
//     pub struct Rect {
//         width: usize,
//         height: usize,
//     }
//     pub enum Shape {
//         Rect(Rect),
//     }
//     pub trait ShapeI {
//         fn area(&self) -> usize;
//     }
//     impl ::core::convert::From<Rect> for Shape {
//         fn from(v: Rect) -> Shape {
//             Shape::Rect(v)
//         }
//     }
//     impl ::core::convert::TryInto<Rect> for Shape {
//         type Error = &'static str;
//         fn try_into(
//             self,
//         ) -> ::core::result::Result<
//             Rect,
//             <Self as ::core::convert::TryInto<Rect>>::Error,
//         > {
//             match self {
//                 Shape::Rect(v) => Ok(v),
//             }
//         }
//     }
//     impl ShapeI for Shape {
//         #[inline]
//         fn area(&self) -> usize {
//             match self {
//                 Shape::Rect(inner) => ShapeI::area(inner),
//             }
//         }
//     }
//     impl ShapeI for Rect {
//         fn area(&self) -> usize {
//             self.width * self.height
//         }
//     }
// }
// mod b {
//     use crate::a::{ShapeI, Rect};
//     enum Shape {
//         Rect(Rect),
//     }
//     impl ShapeI for Shape {
//         #[inline]
//         fn area(&self) -> usize {
//             match self {
//                 Shape::Rect(inner) => ShapeI::area(inner),
//             }
//         }
//     }
//     impl ShapeI for Shape {
//         #[inline]
//         fn area(&self) -> usize {
//             match self {
//                 Shape::Rect(inner) => ShapeI::area(inner),
//             }
//         }
//     }
// }
// fn main() {}

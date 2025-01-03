struct Rect<T> {
    width: T,
    height: T,
}

#[enum_dispatch::enum_dispatch]
enum Shape<T> {
    Rect(Rect<T>),
}

#[enum_dispatch::enum_dispatch(Shape<f64>)]
trait ShapeI {
    fn area(&self) -> f64;
}

impl<T> ShapeI for Rect<T>
where
    T: std::ops::Mul,
{
    fn area(&self) -> f64 {
        (self.width * self.height) as f64
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
// struct Rect<T> {
//     width: T,
//     height: T,
// }
// enum Shape<T> {
//     Rect(Rect<T>),
// }
// trait ShapeI {
//     fn area(&self) -> f64;
// }
// impl<T> ::core::convert::From<Rect<T>> for Shape<T> {
//     fn from(v: Rect<T>) -> Shape<T> {
//         Shape::Rect(v)
//     }
// }
// impl<T> ::core::convert::TryInto<Rect<T>> for Shape<T> {
//     type Error = &'static str;
//     fn try_into(
//         self,
//     ) -> ::core::result::Result<
//         Rect<T>,
//         <Self as ::core::convert::TryInto<Rect<T>>>::Error,
//     > {
//         match self {
//             Shape::Rect(v) => Ok(v),
//         }
//     }
// }
// impl<T> ShapeI for Shape<T> {
//     #[inline]
//     fn area(&self) -> f64 {
//         match self {
//             Shape::Rect(inner) => ShapeI::area(inner),
//         }
//     }
// }
// impl<T> ShapeI for Rect<T>
// where
//     T: std::ops::Mul,
// {
//     fn area(&self) -> f64 {
//         (self.width * self.height) as f64
//     }
// }
// fn main() {}

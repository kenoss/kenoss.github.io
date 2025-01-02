struct Rect<T> {
    width: T,
    height: T,
}

#[enum_dispatch::enum_dispatch]
enum Shape<T> {
    Rect(Rect<T>),
}

#[enum_dispatch::enum_dispatch(Shape<usize>)]
trait ShapeI {
    fn area(&self) -> usize;
}

impl<T> ShapeI for Rect<T>
where
    T: std::ops::Mul,
{
    fn area(&self) -> usize {
        (self.width * self.height) as usize
    }
}

fn main() {
    let rect = Rect { width: 2, height: 3 };
    assert_eq!(rect.area(), 6);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6);
}

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
//     fn area(&self) -> usize;
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
//     fn area(&self) -> usize {
//         match self {
//             Shape::Rect(inner) => ShapeI::area(inner),
//         }
//     }
// }
// impl<T> ShapeI for Rect<T>
// where
//     T: std::ops::Mul,
// {
//     fn area(&self) -> usize {
//         (self.width * self.height) as usize
//     }
// }
// fn main() {
//     let rect = Rect { width: 2, height: 3 };
//     match (&rect.area(), &6) {
//         (left_val, right_val) => {
//             if !(*left_val == *right_val) {
//                 let kind = ::core::panicking::AssertKind::Eq;
//                 ::core::panicking::assert_failed(
//                     kind,
//                     &*left_val,
//                     &*right_val,
//                     ::core::option::Option::None,
//                 );
//             }
//         }
//     };
//     let shape = Shape::Rect(rect);
//     match (&shape.area(), &6) {
//         (left_val, right_val) => {
//             if !(*left_val == *right_val) {
//                 let kind = ::core::panicking::AssertKind::Eq;
//                 ::core::panicking::assert_failed(
//                     kind,
//                     &*left_val,
//                     &*right_val,
//                     ::core::option::Option::None,
//                 );
//             }
//         }
//     };
// }

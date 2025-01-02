#[auto_delegate::delegate]
trait ShapeI {
    fn area(&self) -> usize;
}

struct Rect {
    width: usize,
    height: usize,
}

#[derive(auto_delegate::Delegate)]
#[to(ShapeI)]
enum Shape {
    Rect(Rect),
}

impl ShapeI for Rect {
    fn area(&self) -> usize {
        self.width * self.height
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
// trait ShapeI {
//     fn area(&self) -> usize;
// }
// impl<DelegateImpl> ShapeI for DelegateImpl
// where
//     DelegateImpl: auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::A: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::B: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::C: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::D: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::D: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::E: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::F: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::G: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::H: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::I: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::J: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::K: ShapeI,
//     <DelegateImpl as auto_delegate::Delegatable<
//         's',
//         'h',
//         'a',
//         'p',
//         'e',
//         'i',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//         ' ',
//     >>::L: ShapeI,
// {
//     #[inline(always)]
//     fn area(&self) -> usize {
//         let m = self.delegate_by_ref();
//         if let Some(t) = m.0 {
//             return t.area();
//         }
//         if let Some(t) = m.1 {
//             return t.area();
//         }
//         if let Some(t) = m.2 {
//             return t.area();
//         }
//         if let Some(t) = m.3 {
//             return t.area();
//         }
//         if let Some(t) = m.4 {
//             return t.area();
//         }
//         if let Some(t) = m.5 {
//             return t.area();
//         }
//         if let Some(t) = m.6 {
//             return t.area();
//         }
//         if let Some(t) = m.7 {
//             return t.area();
//         }
//         if let Some(t) = m.8 {
//             return t.area();
//         }
//         if let Some(t) = m.9 {
//             return t.area();
//         }
//         if let Some(t) = m.10 {
//             return t.area();
//         }
//         if let Some(t) = m.11 {
//             return t.area();
//         }
//         {
//             ::core::panicking::panic_fmt(format_args!("unreachable"));
//         };
//     }
// }
// struct Rect {
//     width: usize,
//     height: usize,
// }
// #[to(ShapeI)]
// enum Shape {
//     Rect(Rect),
// }
// impl auto_delegate::Delegatable<
//     's',
//     'h',
//     'a',
//     'p',
//     'e',
//     'i',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
//     ' ',
// > for Shape {
//     type A = Rect;
//     type B = Rect;
//     type C = Rect;
//     type D = Rect;
//     type E = Rect;
//     type F = Rect;
//     type G = Rect;
//     type H = Rect;
//     type I = Rect;
//     type J = Rect;
//     type K = Rect;
//     type L = Rect;
//     #[inline(always)]
//     fn delegate_by_owned(
//         self,
//     ) -> auto_delegate::Delegates<
//         Self::A,
//         Self::B,
//         Self::C,
//         Self::D,
//         Self::E,
//         Self::F,
//         Self::G,
//         Self::H,
//         Self::I,
//         Self::J,
//         Self::K,
//         Self::L,
//     > {
//         match self {
//             Self::Rect(v) => {
//                 auto_delegate::Delegates(
//                     Some(v),
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                 )
//             }
//         }
//     }
//     #[inline(always)]
//     fn delegate_by_ref(
//         &self,
//     ) -> auto_delegate::Delegates<
//         &Self::A,
//         &Self::B,
//         &Self::C,
//         &Self::D,
//         &Self::E,
//         &Self::F,
//         &Self::G,
//         &Self::H,
//         &Self::I,
//         &Self::J,
//         &Self::K,
//         &Self::L,
//     > {
//         match self {
//             Self::Rect(v) => {
//                 auto_delegate::Delegates(
//                     Some(v),
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                 )
//             }
//         }
//     }
//     #[inline(always)]
//     fn delegate_by_mut(
//         &mut self,
//     ) -> auto_delegate::Delegates<
//         &mut Self::A,
//         &mut Self::B,
//         &mut Self::C,
//         &mut Self::D,
//         &mut Self::E,
//         &mut Self::F,
//         &mut Self::G,
//         &mut Self::H,
//         &mut Self::I,
//         &mut Self::J,
//         &mut Self::K,
//         &mut Self::L,
//     > {
//         match self {
//             Self::Rect(v) => {
//                 auto_delegate::Delegates(
//                     Some(v),
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                     None,
//                 )
//             }
//         }
//     }
// }
// impl ShapeI for Rect {
//     fn area(&self) -> usize {
//         self.width * self.height
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

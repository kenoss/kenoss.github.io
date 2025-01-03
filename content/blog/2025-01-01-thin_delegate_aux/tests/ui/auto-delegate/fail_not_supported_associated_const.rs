#[auto_delegate::delegate]
trait ShapeI {
    type Output;
    const NAME: &'static str;

    fn area(&self) -> Self::Output;
}

#[derive(auto_delegate::Delegate)]
#[to(ShapeI)]
enum Shape {
    Rect(Rect),
    Circle(Circle),
}

struct Rect {
    width: f64,
    height: f64,
}

impl ShapeI for Rect {
    type Output = f64;
    const NAME: &'static str = "Rect";

    fn area(&self) -> Self::Output {
        self.width * self.height
    }
}

struct Circle {
    radius: f64,
}

impl ShapeI for Circle {
    type Output = f64;
    const NAME: &'static str = "Circle";

    fn area(&self) -> Self::Output {
        3.14 * self.radius * self.radius
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
// trait ShapeI {
//     type Output;
//     const NAME: &'static str;
//     fn area(&self) -> Self::Output;
// }
// impl<DelegateImpl, Output> ShapeI for DelegateImpl
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
//     >>::A: ShapeI<Output = Output>,
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
//     >>::B: ShapeI<Output = Output>,
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
//     >>::C: ShapeI<Output = Output>,
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
//     >>::D: ShapeI<Output = Output>,
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
//     >>::D: ShapeI<Output = Output>,
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
//     >>::E: ShapeI<Output = Output>,
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
//     >>::F: ShapeI<Output = Output>,
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
//     >>::G: ShapeI<Output = Output>,
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
//     >>::H: ShapeI<Output = Output>,
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
//     >>::I: ShapeI<Output = Output>,
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
//     >>::J: ShapeI<Output = Output>,
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
//     >>::K: ShapeI<Output = Output>,
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
//     >>::L: ShapeI<Output = Output>,
// {
//     type Output = <<Self as auto_delegate::Delegatable<
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
//     >>::A as ShapeI>::Output;
//     #[inline(always)]
//     fn area(&self) -> Self::Output {
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
// #[to(ShapeI)]
// enum Shape {
//     Rect(Rect),
//     Circle(Circle),
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
//     type B = Circle;
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
//             Self::Circle(v) => {
//                 auto_delegate::Delegates(
//                     None,
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
//             Self::Circle(v) => {
//                 auto_delegate::Delegates(
//                     None,
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
//             Self::Circle(v) => {
//                 auto_delegate::Delegates(
//                     None,
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
//                 )
//             }
//         }
//     }
// }
// struct Rect {
//     width: f64,
//     height: f64,
// }
// impl ShapeI for Rect {
//     type Output = f64;
//     const NAME: &'static str = "Rect";
//     fn area(&self) -> Self::Output {
//         self.width * self.height
//     }
// }
// struct Circle {
//     radius: f64,
// }
// impl ShapeI for Circle {
//     type Output = f64;
//     const NAME: &'static str = "Circle";
//     fn area(&self) -> Self::Output {
//         3.14 * self.radius * self.radius
//     }
// }
// fn main() {}

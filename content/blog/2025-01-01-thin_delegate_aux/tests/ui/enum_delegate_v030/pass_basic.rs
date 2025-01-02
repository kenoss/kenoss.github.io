use enum_delegate_v030 as enum_delegate;

#[enum_delegate::delegate]
trait ShapeI {
    fn area(&self) -> f64;
}

#[enum_delegate::delegate(derive(ShapeI))]
enum Shape {
    Rect(Rect),
    Circle(Circle),
}

struct Rect {
    width: f64,
    height: f64,
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

struct Circle {
    radius: f64,
}

impl ShapeI for Circle {
    fn area(&self) -> f64 {
        3.14 * self.radius * self.radius
    }
}

fn main() {
    let rect = Rect { width: 2.0, height: 3.0 };
    assert_eq!(rect.area(), 6.0);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6.0);
    let circle = Circle { radius: 2.0 };
    assert_eq!(circle.area(), 12.56);
    let shape = Shape::Circle(circle);
    assert_eq!(shape.area(), 12.56);
}

// #![feature(prelude_import)]
// #[prelude_import]
// use std::prelude::rust_2021::*;
// #[macro_use]
// extern crate std;
// use enum_delegate_v030 as enum_delegate;
// trait ShapeI: __delegate_ShapeI__Scope {
//     fn area(&self) -> f64;
// }
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// #[doc(hidden)]
// trait __delegate_ShapeI__Scope {
//     #[allow(non_camel_case_types)]
//     type __delegate_ShapeI__Bind0<'__delegate>;
// }
// #[automatically_derived]
// impl<__Delegate: ?::core::marker::Sized> __delegate_ShapeI__Scope for __Delegate {
//     type __delegate_ShapeI__Bind0<'__delegate> = __delegate_ShapeI__Bind0<
//         '__delegate,
//         __Delegate,
//     >;
// }
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// #[doc(hidden)]
// struct __delegate_ShapeI__Bind0<'__delegate, __Delegate: ?::core::marker::Sized>(
//     ::core::marker::PhantomData<(&'__delegate (), *const __Delegate)>,
// );
// #[automatically_derived]
// impl<
//     '__delegate,
//     __Delegate: ?::core::marker::Sized,
// > ::enum_delegate_v030::__macros::TypeOf
// for __delegate_ShapeI__Bind0<'__delegate, __Delegate> {
//     type T = f64;
// }
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// trait __delegate_ShapeI__DelegateOwned {}
// #[automatically_derived]
// impl<__Left, __Right> __delegate_ShapeI__DelegateOwned
// for ::enum_delegate_v030::__macros::Either<__Left, __Right>
// where
//     __Left: ShapeI,
//     __Right: __delegate_ShapeI__DelegateOwned,
// {}
// #[automatically_derived]
// impl __delegate_ShapeI__DelegateOwned for ::enum_delegate_v030::__macros::Void {}
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// trait __delegate_ShapeI__DelegateRef<'__delegate>
// where
//     Self: Sized + '__delegate,
// {
//     fn area(self) -> f64;
// }
// #[automatically_derived]
// impl<'__delegate, __Left, __Right> __delegate_ShapeI__DelegateRef<'__delegate>
// for ::enum_delegate_v030::__macros::Either<&'__delegate __Left, __Right>
// where
//     Self: Sized + '__delegate,
//     __Left: ShapeI,
//     __Right: __delegate_ShapeI__DelegateRef<'__delegate>,
// {
//     fn area(self) -> f64 {
//         match self {
//             Self::Left(__delegate) => <__Left as ShapeI>::area(__delegate),
//             Self::Right(__delegate) => {
//                 <__Right as __delegate_ShapeI__DelegateRef<
//                     '__delegate,
//                 >>::area(__delegate)
//             }
//         }
//     }
// }
// #[automatically_derived]
// impl<'__delegate> __delegate_ShapeI__DelegateRef<'__delegate>
// for ::enum_delegate_v030::__macros::Void
// where
//     Self: Sized + '__delegate,
// {
//     fn area(self) -> f64 {
//         match self {}
//     }
// }
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// trait __delegate_ShapeI__DelegateRefMut<'__delegate>
// where
//     Self: Sized + '__delegate,
// {}
// #[automatically_derived]
// impl<'__delegate, __Left, __Right> __delegate_ShapeI__DelegateRefMut<'__delegate>
// for ::enum_delegate_v030::__macros::Either<&'__delegate mut __Left, __Right>
// where
//     Self: Sized + '__delegate,
//     __Left: ShapeI,
//     __Right: __delegate_ShapeI__DelegateRefMut<'__delegate>,
// {}
// #[automatically_derived]
// impl<'__delegate> __delegate_ShapeI__DelegateRefMut<'__delegate>
// for ::enum_delegate_v030::__macros::Void
// where
//     Self: Sized + '__delegate,
// {}
// #[automatically_derived]
// impl<__Delegate> ShapeI for ::enum_delegate_v030::__macros::Wrapper<__Delegate>
// where
//     __Delegate: ::enum_delegate_v030::__macros::Convert,
//     <__Delegate as ::enum_delegate_v030::__macros::Convert>::Owned: __delegate_ShapeI__DelegateOwned,
//     for<'__delegate> <__Delegate as ::enum_delegate_v030::__macros::Convert>::Ref<
//         '__delegate,
//     >: __delegate_ShapeI__DelegateRef<'__delegate>,
//     for<'__delegate> <__Delegate as ::enum_delegate_v030::__macros::Convert>::RefMut<
//         '__delegate,
//     >: __delegate_ShapeI__DelegateRefMut<'__delegate>,
// {
//     fn area(&self) -> f64 {
//         <<__Delegate as ::enum_delegate_v030::__macros::Convert>::Ref<
//             '_,
//         > as __delegate_ShapeI__DelegateRef<
//             '_,
//         >>::area(
//             <__Delegate as ::enum_delegate_v030::__macros::Convert>::convert_ref(&self.0),
//         )
//     }
// }
// #[allow(non_snake_case, unused_imports)]
// #[automatically_derived]
// #[doc(hidden)]
// use __delegate_ShapeI1316740643954748179600 as ShapeI;
// #[automatically_derived]
// const _: fn() = || {
//     struct OnlyMarkerSelfBoundsSupportedForNow;
//     fn assert_impl_all<T: Sized>() {}
//     assert_impl_all::<OnlyMarkerSelfBoundsSupportedForNow>();
// };
// enum Shape {
//     Rect(Rect),
//     Circle(Circle),
// }
// #[automatically_derived]
// impl ::enum_delegate_v030::__macros::Convert for Shape {
//     type Owned = ::enum_delegate_v030::__macros::Either<
//         Rect,
//         ::enum_delegate_v030::__macros::Either<
//             Circle,
//             ::enum_delegate_v030::__macros::Void,
//         >,
//     >;
//     type Ref<'__delegate> = ::enum_delegate_v030::__macros::Either<
//         &'__delegate Rect,
//         ::enum_delegate_v030::__macros::Either<
//             &'__delegate Circle,
//             ::enum_delegate_v030::__macros::Void,
//         >,
//     >
//     where
//         Rect: '__delegate,
//         Circle: '__delegate;
//     type RefMut<'__delegate> = ::enum_delegate_v030::__macros::Either<
//         &'__delegate mut Rect,
//         ::enum_delegate_v030::__macros::Either<
//             &'__delegate mut Circle,
//             ::enum_delegate_v030::__macros::Void,
//         >,
//     >
//     where
//         Rect: '__delegate,
//         Circle: '__delegate;
//     fn convert_owned(self) -> <Self as ::enum_delegate_v030::__macros::Convert>::Owned {
//         match self {
//             Self::Rect(v) => ::enum_delegate_v030::__macros::Either::Left(v),
//             Self::Circle(v) => {
//                 ::enum_delegate_v030::__macros::Either::Right(
//                     ::enum_delegate_v030::__macros::Either::Left(v),
//                 )
//             }
//         }
//     }
//     fn convert_ref(&self) -> <Self as ::enum_delegate_v030::__macros::Convert>::Ref<'_> {
//         match self {
//             Self::Rect(v) => ::enum_delegate_v030::__macros::Either::Left(v),
//             Self::Circle(v) => {
//                 ::enum_delegate_v030::__macros::Either::Right(
//                     ::enum_delegate_v030::__macros::Either::Left(v),
//                 )
//             }
//         }
//     }
//     fn convert_ref_mut(
//         &mut self,
//     ) -> <Self as ::enum_delegate_v030::__macros::Convert>::RefMut<'_> {
//         match self {
//             Self::Rect(v) => ::enum_delegate_v030::__macros::Either::Left(v),
//             Self::Circle(v) => {
//                 ::enum_delegate_v030::__macros::Either::Right(
//                     ::enum_delegate_v030::__macros::Either::Left(v),
//                 )
//             }
//         }
//     }
// }
// #[automatically_derived]
// impl ShapeI for Shape {
//     fn area(
//         &self,
//     ) -> <Self::__delegate_ShapeI__Bind0<
//         '_,
//     > as ::enum_delegate_v030::__macros::TypeOf>::T {
//         <::enum_delegate_v030::__macros::Wrapper<
//             Shape,
//         > as ShapeI>::area(
//             #[allow(clippy::transmute_ptr_to_ptr, unsafe_code)]
//             unsafe { ::core::mem::transmute(self) },
//         )
//     }
// }
// struct Rect {
//     width: f64,
//     height: f64,
// }
// impl ShapeI for Rect {
//     fn area(&self) -> f64 {
//         self.width * self.height
//     }
// }
// struct Circle {
//     radius: f64,
// }
// impl ShapeI for Circle {
//     fn area(&self) -> f64 {
//         3.14 * self.radius * self.radius
//     }
// }
// fn main() {
//     let rect = Rect { width: 2.0, height: 3.0 };
//     match (&rect.area(), &6.0) {
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
//     match (&shape.area(), &6.0) {
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
//     let circle = Circle { radius: 2.0 };
//     match (&circle.area(), &12.56) {
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
//     let shape = Shape::Circle(circle);
//     match (&shape.area(), &12.56) {
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

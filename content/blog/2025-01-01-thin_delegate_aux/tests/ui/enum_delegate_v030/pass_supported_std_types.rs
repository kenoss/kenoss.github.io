use enum_delegate_v030 as enum_delegate;

#[enum_delegate::delegate]
pub trait Hello {
    fn hello(&self) -> String;
}

impl Hello for String {
    fn hello(&self) -> String {
        format!("hello, {self}")
    }
}

#[enum_delegate::delegate(derive(Hello))]
struct Hoge(String);

fn main() {
    let hoge = Hoge("hoge".to_string());
    assert_eq!(hoge.hello(), "hello, hoge");
}

// #![feature(prelude_import)]
// #[prelude_import]
// use std::prelude::rust_2021::*;
// #[macro_use]
// extern crate std;
// use enum_delegate_v030 as enum_delegate;
// pub trait Hello: __delegate_Hello__Scope {
//     fn hello(&self) -> String;
// }
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// #[doc(hidden)]
// pub trait __delegate_Hello__Scope {
//     #[allow(non_camel_case_types)]
//     type __delegate_Hello__Bind0<'__delegate>;
// }
// #[automatically_derived]
// impl<__Delegate: ?::core::marker::Sized> __delegate_Hello__Scope for __Delegate {
//     type __delegate_Hello__Bind0<'__delegate> = __delegate_Hello__Bind0<
//         '__delegate,
//         __Delegate,
//     >;
// }
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// #[doc(hidden)]
// pub struct __delegate_Hello__Bind0<'__delegate, __Delegate: ?::core::marker::Sized>(
//     ::core::marker::PhantomData<(&'__delegate (), *const __Delegate)>,
// );
// #[automatically_derived]
// impl<
//     '__delegate,
//     __Delegate: ?::core::marker::Sized,
// > ::enum_delegate_v030::__macros::TypeOf
// for __delegate_Hello__Bind0<'__delegate, __Delegate> {
//     type T = String;
// }
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// trait __delegate_Hello__DelegateOwned {}
// #[automatically_derived]
// impl<__Left, __Right> __delegate_Hello__DelegateOwned
// for ::enum_delegate_v030::__macros::Either<__Left, __Right>
// where
//     __Left: Hello,
//     __Right: __delegate_Hello__DelegateOwned,
// {}
// #[automatically_derived]
// impl __delegate_Hello__DelegateOwned for ::enum_delegate_v030::__macros::Void {}
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// trait __delegate_Hello__DelegateRef<'__delegate>
// where
//     Self: Sized + '__delegate,
// {
//     fn hello(self) -> String;
// }
// #[automatically_derived]
// impl<'__delegate, __Left, __Right> __delegate_Hello__DelegateRef<'__delegate>
// for ::enum_delegate_v030::__macros::Either<&'__delegate __Left, __Right>
// where
//     Self: Sized + '__delegate,
//     __Left: Hello,
//     __Right: __delegate_Hello__DelegateRef<'__delegate>,
// {
//     fn hello(self) -> String {
//         match self {
//             Self::Left(__delegate) => <__Left as Hello>::hello(__delegate),
//             Self::Right(__delegate) => {
//                 <__Right as __delegate_Hello__DelegateRef<
//                     '__delegate,
//                 >>::hello(__delegate)
//             }
//         }
//     }
// }
// #[automatically_derived]
// impl<'__delegate> __delegate_Hello__DelegateRef<'__delegate>
// for ::enum_delegate_v030::__macros::Void
// where
//     Self: Sized + '__delegate,
// {
//     fn hello(self) -> String {
//         match self {}
//     }
// }
// #[allow(non_camel_case_types)]
// #[automatically_derived]
// trait __delegate_Hello__DelegateRefMut<'__delegate>
// where
//     Self: Sized + '__delegate,
// {}
// #[automatically_derived]
// impl<'__delegate, __Left, __Right> __delegate_Hello__DelegateRefMut<'__delegate>
// for ::enum_delegate_v030::__macros::Either<&'__delegate mut __Left, __Right>
// where
//     Self: Sized + '__delegate,
//     __Left: Hello,
//     __Right: __delegate_Hello__DelegateRefMut<'__delegate>,
// {}
// #[automatically_derived]
// impl<'__delegate> __delegate_Hello__DelegateRefMut<'__delegate>
// for ::enum_delegate_v030::__macros::Void
// where
//     Self: Sized + '__delegate,
// {}
// #[automatically_derived]
// impl<__Delegate> Hello for ::enum_delegate_v030::__macros::Wrapper<__Delegate>
// where
//     __Delegate: ::enum_delegate_v030::__macros::Convert,
//     <__Delegate as ::enum_delegate_v030::__macros::Convert>::Owned: __delegate_Hello__DelegateOwned,
//     for<'__delegate> <__Delegate as ::enum_delegate_v030::__macros::Convert>::Ref<
//         '__delegate,
//     >: __delegate_Hello__DelegateRef<'__delegate>,
//     for<'__delegate> <__Delegate as ::enum_delegate_v030::__macros::Convert>::RefMut<
//         '__delegate,
//     >: __delegate_Hello__DelegateRefMut<'__delegate>,
// {
//     fn hello(&self) -> String {
//         <<__Delegate as ::enum_delegate_v030::__macros::Convert>::Ref<
//             '_,
//         > as __delegate_Hello__DelegateRef<
//             '_,
//         >>::hello(
//             <__Delegate as ::enum_delegate_v030::__macros::Convert>::convert_ref(&self.0),
//         )
//     }
// }
// #[allow(non_snake_case, unused_imports)]
// #[automatically_derived]
// #[doc(hidden)]
// pub use __delegate_Hello1234221945672457041000 as Hello;
// #[automatically_derived]
// const _: fn() = || {
//     struct OnlyMarkerSelfBoundsSupportedForNow;
//     fn assert_impl_all<T: Sized>() {}
//     assert_impl_all::<OnlyMarkerSelfBoundsSupportedForNow>();
// };
// impl Hello for String {
//     fn hello(&self) -> String {
//         {
//             let res = ::alloc::fmt::format(format_args!("hello, {0}", self));
//             res
//         }
//     }
// }
// struct Hoge(String);
// #[automatically_derived]
// impl ::enum_delegate_v030::__macros::Convert for Hoge {
//     type Owned = ::enum_delegate_v030::__macros::Either<
//         String,
//         ::enum_delegate_v030::__macros::Void,
//     >;
//     type Ref<'__delegate> = ::enum_delegate_v030::__macros::Either<
//         &'__delegate String,
//         ::enum_delegate_v030::__macros::Void,
//     >
//     where
//         String: '__delegate;
//     type RefMut<'__delegate> = ::enum_delegate_v030::__macros::Either<
//         &'__delegate mut String,
//         ::enum_delegate_v030::__macros::Void,
//     >
//     where
//         String: '__delegate;
//     fn convert_owned(self) -> <Self as ::enum_delegate_v030::__macros::Convert>::Owned {
//         ::enum_delegate_v030::__macros::Either::Left(self.0)
//     }
//     fn convert_ref(&self) -> <Self as ::enum_delegate_v030::__macros::Convert>::Ref<'_> {
//         ::enum_delegate_v030::__macros::Either::Left(&self.0)
//     }
//     fn convert_ref_mut(
//         &mut self,
//     ) -> <Self as ::enum_delegate_v030::__macros::Convert>::RefMut<'_> {
//         ::enum_delegate_v030::__macros::Either::Left(&mut self.0)
//     }
// }
// #[automatically_derived]
// impl Hello for Hoge {
//     fn hello(
//         &self,
//     ) -> <Self::__delegate_Hello__Bind0<
//         '_,
//     > as ::enum_delegate_v030::__macros::TypeOf>::T {
//         <::enum_delegate_v030::__macros::Wrapper<
//             Hoge,
//         > as Hello>::hello(
//             #[allow(clippy::transmute_ptr_to_ptr, unsafe_code)]
//             unsafe { ::core::mem::transmute(self) },
//         )
//     }
// }
// fn main() {
//     let hoge = Hoge("hoge".to_string());
//     match (&hoge.hello(), &"hello, hoge") {
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

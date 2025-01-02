#[ambassador::delegatable_trait]
trait Shout<T>
where
    T: std::fmt::Display,
{
    fn shout(&self, input: T) -> String;
}

impl<T> Shout<T> for String
where
    T: std::fmt::Display,
{
    fn shout(&self, input: T) -> String {
        format!("{}, {}", self, input)
    }
}

#[derive(ambassador::Delegate)]
#[delegate(Shout<T>, generics = "T", where = "T: std::fmt::Display")]
pub struct Cat(String);

// `T` in `pub trait MatchShout<T, ambassador_X: Shout<T>>: Shout<T> {}` doesn't satisfy `T: std::fmt::Display`.
#[derive(ambassador::Delegate)]
#[delegate(Shout<T>, generics = "T", where = "T: std::fmt::Display")]
enum Animal {
    Cat(Cat),
}

fn main() {}

// The result of `cargo expand`:
//
// #![feature(prelude_import)]
// #[prelude_import]
// use std::prelude::rust_2021::*;
// #[macro_use]
// extern crate std;
// trait Shout<T>
// where
//     T: std::fmt::Display,
// {
//     fn shout(&self, input: T) -> String;
// }
// #[doc(inline)]
// ///A macro to be used by [`ambassador::Delegate`] to delegate [`Shout`]
// use _ambassador_impl_Shout as ambassador_impl_Shout;
// #[doc(hidden)]
// #[allow(non_snake_case)]
// mod ambassador_impl_Shout {}
// impl<T> Shout<T> for String
// where
//     T: std::fmt::Display,
// {
//     fn shout(&self, input: T) -> String {
//         {
//             let res = ::alloc::fmt::format(format_args!("{0}, {1}", self, input));
//             res
//         }
//     }
// }
// #[delegate(Shout<T>, generics = "T", where = "T: std::fmt::Display")]
// pub struct Cat(String);
// impl<T> Shout<T> for Cat
// where
//     T: std::fmt::Display,
//     String: Shout<T>,
// {
//     #[inline]
//     #[allow(unused_braces)]
//     fn shout(&self, input: T) -> String {
//         self.0.shout(input)
//     }
// }
// #[delegate(Shout<T>, generics = "T", where = "T: std::fmt::Display")]
// enum Animal {
//     Cat(Cat),
// }
// #[allow(non_snake_case)]
// mod ambassador_module_Shout_for_Animal {
//     use super::*;
//     #[doc(hidden)]
//     #[allow(non_camel_case_types)]
//     pub trait MatchShout<T, ambassador_X: Shout<T>>: Shout<T> {}
//     #[allow(non_camel_case_types)]
//     impl<T, ambassador_X: Shout<T>, ambassador_Y: Shout<T>> MatchShout<T, ambassador_X>
//     for ambassador_Y {}
//     impl<T> Shout<T> for Animal
//     where
//         T: std::fmt::Display,
//         Cat: Shout<T>,
//     {
//         #[inline]
//         #[allow(unused_braces)]
//         fn shout(&self, input: T) -> String {
//             match self {
//                 Animal::Cat(inner) => inner.shout(input),
//             }
//         }
//     }
// }
// fn main() {}

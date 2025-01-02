#[ambassador::delegatable_trait]
trait Shout<T>
// where
//     T: std::fmt::Display,
{
    fn shout(&self, input: T) -> String;
}

#[derive(ambassador::Delegate)]
#[delegate(Shout<T>, generics = "T", where = "T: std::fmt::Display")]
enum Animal {
    Cat(Cat),
}

#[derive(ambassador::Delegate)]
#[delegate(Shout<T>, generics = "T", where = "T: std::fmt::Display")]
pub struct Cat(String);

impl<T> Shout<T> for String
where
    T: std::fmt::Display,
{
    fn shout(&self, input: T) -> String {
        format!("{}, {}", self, input)
    }
}

fn main() {
    let cat = Cat("meow".to_string());
    assert_eq!(cat.shout("world"), "meow, world");
    let animal = Animal::Cat(cat);
    assert_eq!(animal.shout("world"), "meow, world");
}

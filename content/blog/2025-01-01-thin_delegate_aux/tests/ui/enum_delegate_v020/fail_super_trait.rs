use enum_delegate_v020 as enum_delegate;

#[enum_delegate::register]
trait ShapeI: std::fmt::Debug {
    fn area(&self) -> f64;
}

#[derive(Debug)]
struct Rect {
    width: f64,
    height: f64,
}

#[derive(Debug)]
#[enum_delegate::implement(ShapeI)]
enum Shape {
    Rect(Rect),
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn main() {}

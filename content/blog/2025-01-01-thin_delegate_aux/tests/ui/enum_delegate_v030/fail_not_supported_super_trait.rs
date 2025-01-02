use enum_delegate_v030 as enum_delegate;

#[enum_delegate::delegate]
trait ShapeI: std::fmt::Debug {
    fn area(&self) -> f64;
}

#[derive(Debug)]
#[enum_delegate::delegate(derive(ShapeI))]
enum Shape {
    Rect(Rect),
    Circle(Circle),
}

#[derive(Debug)]
struct Rect {
    width: f64,
    height: f64,
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

#[derive(Debug)]
struct Circle {
    radius: f64,
}

impl ShapeI for Circle {
    fn area(&self) -> f64 {
        3.14 * self.radius * self.radius
    }
}

fn main() {}

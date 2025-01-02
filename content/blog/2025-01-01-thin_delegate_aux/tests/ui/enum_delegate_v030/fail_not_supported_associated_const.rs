use enum_delegate_v030 as enum_delegate;

#[enum_delegate::delegate]
trait ShapeI {
    type Output;
    const NAME: &'static str;

    fn area(&self) -> Self::Output;
}

#[enum_delegate::delegate]
enum Shape {
    Rect(Rect),
    Circle(Circle),
}

struct Rect {
    width: usize,
    height: usize,
}

impl ShapeI for Rect {
    type Output = usize;
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

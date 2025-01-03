#[enum_dispatch::enum_dispatch]
trait ShapeI {
    fn area(&self) -> f64;
}

struct Rect {
    width: f64,
    height: f64,
}

#[enum_dispatch::enum_dispatch(ShapeI)]
struct Shape(Rect);

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn main() {}

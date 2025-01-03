#[enum_dispatch::enum_dispatch]
trait ShapeI: std::fmt::Debug {
    fn area(&self) -> f64;
}

#[derive(Debug)]
struct Rect {
    width: f64,
    height: f64,
}

#[derive(Debug)]
#[enum_dispatch::enum_dispatch(ShapeI)]
enum Shape {
    Rect(Rect),
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn main() {
    let rect = Rect { width: 2.0, height: 3.0 };
    assert_eq!(rect.area(), 6.0);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6.0);
}

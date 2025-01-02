#[enum_dispatch::enum_dispatch]
trait ShapeI: std::fmt::Debug {
    fn area(&self) -> usize;
}

#[derive(Debug)]
struct Rect {
    width: usize,
    height: usize,
}

#[derive(Debug)]
#[enum_dispatch::enum_dispatch(ShapeI)]
enum Shape {
    Rect(Rect),
}

impl ShapeI for Rect {
    fn area(&self) -> usize {
        self.width * self.height
    }
}

fn main() {
    let rect = Rect { width: 2, height: 3 };
    assert_eq!(rect.area(), 6);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6);
}

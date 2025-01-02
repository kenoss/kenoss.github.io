#[enum_dispatch::enum_dispatch]
trait ShapeI {
    fn area(&self) -> usize;
}

struct Rect {
    width: usize,
    height: usize,
}

// enum_dispatch can't detect typos.
#[enum_dispatch::enum_dispatch(ShapeITypo)]
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

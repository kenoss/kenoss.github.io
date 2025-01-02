#[enum_dispatch::enum_dispatch]
trait ShapeI {
    fn area(&self) -> usize;
}

struct Rect {
    width: usize,
    height: usize,
}

#[enum_dispatch::enum_dispatch(ShapeI)]
struct Shape(Rect);

impl ShapeI for Rect {
    fn area(&self) -> usize {
        self.width * self.height
    }
}

fn main() {}

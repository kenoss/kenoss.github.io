use enum_delegate_v020 as enum_delegate;

mod external {
    pub trait ShapeI {
        fn area(&self) -> usize;
    }
}

struct Rect {
    width: usize,
    height: usize,
}

#[enum_delegate::implement(
    external::ShapeI,
    trait ShapeI {
        fn area(&self) -> usize;
    }
)]
enum Shape {
    Rect(Rect),
}

impl external::ShapeI for Rect {
    fn area(&self) -> usize {
        self.width * self.height
    }
}

fn main() {
    use external::ShapeI;

    let rect = Rect { width: 2, height: 3 };
    assert_eq!(rect.area(), 6);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6);
}

use enum_delegate_v020 as enum_delegate;

mod external {
    pub trait ShapeI {
        fn area(&self) -> f64;
    }
}

struct Rect {
    width: f64,
    height: f64,
}

#[enum_delegate::implement(
    external::ShapeI,
    trait ShapeI {
        fn area(&self) -> f64;
    }
)]
enum Shape {
    Rect(Rect),
}

impl external::ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn main() {
    use external::ShapeI;

    let rect = Rect { width: 2.0, height: 3.0 };
    assert_eq!(rect.area(), 6.0);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6.0);
}

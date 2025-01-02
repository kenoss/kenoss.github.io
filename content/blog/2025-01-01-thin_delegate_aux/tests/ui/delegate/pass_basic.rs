trait ShapeI {
    fn area(&self) -> f64;
}

enum Shape {
    Rect(Rect),
    Circle { circle: Circle, center: (f64, f64) }
}

impl Shape {
    delegate::delegate! {
        to match self {
            Self::Rect(x) => x,
            Self::Circle { circle, .. } => circle,
        }
        {
            fn area(&self) -> f64;
        }
    }

    fn center(&self) -> (f64, f64) {
        match self {
            Self::Rect(_) => unimplemented!(),
            Self::Circle { center, .. } => center.clone(),
        }
    }
}

struct Rect {
    width: f64,
    height: f64,
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

struct Circle {
    radius: f64,
}

impl ShapeI for Circle {
    fn area(&self) -> f64 {
        3.14 * self.radius * self.radius
    }
}

fn main() {
    let rect = Rect { width: 2.0, height: 3.0 };
    assert_eq!(rect.area(), 6.0);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6.0);
    let circle = Circle { radius: 2.0 };
    assert_eq!(circle.area(), 12.56);
    let shape = Shape::Circle { circle, center: (0.0, 0.0) };
    assert_eq!(shape.area(), 12.56);
    assert_eq!(shape.center(), (0.0, 0.0));
}

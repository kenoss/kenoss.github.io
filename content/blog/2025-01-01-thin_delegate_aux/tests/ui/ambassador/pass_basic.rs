#[ambassador::delegatable_trait]
trait ShapeI {
    fn area(&self) -> f64;
}

#[derive(ambassador::Delegate)]
#[delegate(ShapeI)]
enum Shape {
    Rect(Rect),
    Circle(Circle),
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
    let shape = Shape::Circle(circle);
    assert_eq!(shape.area(), 12.56);
}

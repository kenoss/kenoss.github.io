trait ShapeI {
    fn area(&self) -> f64;
}

struct RectWrapper(Rect);

#[delegate_attr::delegate(self.0)]
impl RectWrapper {
    fn area(&self) -> f64 {}
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

fn main() {
    let rect = Rect { width: 2.0, height: 3.0 };
    assert_eq!(rect.area(), 6.0);
    let wrapper = RectWrapper(rect);
    assert_eq!(wrapper.area(), 6.0);
}

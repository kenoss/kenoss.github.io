// It's umbiguous how to handle a trait method with default implementation.
// `portrait` does delegate them to a delegatee by default.

#[portrait::make]
trait ShapeI {
    const NAME: &'static str;

    fn area(&self) -> f64;
    fn name(&self) -> &str {
        Self::NAME
    }
}

struct RectWrapper(Rect);

#[portrait::fill(portrait::delegate(Rect; self.0))]
impl ShapeI for RectWrapper {
    const NAME: &'static str = "RectWrapper";
}

struct Rect {
    width: f64,
    height: f64,
}

impl ShapeI for Rect {
    const NAME: &'static str = "Rect";

    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn main() {
    let rect = Rect { width: 2.0, height: 3.0 };
    assert_eq!(rect.area(), 6.0);
    assert_eq!(Rect::NAME, "Rect");
    assert_eq!(rect.name(), "Rect");
    let wrapper = RectWrapper(rect);
    assert_eq!(wrapper.area(), 6.0);
    assert_eq!(RectWrapper::NAME, "RectWrapper");
    // `wrapper.name()` should be "RectWrapper" in this case.
    // assert_eq!(wrapper.name(), "RectWraper");
    assert_eq!(wrapper.name(), "Rect");
}

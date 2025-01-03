pub mod enum_dispatch {
    pub struct Rect {
        width: f64,
        height: f64,
    }

    #[enum_dispatch::enum_dispatch]
    pub enum Shape {
        Rect(Rect),
    }

    #[enum_dispatch::enum_dispatch(Shape)]
    pub trait ShapeI {
        fn area(&self) -> f64;
    }

    impl ShapeI for Rect {
        fn area(&self) -> f64 {
            self.width * self.height
        }
    }
}

pub mod enum_dispatch {
    pub struct Rect {
        width: usize,
        height: usize,
    }

    #[enum_dispatch::enum_dispatch]
    pub enum Shape {
        Rect(Rect),
    }

    #[enum_dispatch::enum_dispatch(Shape)]
    pub trait ShapeI {
        fn area(&self) -> usize;
    }

    impl ShapeI for Rect {
        fn area(&self) -> usize {
            self.width * self.height
        }
    }
}

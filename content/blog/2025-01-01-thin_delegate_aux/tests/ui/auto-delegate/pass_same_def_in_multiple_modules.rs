mod a {
    #[auto_delegate::delegate]
    pub trait Hello {
        fn hello(&self) -> String;
    }

    struct Inner(String);

    impl Hello for Inner {
        fn hello(&self) -> String {
            format!("hello, {}", self.0)
        }
    }

    #[derive(auto_delegate::Delegate)]
    struct Hoge(#[to(Hello)] Inner);
}

mod b {
    #[auto_delegate::delegate]
    pub trait Hello {
        fn hello(&self) -> String;
    }

    struct Inner(String);

    impl Hello for Inner {
        fn hello(&self) -> String {
            format!("hello, {}", self.0)
        }
    }

    #[derive(auto_delegate::Delegate)]
    struct Hoge {
        #[to(Hello)]
        inner: Inner,
    }
}

fn main() {}

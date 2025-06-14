#[macro_export]
macro_rules! with_dot {
    ($($ext:expr),*) => {
        [$(concat!(".", $ext)),*]
    };
}

#[macro_export]
macro_rules! lock {
    ($x:expr) => {{
        $x.lock().unwrap()
    }};
}
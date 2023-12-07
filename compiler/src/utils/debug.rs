#[macro_export]
macro_rules! time_init {
    () => {{
        #[cfg(feature = "debug")]
        $crate::debug::time::time_init();
    }};
}

#[macro_export]
macro_rules! time {
    ($marker:expr) => {{
        #[cfg(feature = "debug")]
        $crate::debug::time::time($marker);
    }};
}

#[macro_export]
macro_rules! display {
    ($prg:expr, $pass:ident) => {{
        #[cfg(feature = "debug")]
        $crate::debug::display::display($prg, $crate::debug::Pass::$pass);
    }};
}

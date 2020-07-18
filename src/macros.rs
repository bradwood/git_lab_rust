#[macro_export]
macro_rules! datefield {
    ($s:expr, $a:expr) => {
        Utc::now() - chrono::Duration::from_std(humantime::parse_duration($a.value_of($s).unwrap()).unwrap()).unwrap()
    };
}


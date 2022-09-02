#[macro_export]
macro_rules! assert_is_err {
    ($value: expr $(,)?) => {
        match $value {
            Ok(v) => panic!("Expected error found: Ok({:?})", v),
            Err(e) => e,
        }
    };
}

#[macro_export]
macro_rules! assert_err_is {
    ($value: expr, $msg: expr $(,)?) => {
        let err = crate::assert_is_err!($value);
        assert_eq!(format!("{}", err), $msg);
    };
}

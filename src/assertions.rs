pub fn assert_is_err<T: std::fmt::Debug, E>(value: Result<T, E>) -> E {
    match value {
        Ok(v) => panic!("Expected error found: Ok({:?})", v),
        Err(e) => e,
    }
}

pub fn assert_err_is<T: std::fmt::Debug, E: std::fmt::Display>(value: Result<T, E>, err_msg: &str) {
    let err = assert_is_err(value);
    assert_eq!(format!("{}", err), err_msg);
}

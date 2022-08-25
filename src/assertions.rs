
pub fn assert_is_err<T: std::fmt::Debug, E>(value: Result<T, E>) -> E {
    match value {
        Ok(v) => panic!("Expected error found: Ok({:?})", v),
        Err(e) => e
    }
}


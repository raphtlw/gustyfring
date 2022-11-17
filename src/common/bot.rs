/// Use this macro to send a reply, returning from the function
///
/// This expands to `Ok(Some(String::from(*)))`
///
/// ### Usage
///
/// ```
/// respond!("deez nuts")
/// ```
macro_rules! respond {
    (
        $response:literal
    ) => {
        return Ok(Some(String::from($response)))
    };
    (
        $response:expr
    ) => {
        return Ok(Some(String::from($response)))
    };
}
pub(crate) use respond;

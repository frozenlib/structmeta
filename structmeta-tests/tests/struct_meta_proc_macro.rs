use structmeta_tests::*;
#[allow(dead_code)]
#[test]
fn test_proc_macro_derive() {
    #[derive(MyMsg)]
    #[my_msg(msg = "abc")]
    struct TestType;

    assert_eq!(MSG, "abc");
}

#[test]
fn test_proc_macro_attr() {
    #[my_attr(msg = "xyz")]
    struct TestType;

    assert_eq!(MSG, "xyz");
}

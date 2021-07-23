use enterpolation_attribute::chain_result;

#[test]
fn mutable_reference() {
    struct Test {}
    struct TestError {}
    struct Test2 {
        inner: Result<Test, TestError>,
    }

    #[chain_result(Test2,TestError)]
    impl Test {
        fn something(&mut self) -> Test {
            Test{}
        }
    }
    // There should be a function `something` for Test2
    let test = Test2 {inner: Ok(Test{})};
    test.something();
}

#[test]
fn move_syntax() {
    struct Test {}
    struct TestError {}
    struct Test2 {
        inner: Result<Test, TestError>,
    }

    #[chain_result(Test2,TestError)]
    impl Test {
        fn something(self) -> Test {
            Test{}
        }
    }
    // There should be a function `something` for Test2
    let test = Test2 {inner: Ok(Test{})};
    test.something();
}

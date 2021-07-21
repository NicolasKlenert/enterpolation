use enterpolation_attribute::chain_result;

#[test]
fn impl_block() {
    struct Test {}
    struct Test2 {}
    #[chain_result(Test2,TestError)]
    impl Test {
        fn something() -> bool {
            true
        }
    }
    // There should be a function `something` for Test2
    assert!(Test2::something());
}

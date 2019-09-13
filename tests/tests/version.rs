#[test]
fn test_version() {
    let out = cargo_eval!("--version").unwrap();
    assert!(out.success());
    scan!(&out.stdout;
        ("cargo-eval", &::std::env::var("CARGO_PKG_VERSION").unwrap(), .._) => ()
    )
    .unwrap();
}

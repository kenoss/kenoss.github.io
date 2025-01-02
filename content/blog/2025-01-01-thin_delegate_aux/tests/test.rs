#[test]
fn ui_test() {
    let t = trybuild::TestCases::new();
    for crate_name in [
        "ambassador",
        "auto-delegate",
        "delegate",
        "delegate-attr",
        "enum_delegate_v020",
        "enum_delegate_v030",
        "enum_dispatch",
        "portrait",
    ] {
        t.pass(format!("tests/ui/{crate_name}/pass_*.rs"));
        t.compile_fail(format!("tests/ui/{crate_name}/fail*.rs"));
    }
}

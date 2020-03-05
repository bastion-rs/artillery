#[test]
fn test_launcher() {
    use std::fs;

    let t = trybuild::TestCases::new();

    for entry in fs::read_dir("fptests").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        dbg!(path.clone());
        if !path
            .clone()
            .into_os_string()
            .into_string()
            .unwrap()
            .contains("launcher")
        {
            t.pass(path);
        }
    }
}

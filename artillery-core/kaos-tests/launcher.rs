#[test]
fn kaos() {
    use std::fs;
    use std::time::Duration;

    let k = kaos::Runs::new();

    for entry in fs::read_dir("kaos-tests").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        dbg!(path.clone());
        if !path
            .clone()
            .into_os_string()
            .into_string()
            .unwrap()
            .contains("launcher")
            && !path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap()
                .contains("mod")
            && !path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap()
                .contains("chaos")
        {
            // Every service run should be available at least 2 seconds
            k.available(path, Duration::from_secs(2));
        }
    }
}

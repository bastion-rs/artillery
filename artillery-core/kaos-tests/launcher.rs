#[test]
fn chaos_tests() {
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
            .contains("launcher") // Filter out itself
            && !path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap()
                .contains("mod") // Filter out module hierarchy
            && !path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap()
                .contains("base")
        // Filter out common code as test
        {
            if path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap()
                .contains("chaotic")
            // Chaotic test rather than availability
            {
                // Let's have 5 varying runs.
                let run_count = 5;

                // Minimum availability to expect as milliseconds for the runs.
                // Which corresponds as maximum surge between service runs.
                // Let's have it 10 seconds.
                let max_surge = 10 * 1000;

                // Run chaotic test.
                k.chaotic(path, run_count, max_surge);
            } else {
                // Every service run should be available at least 2 seconds
                k.available(path, Duration::from_secs(2));
            }
        }
    }
}

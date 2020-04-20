use criterion::{criterion_group, criterion_main, Criterion};

use artillery_ddata::craq::prelude::*;

use futures::stream::StreamExt;
use rand::distributions::Alphanumeric;
use rand::prelude::*;

pub fn entry_bench_read(sip: String, args: Vec<&str>) -> Vec<DDataCraqClient> {
    // Connect to servers
    let num_clients: usize = args[0].parse::<usize>().unwrap();
    let num_bytes: usize = args[1].parse::<usize>().unwrap();
    let _trials: usize = args[2].parse::<usize>().unwrap();
    let num_servers = args.len() - 3;

    let hpc: Vec<&str> = sip.split(":").collect();
    let mut hosts = vec![(hpc[0], hpc[1].parse::<u16>().unwrap())];

    (0..num_servers).into_iter().for_each(|i| {
        let hpc: Vec<&str> = args[i + 3].split(":").collect();
        let host = hpc[0];
        let port = hpc[1];
        let port = port.parse::<u16>().unwrap();

        hosts.extend([(host, port)].iter());
    });

    let mut clients: Vec<DDataCraqClient> = (0..num_clients)
        .into_iter()
        .map(|i| {
            let (host, port) = hosts[i % hosts.len()];
            DDataCraqClient::connect_host_port(host, port).unwrap()
        })
        .collect();

    if clients[0].write(gen_random_str(num_bytes)).is_err() {
        println!("bench_write: Couldn't write new revision.");
    }

    // Check if any object is written...
    if clients[0].read(CraqConsistencyModel::Strong, 0).is_err() {
        println!("bench_read: Could not read object.");
    }

    clients
}

pub fn gen_random_str(slen: usize) -> String {
    thread_rng().sample_iter(&Alphanumeric).take(slen).collect()
}

pub fn stub_read(clients: &mut Vec<DDataCraqClient>) {
    clients.iter_mut().for_each(|client| {
        let _ = client.read(CraqConsistencyModel::Eventual, 0);
    });
}

fn client_benchmarks(c: &mut Criterion) {
    {
        // 1 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["1", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_1_client", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 2 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["2", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_2_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 3 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["3", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_3_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 4 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["4", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_4_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 5 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["5", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_5_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 10 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["10", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_10_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 20 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["20", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_20_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 30 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["30", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_30_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 40 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["40", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_40_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 50 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["50", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_50_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    {
        // 100 clients
        let mut clients = entry_bench_read(
            "127.0.0.1:30001".to_string(),
            vec!["100", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"],
        );
        c.bench_function("benchmark_read_100_clients", |b| {
            b.iter(|| stub_read(&mut clients))
        });
    }

    // {
    //     // 500 clients
    //     let mut clients = entry_bench_read("127.0.0.1:30001".to_string(), vec!["500", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"]);
    //     c.bench_function("benchmark_read_500_clients", |b| b.iter(|| stub_read(&mut clients)));
    // }

    // {
    //     // 1000 clients
    //     let mut clients = entry_bench_read("127.0.0.1:30001".to_string(), vec!["1000", "1000", "100", "127.0.0.1:30002", "127.0.0.1:30003"]);
    //     c.bench_function("benchmark_read_1000_clients", |b| b.iter(|| stub_read(&mut clients)));
    // }
}

criterion_group!(benches, client_benchmarks);
criterion_main!(benches);

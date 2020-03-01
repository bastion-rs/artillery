---
title: 'Primitives'
---

<Block>
# Primitives
</Block>


<Block>

Artillery Core consists of various primitives. We will start with Service Discovery primitives and pave out way to Cluster primitives.

</Block>


<Block>

## Service Discovery Primitives

For distributed operation we need to have a service discovery to find out who is operating/serving which services and service capabilities.

Our design consists of various service discovery techniques.

</Block>


<Block>

## UDP Anycast

We have UDP anycast which allows the devices in the same network to nag each other continuously with a specific set of service requests to form a cluster initiation.

**NOTE:** Convergance of the UDP anycast might take longer time than the other zeroconf approaches.

<Example>

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, PartialEq)]
struct ExampleSDReply {
    ip: String,
    port: u16,
}

let epidemic_sd_config = ExampleSDReply {
    ip: "127.0.0.1".into(),
    port: 1337, // Cluster Formation Port of this instance
};

let reply = ServiceDiscoveryReply {
    serialized_data: serde_json::to_string(&epidemic_sd_config).unwrap(),
};

// Initialize receiver channels
let (tx, discoveries) = channel();

// Register seeker endpoint
sd.register_seeker(tx).unwrap();

// Sometimes you seek for nodes,
// sometimes you need to be a listener to respond them.
if let Some(_) = seeker {
   sd.seek_peers().unwrap();
} else {
   sd.set_listen_for_peers(true).unwrap();
}

for discovery in discoveries.iter() {
    let discovery: ExampleSDReply =
        serde_json::from_str(&discovery.serialized_data).unwrap();
    if discovery.port != epidemic_sd_config.port {
        debug!("Seed node address came");
        let seed_node = format!("{}:{}", discovery.ip, discovery.port);
        // We have received a discovery request.
    }
}
```

</Example>

</Block>

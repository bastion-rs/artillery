<div align="center">
  <img src="https://github.com/bastion-rs/artillery/blob/master/img/artillery_cropped.png" width="512" height="512"><br>
</div>

-----------------

<h1 align="center">Artillery: Cluster management & Distributed data protocol</h1>


It contains the modules below:
* `artillery-ddata`: Used for distributed data replication
* `artillery-core`: Contains:
    * `cluster`: Prepared self-healing cluster structures
    * `epidemic`: Infection style clustering
    * `service_discovery`: Service discovery types
        * `mdns`: MDNS based service discovery
        * `udp_anycast`: UDP Anycast based service discovery 
* `artillery-hierman`: Supervision hierarchy management layer (aka Bastion's core carrier protocol)

## Examples
Below you can find examples to learn Artillery.
You can also take a look at the [Core Examples](https://github.com/bastion-rs/artillery/tree/master/artillery-core/examples).

### Launching a local AP Cluster
To spawn a local AP cluster at any size you can use the command below in the root directory of the project:
```bash
$ deployment-tests/cluster-mdns-ap-test.sh -s 50
```

Argument `-s` defines the amount of nodes in the cluster.
To shut down the cluster either use:
```bash
$ killall cball_mdns_sd_infection
```
or kill processes one by one to see that cluster is self-healing.

---
title: 'Local Examples'
---

<Block>
# Local Examples
</Block>

<Block>

## Cluster Examples
Below you can find examples to learn Artillery.
You can also take a look at the [Core Examples](https://github.com/bastion-rs/artillery/tree/master/artillery-core/examples).

</Block>

<Block>

## Launching a local AP Cluster
To spawn a local AP cluster at any size you can use the command below in the root directory of the project.

<Example>

```bash
$ deployment-tests/cluster-mdns-ap-test.sh -s 50
```

```bash
$ killall cball_mdns_sd_infection
```

</Example>

Argument `-s` defines the amount of nodes in the cluster.
To shut down the cluster either use `killall` or kill processes
one by one to see that cluster is self-healing.

</Block>

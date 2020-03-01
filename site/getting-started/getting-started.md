---
title: 'Getting Started'
---

<Block>
# Getting Started
</Block>


<Block>

## Basics

To use Artillery, you need to evaluate your requirements for distributed operation very carefully.
Every layer in artillery is usable modularly. Artillery uses "Take it or leave it" approach.
If you don't need it you don't include.

Artillery consists of various layers. Layers can have various consistency degree and capability model.
Artillery layers are build on top each other. Most basic layer is `Core`.
Core layer contains various prepared cluster configurations.
Currently it is supporting:
* **AP(Availability, Partition Tolerance** Cluster mode
* **CP(Consistency, Partition Tolerance)** Cluster mode (soon)

In addition to cluster modes, it contains primitives to build your own cluster structures for your own designated environment.

<Example>

* `artillery-core`
    * `cluster`: Prepared self-healing cluster structures
    * `epidemic`: Infection style clustering
    * `service_discovery`: Service discovery types
        * `mdns`: MDNS based service discovery
        * `udp_anycast`: UDP Anycast based service discovery 
(aka [Bastion](https://bastion.rs)'s core carrier protocol)

</Example>

</Block>

<Block>

## Distributed Data

You might want to pass by the distributed configuration part and directly looking forward to have a distributed
data primitives. Like replicating your local map to some other instance's local map etc.

This is where `Ddata` package kicks in. `Ddata` supplies the most basic distributed data dissemination at the highest abstraction level.

<Example>

* `artillery-ddata`: Used for distributed data replication

</Example>

</Block>


<Block>

## Hierarchy Management

This layer is specifically build for Bastion and it's distributed communication.
It contains a Hierarchy Management protocol. This protocol manages remote processes, links as well as their state.

<Example>

* `artillery-hierman`: Supervision hierarchy management layer

</Example>
</Block>

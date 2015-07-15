========
Concepts
========


Overview
========

The Cantal is a monitoring system designed specifically for real-time
measuring and load-balancing distributed computing clusters.

The essential part of cantal is a **Cantal Agent** it does the following
things:

* Scans for local metrics at **2 second interval**
* Preserves **one hour** history of all metrics data with *100% precision*
  (compressed)
* Provides **web interface** for viewing local metrics
* Has peer to peer **discovery** mechanism
* On demand provides aggregated statistics **over cluster**

Additional features:

* Cantal is aware of linux containers
* Almost zero-cost connunication between processes and agent
* Written in rust, so can track thousands of metrics with 2 second precision
  in less than couple of percents of a single CPU core

The project also consists of:

* Protocol to submit data to agent at nearly zero cost
* Command-line tool to view data locally without running agent


Background
==========

Since Cantal is designed for real-time load balancing, it has very strong and
very specific requirements:

1. Very high precision (cantal scans at 2 second interval, while common case
   is about a minute interval, very rarely interval is 10 seconds or less)
2. Similarly very fast collection of metrics across large cluster
3. Discovering trends quickly (i.e. having 30 value snapshots per minute we can
   find out load growth in a fraction of minute)
4. High availability (no master, quorum or similar)
5. Being able to observe individual nodes in case of partitioning
6. Lightweight


Design Decisions
================

Here is short roundup of all the important design decisions. Some of them
are described in detail in the following sections.

(1) Agent has embedded web server. So you can point your browser to::

  http://node.domain.in.local.network:22682

And see all the statistics on the node.

(2) Agent stores history locally. So we don't loose stats in case of
network failure

(3) Agent has peer to peer gossip-like discovery with UDP. So we don't rely on
any other discovery mechanism when time comes to gather metrics over cluster.
Note: we do use UDP only for discovery, so we don't loose statistics when
network is lossy.

(4) You can ask **any instance** of agent to get metrics for whole cluster. This
is how we allow to get data over whole cluster with a single HTTP request. But
we do it *lazily*, so that we don't have full mesh of connections. I.e. when
first client asks, we connect to every node by TCP and subcribe for connections.


Discovery
=========


Aggregated Metrics
==================


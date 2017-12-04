====================
Daemon Configuration
====================


We're doing our best to keep cantal working without any configuration. But
for achieving complex tast we need some configuration.

Important command-line options:

1. Enable cluster setup ``--cluster-name=your-name``. Name must be the same
   on all nodes in the cluster (i.e. all nodes which should see each other)
2. Keep some metrics for restart ``--storage-dir=/var/lib/cantal``. In
   clustered setup this also stores list of peers, so that if all the nodes
   are restarted simultaneously, they discover each other


Cluster Setup
=============

Another piece of cluster setup is: introduce nodes to each other::

    curl http://some.known.host:22682/add_host.json -d '{"addr": "1.2.3.4:22682"}'"

This only works if ``cluster-name`` matches and after nodes are able to
interchange ping-pong packets between each other (also ``machine-id`` must be
different which is usually provided by the system).

Cantal Changes by Version
=========================


.. _changelog-0.6.0:

Cantal 0.6.0
------------

* We reworked network subsystem to use tokio instead of home-grown async, this
  looses some features for now, but is an important step for future
* Breaking: remote subsystem doesn't work, including the whole ``/remote``
  route, we will be working to add feature back soon
* Feature: add graphql API (only status for now)
* Breaking: ``/status.json`` contains less data, use graphql API

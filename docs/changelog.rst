Cantal Changes by Version
=========================


.. _changelog-0.6.8:

Cantal 0.6.8
------------

* Bugfix: machine uptime and process' uptimes were broken in UI


.. _changelog-0.6.7:

Cantal 0.6.7
------------

* Bugfix: previously cantal could crash when time jumps backwards. Currently,
  it waits (in metrics scanner code) if delta is < 10 sec and crashes with
  clear log message for large time jumps.


.. _changelog-0.6.6:

Cantal 0.6.6
------------

* Bugfix: when a ``CANTAL_PATH`` referring to a same file is encountered in
  multiple processes we no longer duplicate metrics


.. _changelog-0.6.5:

Cantal 0.6.5
------------

* Feature: add ``peers`` graphql field


.. _changelog-0.6.4:

Cantal 0.6.4
------------

* Feature: add ``local.cgroups`` graphql endpoint
* Feature: add ``local.processes`` graphql endpoint


.. _changelog-0.6.3:

Cantal 0.6.3
------------

* Bugfix: add ``num_peers``, ``num_stale`` back to ``/status.json``, same
  fields added to graphql endpoint


.. _changelog-0.6.2:

Cantal 0.6.2
------------

* Bugfix: larger timeouts for incoming http requests
* Bugfix: add ``version`` back to ``/status.json``


.. _changelog-0.6.1:

Cantal 0.6.1
------------

* Bugfix: fix JS error on /local/peers page


.. _changelog-0.6.0:

Cantal 0.6.0
------------

* We reworked network subsystem to use tokio instead of home-grown async, this
  looses some features for now, but is an important step for future
* Breaking: remote subsystem doesn't work, including the whole ``/remote``
  route, we will be working to add feature back soon
* Feature: add graphql API (only status for now)
* Breaking: ``/status.json`` contains less data, use graphql API

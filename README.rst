======
Cantal
======

:Status: beta
:Documentation: http://cantal.readthedocs.io/

Cantal is an expermimental heartbeating, monitoring and statistics solution.
Main design goals:

* Nearly zero-cost for application to send data
* Fine grained statistics and current state info
* Decentralized and highly available

Cantal consists of:

* A protocol to submit monitoring data to local agent
* The reference implementation of the library for python (cantal-py_)
* Command-line tool to view data
* Local agent to collect/aggregate/forward data
* A protocol for forwarding data to aggregator (carbon/graphite)

License
=======

Licensed under

* MIT license (LICENSE_ or http://opensource.org/licenses/MIT)

Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, shall be licensed under
the MIT license, without any additional terms or conditions.

.. _LICENSE: https://github.com/tailhook/cantal-py
.. _cantal-py: https://github.com/tailhook/cantal-py


===
API
===

Policy
======

Currently Cantal has ``/v1/`` API. We don't increment API version on backwards
compatible changes. The following is deemed backwards-compatible:

* Addition of new resources
* Addition of new fields in structures
* Deprecation (but not removal) of resources
* Deprecation (but not removal) of fields in structures
* New formats of output (with some negotiation way, i.e. Accept header)

The (backwards-compatible) changes in API are listed here by version of a
Cantal agent itself.

Until cantal reaches ``1.0`` it's only guaranteed to support single API version,
after ``1.0`` we will support previous version of API for several releases after
new API is introduced.

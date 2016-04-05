===================
Memory Map Protocol
===================


Motivation
==========

Cantal scans the whole system at 2 second interval. This includes metrics of
your application. If cantal would poll applications by some kind of remote
procedure call (RPC), it would rely too much on the application responsiveness
to provide fine-grained statistics. For many synchronous applications it just
doesn't work, because it may wait more than 2 seconds for a database on occassion).

Cantal is implemented with another kind of inter-process communication (IPC),
the shared memory. As you will see later in the text it requires almost zero
configuration and allows efficient collection of statistics for most kinds
of programs. For scripting languages it's also practially zero-cost. For
fully-threaded programs it's usually as cheap as any other way you could
implement.

For scripting languages using shared memory appoach described here also allows
to dive into the application that is currently slow or unresponsive.


Overview of Files and Discovery
===============================

The metrics are discovered by cantal by scanning environment variables of
running processes. Whenever it sees ``CANTAL_PATH`` in environment the
metrics are gathered from there.

For ``CANTAL_PATH=/run/myapp``, catal will look into:

* ``/run/myapp.meta`` for metadata (names size and alignemnt) for metrics
* ``/run/myapp.values`` for metrics

Here is a short example of the contents of the meta file::

    counter 8: {"metric": "requests.number"}
    counter 8: {"metric": "requests.duration", "unit": "ms"}

It contains two 8 byte (64bit) unsigned integers, which are growing counters.
Here is the respective ``.values`` file (displayed in a format of
``hexdump -C``)::

    00000000  61 00 00 00 00 00 00 00  67 62 00 00 00 00 00 00  |a.......gb......|

Here we can see that there have been 97 requests (0x61) each lasts of
almost 260 milleconds on average (0x6261/0x61).

The files must reside on some in-memory file system (``tmpfs``). On typical
system ``/run`` folder is a good place. On some systems ``/tmp`` is a ``tmpfs``
too, but be careful not to put it into HDD or SSD. In docker_ containers you
need to map some tmpfs folder from host system (example command line:
``docker run -v /run/containers/my1:/run/cantal -e CANTAL_PATH=/run/cantal ...``)
In container running by lithos_ a ``!Statedir`` is a good place.


Metadata File Format
====================

File format of the meta data is simple: every next line is the metric (or a
padding). Format of the line::

    TYPE NUMBER_OF_BYTES: JSON_METADATA

For example::

    counter 8:  {"metric": "memoryusage"}

The ``JSON_METADATA`` field is a subset of a JSON, and is currently limited to
(we may extend it to a larger subset of or full JSON later):

1. Serialized data should contain no newlines (you can't pretty print json)
2. Only a dictionary (object) with string keys and string values is supported

The keys and the values of the dictionary might be arbitrary. But the whole
set of keys must be unique for the file.

An a padding is just::

    pad 123

Where ``123`` is the number of bytes.

The values of the respective lengths are stored consecutively in the
``.values`` file in the same order as entries in metadata. The ``pad`` entries
might be used to align counters to addresses of multiples of 8, or whatever is
needed for efficient accounting.

Metadata file is **imutable**. To create a metadata file you must write to
a temporary name then do an atomic rename operation to put it to the right
path.


Values File Format
==================

Values file is a binary file that contains raw values in host byte order
written consecutively one after another with actually any file structure,
or in other words with the structure defined in metadata file. For example,
if we have a metadata of::

    counter 8: {"metric": "requests.number"}
    counter 8: {"metric": "requests.duration", "unit": "ms"}
    pad 48
    state 64: {"value": "request.sql.request"}

There are exactly:

* 8 bytes, integer in host byte order, counter of the number of requests
* 8 bytes, integer in host byte order, sum of the duration of all requests
* 48 bytes of padding, any garbage can be there, but usually just zeros
* 64 bytes state, first bytes of the sql request that is currently going on

The file size is 128 bytes. As you can see the state is aligned to 64 bytes,
because this makes it another CPU cache line. This means two processors can
write counters and state simultatenously without any kind of contention. This
is very CPU-dependent and optional for file format, but usually some kind of
padding is implemented by the implementation.

Overall, the structure of the file is implemented this way so that program
can atomically adjust any counter directly in shared memory without ever
duplicating metrics or delaying the statistics submission (for some
very fast and heavy-threaded programs it may still be a lot of contention and
traditional technics may be applied here, but please do benchmarks first).


Data Types
==========

Data types that are currently supported by cantal agent:

============= ================ =============== ===============================
  Type Name    Allowed Sizes   Alignment       Description
                               (recommended)
============= ================ =============== ===============================
``counter``   8 bytes (64bit)   8 bytes        A 64bit ever-growing counter.
``level``     8 bytes (64bit)   8 bytes        A current value of something,
                                               may grow or decrease
``state``     16-65535 bytes    64 bytes       An arbitrary string value that
                                               is visible in cantal. No
                                               history of it is stored.
``pad``       1-65535 bytes     --             No data
============= ================ =============== ===============================

More types and sizes will be implemented later.

The ``counter`` value is a most useful type. You should increment the
value of counter using atomic operations (unless you have a GIL so any small
write is atomic) and never write whole value to it. It's fine to initialize
counter value to zero on application restart, you don't need to store value
somewhere.

Good use cases for ``counter`` are:

1. Number of requests
2. Total duration of requests
3. Tasks processed

From the above you can derive the following values, which you **should not
write by the application**, but they are calculated by a cantal itself:

1. The number requests per second (or any other unit in time)
2. The average duration of each request
3. Tasks processed per second

Good use cases for ``level`` are:

1. Memory used by object pool
2. Current queue size

*Don't use* ``level`` *for things that are number of operations per second or
similar things. Use* ``counter`` *instead. This allows correct statistics even
if collection interval changes, when something is slow and so on.*



.. _lithos: http://lithos.readthedocs.org
.. _docker: http://docker.com

import time
import json
import struct
import warnings
import os
import abc
import mmap
import atexit
from contextlib import contextmanager
from itertools import groupby


_timestr = struct.Struct('L')
CACHE_LINE_SIZE = 64
PAGE_SIZE = 4096


class DuplicateValueException(RuntimeError):
    """Raised when all parameters to value are duplicated"""


class Collection(object):
    """A collection of statistics parameters

    It's just a singleton, which is hold in this module, but we use it
    as a regular class for unittests
    """

    def __init__(self):
        self._all_values = {}

    def add(self, dimensions, value):
        name = json.dumps(dimensions)
        if name in self._all_values:
            raise DuplicateValueException(
                "Counter {} is already defined".format(name))
        self._all_values[name] = value

    def start(self, path):
        values = list(self._all_values.items())
        del self._all_values

        get_size = lambda pair: pair[1]._get_size()
        counter_order = lambda pair: (pair[1]._get_size(), pair[0])
        values.sort(key=counter_order)

        offset = 0
        scheme = []
        offsets = {}
        for size, pairs in groupby(values, key=get_size):
            if size & (size-1) == 0:  # power of two; let's optimize
                if offset % size:
                    pad = size - offset % size
                    offset += pad
                    scheme.append('pad {}'.format(pad))
            elif size % 8 == 0:
                # unless value is small or
                # it's size is crappy we must align to 8
                if offset % 8:
                    pad = size - offset % 8
                    offset += pad
                    scheme.append('pad {}'.format(pad))
            for name, value in pairs:
                offsets[value] = offset
                offset += size
                _, typ = value._get_type()
                scheme.append(typ + ': ' + name)

        size = offset

        tmppath = path + '.tmp'
        metapath = path + '.meta'

        if os.path.exists(metapath):
            os.path.unlink(metapath)
        if os.path.exists(path):
            os.path.unlink(path)
        if os.path.exists(tmppath):
            os.path.unlink(tmppath)

        with open(tmppath, 'w+b') as f:
            # We could use truncate, but it doesn't enlarge file in
            # cross-platform fasion. Fortunately our data is small and usually
            # in RAM anyway
            f.write(b'\x00' * offset)
            f.flush()
            mem = memoryview(mmap.mmap(f.fileno(), offset))

        os.rename(tmppath, path)

        with open(tmppath, 'wt') as f:
            f.write('\n'.join(scheme))

        os.rename(tmppath, metapath)

        for value, offset in offsets.items():
            vtype, _ = value._get_type()
            size = value._get_size()
            value._memoryview = mem[offset:offset+size].cast(vtype)

        return ActiveCollection(path)


class ActiveCollection(object):

    def __init__(self, path):
        self._path = path

    def add(self, dimensions, value):
        raise RuntimeError(
            "Counters can't be added after collection.start()")

    def start(self, path):
        raise RuntimeError("The start() method already called")

    def close(self):
        os.unlink(self._path + '.meta')
        os.unlink(self._path)


global_collection = Collection()


class _Value(abc.ABC):
    __slots__ = ()

    def __init__(self, *, collection=global_collection, **kwargs):
        collection.add(kwargs, self)

    @abc.abstractmethod
    def _get_size(self):
        pass


class Counter(_Value):
    __slots__ = ('_memoryview')

    def __iadd__(self, value):
        self._memoryview[0] += value
        return self

    def incr(self, value=1):
        self._memoryview[0] += value

    def _get_size(self):
        return 8

    def _get_type(self):
        return ('L', 'counter u64')


class Float(_Value):
    __slots__ = ('_memoryview')

    def incr(self, value=1):
        self._memoryview[0] = value

    __iadd__ = incr

    def _get_size(self):
        return 8

    def _get_type(self):
        return ('d', 'level f64')

    def set(self, value):
        self._memoryview[0] = value

    def __setitem__(self, key, value):
        assert key == 0, "Only single value is expected"
        self._memoryview[key] = value


class Integer(_Value):
    __slots__ = ('_memoryview')

    def _get_size(self):
        return 8

    def _get_type(self):
        return ('l', 'level f64')

    def set(self, value):
        self._memoryview[0] = value

    def __setitem__(self, key, value):
        assert key == 0, "Only single value is expected"
        self._memoryview[key] = value


class State(_Value):
    __slots__ = ('_memoryview', 'size')
    HEADER_SIZE = _timestr.size
    assert HEADER_SIZE == 8, 'We use constants in enter/exit/context'

    def __init__(self, size=CACHE_LINE_SIZE-HEADER_SIZE, **kwargs):
        sz = size + self.HEADER_SIZE
        if sz & (sz - 1) or sz % CACHE_LINE_SIZE:
            warnings.warning(
                "Size of state counter should be multiple of {} or smaller"
                "power of two sans header size ({}), perfect size is {}"
                .format(CACHE_LINE_SIZE, self.HEADER_SIZE,
                        CACHE_LINE_SIZE - self.HEADER_SIZE))
        self.size = size
        super().__init__(**kwargs)

    def _get_size(self):
        return self.HEADER_SIZE + self.size

    def _get_type(self):
        return ('B', 'state {}'.format(self.size + self.HEADER_SIZE))

    @contextmanager
    def enter(self, value):
        encoded = value.encode('utf-8')
        le = len(encoded)
        tail = le - self.size
        if tail < 0:
            encoded += b'\x00'
            le += 1
        elif tail > 0:
            encoded = encoded[:self.size]
        le += 8
        chunk = _timestr.pack(int(time.time()*1000)) + encoded
        self._memoryview[0:le] = chunk

    @contextmanager
    def context(self, value):
        self.enter(value)
        yield self
        self._memoryview[0:8] = b'\x00\x00\x00\x00\x00\x00\x00\x00'

    def exit(self):
        self._memoryview[0:8] = b'\x00\x00\x00\x00\x00\x00\x00\x00'


def start(path=None):
    global global_collection
    if path is None:
        path = os.environ.pop("CANTAL_PATH", None)
    if path is None:
        if 'XDG_RUNTIME_DIR' in os.environ:
            path = '{}/cantal.{}'.format(
                os.path.join(os.environ['XDG_RUNTIME_DIR']),
                os.getpid())
        else:
            path = '/tmp/cantal.{}.{}'.format(
                os.getuid(),
                os.getpid())

    global_collection = global_collection.start(path)
    atexit.register(global_collection.cleanup)

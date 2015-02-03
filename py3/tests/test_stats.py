import os
import struct
import textwrap
from cantal import Collection, Counter, Float, Integer, State
from unittest import TestCase


class TestBase(TestCase):

    def setUp(self):
        self.collection = Collection()

    def start(self):
        path = '{}/cantal-test.{}'.format(
            os.path.join(os.environ.get('XDG_RUNTIME_DIR', '/tmp')),
            os.getpid())
        self._path = path
        active = self.collection.start(path)
        self.addCleanup(active.close)
        del self.collection

    def counter(self, **kwargs):
        return Counter(collection=self.collection, **kwargs)

    def float(self, **kwargs):
        return Float(collection=self.collection, **kwargs)

    def integer(self, **kwargs):
        return Integer(collection=self.collection, **kwargs)

    def state(self, **kwargs):
        return State(collection=self.collection, **kwargs)

    def assertRead(self, value, offset=0):
        with open(self._path + '.values', 'rb') as file:
            file.seek(offset, 0)
            self.assertEqual(file.read(), value)

    def assertMeta(self, value):
        with open(self._path + '.meta', 'rt') as file:
            self.assertEqual(file.read(), textwrap.dedent(value).strip())


class TestValues(TestBase):

    def test_counter(self):
        cnt = self.counter(name="hello")
        self.start()
        cnt += 1
        self.assertRead(struct.pack('L', 1))
        cnt += 3
        self.assertRead(struct.pack('L', 4))
        cnt.incr(1234)
        self.assertRead(struct.pack('L', 1238))

    def test_float(self):
        cnt = self.float(name="hello")
        self.start()
        cnt[0] = 1.5
        self.assertRead(struct.pack('d', 1.5))
        cnt[0] = 0.75
        self.assertRead(struct.pack('d', 0.75))
        cnt.set(3.75)
        self.assertRead(struct.pack('d', 3.75))
        cnt.set(32)
        self.assertRead(struct.pack('d', 32))

    def test_int(self):
        cnt = self.integer(name="hello")
        self.start()
        cnt[0] = 3
        self.assertRead(struct.pack('l', 3))
        cnt[0] = -1000
        self.assertRead(struct.pack('l', -1000))
        cnt.set(123564)
        self.assertRead(struct.pack('l', 123564))

    def test_state(self):
        state = self.state(name="hello", value="world")
        self.start()
        with state.context('job1'):
            self.assertRead(b'job1' + b'\x00'*52, 8)
        # we leave garbage, but assert that timestamp is gone
        self.assertRead(b'\x00'*8 + b'job1' + b'\x00'*52)

        state.enter('some_longer_job_name')
        self.assertRead(b'some_longer_job_name' + b'\x00'*36, 8)
        state.exit()

        self.assertRead(b'\x00'*8 + b'some_longer_job_name' + b'\x00'*36)
        with state.context('short'):
            self.assertRead(b'short\x00onger_job_name' + b'\x00'*36, 8)
        self.assertRead(b'\x00'*8 + b'short\x00onger_job_name' + b'\x00'*36)


class TestScheme(TestBase):

    def test_two_counters(self):
        self.counter(name="1")
        self.counter(name="2")
        self.start()
        self.assertMeta("""
            counter 8: {"name": "1"}
            counter 8: {"name": "2"}
        """)

    def test_counter_float(self):
        self.counter(name="2")
        self.float(name="1")
        self.start()
        self.assertMeta("""
            level 8 float: {"name": "1"}
            counter 8: {"name": "2"}
        """)

    def test_counter_state(self):
        self.counter(name="2")
        self.state(name="1")
        self.start()
        self.assertMeta("""
            counter 8: {"name": "2"}
            pad 56
            state 64: {"name": "1"}
        """)

    def test_2counters_state(self):
        self.counter(name="1")
        self.integer(name="2")
        self.counter(name="3")
        self.state(name="100")
        self.start()
        self.assertMeta("""
            counter 8: {"name": "1"}
            level 8 signed: {"name": "2"}
            counter 8: {"name": "3"}
            pad 40
            state 64: {"name": "100"}
        """)

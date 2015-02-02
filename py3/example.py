import os
import time
import cantal

wakeups = cantal.Counter(name='wakeups')
sleep_time = cantal.Float(name='last_sleep')
num_fds = cantal.Integer(name='num_fds')
state = cantal.State(name='main_loop')


def main():
    cantal.start()
    while True:

        with state.context('sleeping'):
            start = time.time()
            time.sleep(0.2)
            wakeups.incr(1)
            sleep_time.set(time.time() - start)

        with state.context('checking_fds'):
            num_fds.set(len(os.listdir('/proc/self/fd')))
            time.sleep(0.1)  # just so we notice state


if __name__ == '__main__':
    main()

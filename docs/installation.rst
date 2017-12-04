============
Installation
============

Ubuntu
======

You need rust_ compiler::

    wget https://static.rust-lang.org/dist/rust-1.1.0-x86_64-unknown-linux-gnu.tar.gz
    tar -xzf rust-1.1.0-x86_64-unknown-linux-gnu.tar.gz
    cd rust-1.1.0-x86_64-unknown-linux-gnu
    sudo ./install.sh

Or you may use `instructions on official website`_

Additional dependencies:

    sudo apt-get install build-essential libssl-dev

Then just download and build project with cargo::

    wget https://github.com/tailhook/cantal/archive/staging.tar.gz
    cd cantal-staging
    cargo build --release

.. note:: We build from *staging* branch, because that contains javascripts
   already built. Building javascripts is a little bit more complex process,
   you shouldn't do, unless you're developing cantal itself.

Then you may either install it with::

    make install

Optionally ``DESTDIR`` and ``PREFIX`` environment vars work.

Or you can build a package with::

      checkinstall --default \
        --pkglicense=MIT --pkgname=cantal \
        --pkgversion="$(cat version.txt)" \
        --requires="libssl1.0.0"
        --nodoc --strip=no \
        make install

Additionally you need an ``upstart`` or ``systemd`` script to start cantal
as a service. Here is one example::

    start on runlevel [2345]
    respawn
    exec /usr/bin/cantal-agent --host 0.0.0.0 --port 22682 \
        --storage-dir /var/lib/cantal

.. _rust: http://rust-lang.org

.. _`instructions on official website`: http://www.rust-lang.org/install.html

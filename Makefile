RUSTC ?= rustc -C opt-level=3

PREFIX ?= /usr
DESTDIR ?=

all: bin js

bin:
	cargo build --release
	cp --remove-destination ./target/release/cantal-agent .

debug-bin:
	cargo build
	cp --remove-destination ./target/debug/cantal-agent .

# ------------------ JAVASCRIPTS -----------------------

js:
	-mkdir public/js 2>/dev/null
	make -C frontend

# ------------------ INSTALL -----------------------

install:
	install -d $(DESTDIR)$(PREFIX)/bin
	install -d $(DESTDIR)$(PREFIX)/lib/cantal
	install -m 755 ./target/release/cantal $(DESTDIR)$(PREFIX)/bin/cantal

	install -m 755 ./target/release/cantal-agent $(DESTDIR)$(PREFIX)/lib/cantal/cantal-agent
	ln -s ../lib/cantal/cantal-agent $(DESTDIR)$(PREFIX)/bin/cantal-agent
	cp -r public $(DESTDIR)$(PREFIX)/lib/cantal/


.PHONY: all install test bin js
.DELETE_ON_ERROR:

RUSTC ?= rustc -C opt-level=3

PREFIX ?= /usr
CONFIGDIR ?= /etc/cantal
DESTDIR ?=
WEBPACK ?= webpack
export CANTAL_VERSION = $(shell git describe)

all: bin cli js

bin:
	cargo build --release
	cp --remove-destination ./target/release/cantal-agent .

cli:
	cd cantal_values; cargo build

cli-release:
	cd cantal_values; cargo build --release

debug-bin:
	cargo build
	cp --remove-destination ./target/debug/cantal-agent .

# ------------------ JAVASCRIPTS -----------------------

js:
	-mkdir public/js 2>/dev/null
	cd web; $(WEBPACK)

js-release:
	-mkdir public/js 2>/dev/null
	cd web; NODE_ENV=production $(WEBPACK) --optimize-minimize

# ------------------ INSTALL -----------------------

install:
	install -d $(DESTDIR)$(PREFIX)/bin
	install -d $(DESTDIR)$(PREFIX)/lib/cantal
	install -d $(DESTDIR)$(CONFIGDIR)
	install -m 755 ./cantal_values/target/release/cantal $(DESTDIR)$(PREFIX)/bin/cantal

	install -m 755 ./target/release/cantal-agent $(DESTDIR)$(PREFIX)/lib/cantal/cantal-agent
	ln -s ../lib/cantal/cantal-agent $(DESTDIR)$(PREFIX)/bin/cantal-agent
	cp -r public $(DESTDIR)$(PREFIX)/lib/cantal/


.PHONY: all install test bin js js-release cli cli-release
.DELETE_ON_ERROR:

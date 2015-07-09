RUSTC ?= rustc -C opt-level=3

PREFIX ?= /usr
DESTDIR ?=

all: bin js

bin:
	cargo build --release
	cp ./target/release/cantal-agent .
	cp ./target/release/cantal .

debug-bin:
	cargo build
	cp ./target/debug/cantal-agent .
	cp ./target/debug/cantal .

# ------------------ JAVASCRIPTS -----------------------

js:
	-mkdir public/js 2>/dev/null
	make -C frontend

# ------------------ INSTALL -----------------------

install:
	install -d $(DESTDIR)$(PREFIX)/bin
	install -d $(DESTDIR)$(PREFIX)/lib/cantal
	install -m 755 cantal $(DESTDIR)$(PREFIX)/bin/cantal

	install -m 755 cantal-agent $(DESTDIR)$(PREFIX)/lib/cantal/cantal-agent
	ln -s ../lib/cantal/cantal-agent $(DESTDIR)$(PREFIX)/bin/cantal-agent
	# setcap is required to be able to read other processes environment
	# without root privileges
	# setcap "cap_sys_ptrace=ep cap_dac_read_search=ep" \
	#	$(DESTDIR)$(PREFIX)/lib/cantal/cantal_agent

	cp -r public $(DESTDIR)$(PREFIX)/lib/cantal/


.PHONY: all install test bin js
.DELETE_ON_ERROR:

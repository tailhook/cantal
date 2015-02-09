RUSTC ?= rustc

PREFIX ?= /usr
DESTDIR ?=

CANTALLIB = libcantal.rlib
ARGPARSELIB = rust-argparse/libargparse.rlib

all: libcantal.rlib cantal cantal_agent

test: cantal_test
	./cantal_test

libcantal.rlib: $(ARGPARSELIB) src/query/lib.rs src/query/*.rs
	$(RUSTC) src/query/lib.rs -g -o $@ \
		-L rust-argparse -L .

cantal_agent: $(ARGPARSELIB) libcantal.rlib src/agent/main.rs src/agent/*.rs
cantal_agent: src/agent/*/*.rs
	$(RUSTC) src/agent/main.rs -g -o $@ \
		-L rust-argparse -L .

cantal: $(ARGPARSELIB) libcantal.rlib src/cli/main.rs src/cli/*.rs
	$(RUSTC) src/cli/main.rs -g -o $@ \
		-L rust-argparse -L .

$(ARGPARSELIB):
	make -C rust-argparse libargparse.rlib

install:
	install -d $(DESTDIR)$(PREFIX)/bin
	install -m 755 lithos_tree $(DESTDIR)$(PREFIX)/bin/cantal
	install -m 755 lithos_tree $(DESTDIR)$(PREFIX)/bin/cantal-agent


.PHONY: all install test

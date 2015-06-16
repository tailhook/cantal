export NODE_PATH=/usr/local/lib/node_modules

JSDIR=../public/js

all: $(JSDIR)/lib.js webpack

$(JSDIR)/lib.js: lib/*.js
	# TODO(tailhook) minify
	cat $^ > $@

webpack:
	webpack

.PHONY: all webpack

.DELETE_ON_ERROR:

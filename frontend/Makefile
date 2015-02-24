export NODE_PATH=/usr/local/lib/node_modules

JSDIR=../public/js

js: $(JSDIR)/lib.js $(JSDIR)/app.js

$(JSDIR)/app.js: util/*.js app/*.js
	# TODO(tailhook) order(?) and minify
	babel -m amd -M util/*.js app/*.js > $@

$(JSDIR)/lib.js: lib/*.js
	# TODO(tailhook) minify
	cat $^ > $@


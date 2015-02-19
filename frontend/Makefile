export NODE_PATH=/usr/local/lib/node_modules

JSDIR=../public/js

js: $(JSDIR)/lib.js $(JSDIR)/riot-tags.js $(JSDIR)/util.js

$(JSDIR)/util.js: util/*.js
	# TODO(tailhook) order(?) and minify
	babel -m amd -M util/*.js > $@

$(JSDIR)/lib.js: lib/*.js
	# TODO(tailhook) minify
	cat $^ > $@

$(JSDIR)/riot-tags.js: tags/*.tag
	riot --type es6 tags $(JSDIR)/riot-tags.js


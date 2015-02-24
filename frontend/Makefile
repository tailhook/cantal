export NODE_PATH=/usr/local/lib/node_modules

JSDIR=../public/js
JS=util/request.js util/html.js util/time.js util/navbar.js util/processes.js

js: $(JSDIR)/lib.js $(JSDIR)/app.js

$(JSDIR)/app.js: $(JS) app/main.js
	# TODO(tailhook) order(?) and minify
	babel -m amd -M $^ > $@

$(JSDIR)/lib.js: lib/*.js
	# TODO(tailhook) minify
	cat $^ > $@

.DELETE_ON_ERROR:

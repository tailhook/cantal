export NODE_PATH=/usr/local/lib/node_modules

JSDIR=../public/js
UTIL_JS= util/base.js util/events.js \
	util/request.js util/html.js util/time.js \
	util/donut.js util/chart.js util/sparkline.js util/plot.js \

APP_JS= \
	util/navbar.js \
	pages/processes.js pages/status.js pages/values.js pages/totals.js

webpack:
	webpack

all: js templates

js: $(JSDIR)/lib.js $(JSDIR)/util.js $(JSDIR)/app.js

$(JSDIR)/util.js: $(UTIL_JS)
	# TODO(tailhook) order(?) and minify
	babel -m amd -M $^ > $@

$(JSDIR)/app.js: $(APP_JS) app/main.js
	# TODO(tailhook) order(?) and minify
	babel -m amd -M $^ > $@

$(JSDIR)/lib.js: lib/*.js
	# TODO(tailhook) minify
	cat $^ > $@

tools/jadecompile: tools/jadecompile.js
	babel $^ --stage 0 > $@

templates: ../marafet templates/*.mft
	node tools/jadecompile templates/*.jade

.PHONY: all js templates webpack

.DELETE_ON_ERROR:

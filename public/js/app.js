/******/ (function(modules) { // webpackBootstrap
/******/ 	// The module cache
/******/ 	var installedModules = {};

/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {

/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId])
/******/ 			return installedModules[moduleId].exports;

/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			exports: {},
/******/ 			id: moduleId,
/******/ 			loaded: false
/******/ 		};

/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);

/******/ 		// Flag the module as loaded
/******/ 		module.loaded = true;

/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}


/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = modules;

/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = installedModules;

/******/ 	// __webpack_public_path__
/******/ 	__webpack_require__.p = "";

/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(0);
/******/ })
/************************************************************************/
/******/ ([
/* 0 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { 'default': obj }; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	var _utilBase = __webpack_require__(1);

	var _utilWebsock = __webpack_require__(3);

	var _utilWebsock2 = _interopRequireDefault(_utilWebsock);

	var _pagesProcesses = __webpack_require__(5);

	var _pagesStatus = __webpack_require__(9);

	var _pagesValues = __webpack_require__(24);

	var _pagesTotals = __webpack_require__(25);

	var _pagesMetrics = __webpack_require__(26);

	var _pagesPeers = __webpack_require__(28);

	var _pagesRemote = __webpack_require__(30);

	var _utilRender = __webpack_require__(2);

	var _utilRouting = __webpack_require__(36);

	var _utilRouting2 = _interopRequireDefault(_utilRouting);

	var _templatesNavbarMft = __webpack_require__(37);

	var _templatesNavbarMft2 = _interopRequireDefault(_templatesNavbarMft);

	var App = (function () {
	    function App() {
	        _classCallCheck(this, App);
	    }

	    _createClass(App, [{
	        key: 'render',
	        value: function render() {
	            return { tag: 'div', children: [_templatesNavbarMft2['default'].render(this.page && this.page.name.toLowerCase()), this.page ? (0, _utilBase.component)(this.page) : ""] };
	        }
	    }, {
	        key: 'change_page',
	        value: function change_page(page) {
	            if (this.page) {
	                this.page = null;
	            }
	            if (page == 'processes') {
	                this.page = _pagesProcesses.Processes;
	            } else if (page == 'status') {
	                this.page = _pagesStatus.Status;
	            } else if (page == 'values') {
	                this.page = _pagesValues.Values;
	            } else if (page == 'totals') {
	                this.page = _pagesTotals.Totals;
	            } else if (page == 'metrics') {
	                this.page = _pagesMetrics.Metrics;
	            } else if (page == 'peers') {
	                this.page = _pagesPeers.Peers;
	            } else if (page == 'remote') {
	                this.page = _pagesRemote.Remote;
	            }
	            (0, _utilRender.update)();
	        }
	    }], [{
	        key: 'start',
	        value: function start() {
	            var app = new App();

	            var router = _utilRouting2['default'].start();
	            router.page_stream.handle(app.change_page.bind(app));
	            app.change_page(router.page);

	            _utilWebsock2['default'].start('ws://' + location.host + '/ws');

	            (0, _utilRender.append)(document.body, app.render.bind(app));
	        }
	    }]);

	    return App;
	})();

	exports.App = App;

	App.start();

/***/ },
/* 1 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	exports.component = component;

	function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { 'default': obj }; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	var _utilRender = __webpack_require__(2);

	var _utilRender2 = _interopRequireDefault(_utilRender);

	var GuardProxy = (function () {
	    function GuardProxy(guard, component) {
	        _classCallCheck(this, GuardProxy);

	        this._guard = guard;
	        this._component = component;
	    }

	    _createClass(GuardProxy, [{
	        key: 'process',
	        value: function process(fun) {
	            var _this = this;

	            this._guard.set_handler(function () {
	                var obj = fun.apply(undefined, arguments);
	                for (var k in obj) {
	                    _this._component[k] = obj[k];
	                }
	                _utilRender2['default'].update();
	            });
	        }
	    }]);

	    return GuardProxy;
	})();

	var Component = (function () {
	    function Component() {
	        _classCallCheck(this, Component);

	        this._guards = {};
	    }

	    _createClass(Component, [{
	        key: 'init',
	        value: function init() {}
	    }, {
	        key: 'guard',
	        value: function guard(name, value) {
	            var old_guard = this._guards[name];
	            if (old_guard) {
	                value = old_guard.replace_with(value);
	                this._guards[name] = value;
	            } else {
	                this._guards[name] = value;
	                value.start();
	            }
	            return new GuardProxy(value, this);
	        }
	    }, {
	        key: 'clear_guard',
	        value: function clear_guard(k) {
	            var g = this._guards[k];
	            if (g) {
	                g.stop();
	            }
	            delete this._guards[k];
	        }
	    }, {
	        key: 'destroy',
	        value: function destroy() {
	            for (var k in this._guards) {
	                this._guards[k].stop();
	            }
	            delete this._guards;
	        }
	    }]);

	    return Component;
	})();

	exports.Component = Component;

	function component(cls) {
	    for (var _len = arguments.length, args = Array(_len > 1 ? _len - 1 : 0), _key = 1; _key < _len; _key++) {
	        args[_key - 1] = arguments[_key];
	    }

	    return function (old_item) {
	        try {
	            if (old_item && old_item.component != null) {
	                if (old_item.component instanceof cls) {
	                    var cmp = old_item.component;
	                    if (cmp.init) {
	                        // TODO(tailhook) optimize init
	                        cmp.init.apply(cmp, args);
	                    }
	                } else {
	                    old_item.component.destroy();
	                    var cmp = new cls();
	                    if (cmp.init) {
	                        cmp.init.apply(cmp, args);
	                    }
	                }
	            } else {
	                var cmp = new cls();
	                if (cmp.init) {
	                    cmp.init.apply(cmp, args);
	                }
	            }
	            var el = cmp.render();
	            while (typeof el == 'function') {
	                el = el(old_item);
	            }
	        } catch (e) {
	            console.error("Rendering error", e, e.stack);
	            return {
	                tag: 'span',
	                attrs: { 'class': 'error' },
	                children: e.toString()
	            };
	        }
	        el.component = cmp;
	        // Todo use add_events from util/events
	        var ev = el.events || (el.events = {});
	        ev['$destroyed'] = function () {
	            cmp.destroy();
	        };
	        return el;
	    };
	}

/***/ },
/* 2 */
/***/ function(module, exports) {

	"use strict";

	Object.defineProperty(exports, "__esModule", {
	    value: true
	});
	exports.append = append;
	exports.update = update;
	var registrations = [];
	var req_id = 0;

	function append(el, fun) {
	    var node = cito.vdom.append(el, fun);
	    registrations.push({
	        node: node,
	        renderer: fun
	    });
	}

	function real_update() {
	    cancelAnimationFrame(req_id);
	    req_id = 0;
	    for (var i = 0, il = registrations.length; i < il; ++i) {
	        var ob = registrations[i];
	        cito.vdom.update(ob.node, ob.renderer);
	    }
	}

	function update() {
	    if (!req_id) {
	        req_id = requestAnimationFrame(real_update);
	    }
	}

	exports["default"] = exports;

/***/ },
/* 3 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _get = function get(_x, _x2, _x3) { var _again = true; _function: while (_again) { var object = _x, property = _x2, receiver = _x3; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x = parent; _x2 = property; _x3 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	exports.start = start;
	exports.send = send;
	exports.remote_enabled = remote_enabled;

	function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { 'default': obj }; }

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	var _utilRender = __webpack_require__(2);

	var _utilRender2 = _interopRequireDefault(_utilRender);

	var _utilProbor = __webpack_require__(4);

	var web_socket;
	var url = url;
	var last_beacon;
	exports.last_beacon = last_beacon;
	var connected;

	exports.connected = connected;

	function start(_url) {
	    url = _url;
	    connect();
	}

	function connect() {
	    web_socket = new WebSocket(url);
	    web_socket.binaryType = "arraybuffer";
	    web_socket.onmessage = message_received;
	    web_socket.onopen = onconnected;
	    web_socket.onclose = ondisconnected;
	}

	var Message = function Message() {
	    _classCallCheck(this, Message);
	};

	var Beacon = (function (_Message) {
	    _inherits(Beacon, _Message);

	    function Beacon(props) {
	        _classCallCheck(this, Beacon);

	        _get(Object.getPrototypeOf(Beacon.prototype), 'constructor', this).call(this);
	        var _iteratorNormalCompletion = true;
	        var _didIteratorError = false;
	        var _iteratorError = undefined;

	        try {
	            for (var _iterator = Object.keys(props)[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
	                var k = _step.value;

	                this[k] = props[k];
	            }
	        } catch (err) {
	            _didIteratorError = true;
	            _iteratorError = err;
	        } finally {
	            try {
	                if (!_iteratorNormalCompletion && _iterator['return']) {
	                    _iterator['return']();
	                }
	            } finally {
	                if (_didIteratorError) {
	                    throw _iteratorError;
	                }
	            }
	        }
	    }

	    return Beacon;
	})(Message);

	Beacon.probor_enum_protocol = [new _utilProbor.Reflect()];

	// We only need beacon for client websockets for now
	Message.probor_protocol = new _utilProbor.Enum({ 0: Beacon });

	function message_received(ev) {
	    if (ev.data.constructor == ArrayBuffer) {
	        var data = (0, _utilProbor.decode)(Message, ev.data);
	        if (data.constructor == Beacon) {
	            var tm = new Date().getTime();
	            var beacon = data;
	            beacon.receive_time = tm;
	            beacon.latency = tm - beacon.current_time;
	            exports.last_beacon = last_beacon = beacon;
	            console.log("Beacon", beacon);
	            _utilRender2['default'].update();
	        }
	    } else {
	        console.error("Spontaneous text data", ev.data);
	    }
	}

	function onconnected(ev) {
	    exports.connected = connected = true;
	}

	function ondisconnected(ev) {
	    exports.connected = connected = false;
	    setTimeout(connect, 1000);
	}

	function send(variant) {
	    for (var _len = arguments.length, args = Array(_len > 1 ? _len - 1 : 0), _key = 1; _key < _len; _key++) {
	        args[_key - 1] = arguments[_key];
	    }

	    web_socket.send(JSON.stringify({ "variant": variant, "fields": args }));
	}

	function remote_enabled() {
	    return last_beacon && last_beacon.remote_total != null;
	}

	exports['default'] = exports;

	window.WEBSOCK_DEBUG_INTERFACE = exports;

/***/ },
/* 4 */
/***/ function(module, exports) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _slicedToArray = (function () { function sliceIterator(arr, i) { var _arr = []; var _n = true; var _d = false; var _e = undefined; try { for (var _i = arr[Symbol.iterator](), _s; !(_n = (_s = _i.next()).done); _n = true) { _arr.push(_s.value); if (i && _arr.length === i) break; } } catch (err) { _d = true; _e = err; } finally { try { if (!_n && _i['return']) _i['return'](); } finally { if (_d) throw _e; } } return _arr; } return function (arr, i) { if (Array.isArray(arr)) { return arr; } else if (Symbol.iterator in Object(arr)) { return sliceIterator(arr, i); } else { throw new TypeError('Invalid attempt to destructure non-iterable instance'); } }; })();

	var _bind = Function.prototype.bind;

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	var _get = function get(_x, _x2, _x3) { var _again = true; _function: while (_again) { var object = _x, property = _x2, receiver = _x3; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x = parent; _x2 = property; _x3 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	exports.decode = decode;

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	var SimpleStruct = function SimpleStruct(obj) {
	    _classCallCheck(this, SimpleStruct);

	    var _iteratorNormalCompletion = true;
	    var _didIteratorError = false;
	    var _iteratorError = undefined;

	    try {
	        for (var _iterator = Object.keys(obj)[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
	            var k = _step.value;

	            this[k] = obj[k];
	        }
	    } catch (err) {
	        _didIteratorError = true;
	        _iteratorError = err;
	    } finally {
	        try {
	            if (!_iteratorNormalCompletion && _iterator['return']) {
	                _iterator['return']();
	            }
	        } finally {
	            if (_didIteratorError) {
	                throw _iteratorError;
	            }
	        }
	    }
	};

	exports.SimpleStruct = SimpleStruct;

	var Proto = function Proto() {
	    _classCallCheck(this, Proto);
	};

	exports.Proto = Proto;

	var Enum = (function (_Proto) {
	    _inherits(Enum, _Proto);

	    function Enum(options) {
	        _classCallCheck(this, Enum);

	        _get(Object.getPrototypeOf(Enum.prototype), 'constructor', this).call(this);
	        this.numeric_options = {};
	        this.named_options = {};
	        var _iteratorNormalCompletion2 = true;
	        var _didIteratorError2 = false;
	        var _iteratorError2 = undefined;

	        try {
	            for (var _iterator2 = Object.keys(options)[Symbol.iterator](), _step2; !(_iteratorNormalCompletion2 = (_step2 = _iterator2.next()).done); _iteratorNormalCompletion2 = true) {
	                var k = _step2.value;

	                var typ = options[k];
	                var n = parseInt(k);
	                if (isNaN(n)) {
	                    this.named_options[k] = typ;
	                } else {
	                    this.numeric_options[n] = typ;
	                    if (typeof typ == 'string') {
	                        this.named_options[typ] = typ;
	                    } else {
	                        this.named_options[typ.name] = typ;
	                    }
	                }
	            }
	        } catch (err) {
	            _didIteratorError2 = true;
	            _iteratorError2 = err;
	        } finally {
	            try {
	                if (!_iteratorNormalCompletion2 && _iterator2['return']) {
	                    _iterator2['return']();
	                }
	            } finally {
	                if (_didIteratorError2) {
	                    throw _iteratorError2;
	                }
	            }
	        }
	    }

	    _createClass(Enum, [{
	        key: 'decode',
	        value: function decode(val, base_type) {
	            if (!Array.isArray(val)) {
	                throw new DecodeError('Enum requires array got ' + val + ' instead');
	            }
	            var typ = this.numeric_options[val[0]] || this.named_options[val[0]];
	            if (typ === undefined) {
	                throw new DecodeError('Unknown enum value ' + val[0] + ' for ' + base_type);
	            } else if (typeof typ == 'string') {
	                return typ;
	            }
	            var args = [];
	            var proto = typ.probor_enum_protocol;
	            if (proto === undefined) {
	                throw new DecodeError("Enum protocol is undefined for " + typ);
	            }
	            for (var i = 0, il = proto.length; i < il; ++i) {
	                args.push(decode_value(proto[i], val[i + 1]));
	            }
	            return new (_bind.apply(typ, [null].concat(args)))();
	        }
	    }]);

	    return Enum;
	})(Proto);

	exports.Enum = Enum;

	var Reflect = (function (_Proto2) {
	    _inherits(Reflect, _Proto2);

	    function Reflect() {
	        _classCallCheck(this, Reflect);

	        _get(Object.getPrototypeOf(Reflect.prototype), 'constructor', this).apply(this, arguments);
	    }

	    _createClass(Reflect, [{
	        key: 'decode',
	        value: function decode(val, typ) {
	            if (typ !== undefined) {
	                return new typ(val);
	            } else {
	                return val;
	            }
	        }
	    }]);

	    return Reflect;
	})(Proto);

	exports.Reflect = Reflect;

	var Dict = (function (_Proto3) {
	    _inherits(Dict, _Proto3);

	    function Dict(key, value) {
	        _classCallCheck(this, Dict);

	        _get(Object.getPrototypeOf(Dict.prototype), 'constructor', this).call(this);
	        this.key = key;
	        this.value = value;
	    }

	    _createClass(Dict, [{
	        key: 'decode',
	        value: function decode(val) {
	            var res = new Map();
	            for (var k in val) {
	                var dk = decode_value(this.key, k);
	                var dv = decode_value(this.value, val[k]);
	                res.set(dk, dv);
	            }
	            return res;
	        }
	    }]);

	    return Dict;
	})(Proto);

	exports.Dict = Dict;

	var List = (function (_Proto4) {
	    _inherits(List, _Proto4);

	    function List(item) {
	        _classCallCheck(this, List);

	        _get(Object.getPrototypeOf(List.prototype), 'constructor', this).call(this);
	        this.item = item;
	    }

	    _createClass(List, [{
	        key: 'decode',
	        value: function decode(val) {
	            var res = [];
	            var _iteratorNormalCompletion3 = true;
	            var _didIteratorError3 = false;
	            var _iteratorError3 = undefined;

	            try {
	                for (var _iterator3 = val[Symbol.iterator](), _step3; !(_iteratorNormalCompletion3 = (_step3 = _iterator3.next()).done); _iteratorNormalCompletion3 = true) {
	                    var item = _step3.value;

	                    res.push(decode_value(this.item, item));
	                }
	            } catch (err) {
	                _didIteratorError3 = true;
	                _iteratorError3 = err;
	            } finally {
	                try {
	                    if (!_iteratorNormalCompletion3 && _iterator3['return']) {
	                        _iterator3['return']();
	                    }
	                } finally {
	                    if (_didIteratorError3) {
	                        throw _iteratorError3;
	                    }
	                }
	            }

	            return res;
	        }
	    }]);

	    return List;
	})(Proto);

	exports.List = List;

	var Str = (function (_Proto5) {
	    _inherits(Str, _Proto5);

	    function Str() {
	        _classCallCheck(this, Str);

	        _get(Object.getPrototypeOf(Str.prototype), 'constructor', this).apply(this, arguments);
	    }

	    _createClass(Str, [{
	        key: 'decode',
	        value: function decode(val) {
	            if (typeof val != 'string') throw new DecodeError('String expected got ' + val);
	            return val;
	        }
	    }]);

	    return Str;
	})(Proto);

	exports.Str = Str;

	var Int = (function (_Proto6) {
	    _inherits(Int, _Proto6);

	    function Int() {
	        _classCallCheck(this, Int);

	        _get(Object.getPrototypeOf(Int.prototype), 'constructor', this).apply(this, arguments);
	    }

	    _createClass(Int, [{
	        key: 'decode',
	        value: function decode(val) {
	            if (typeof val != 'number') throw new DecodeError('Integer expected got ' + val);
	            // TODO(tailhook) should we check for mantissa
	            return val;
	        }
	    }]);

	    return Int;
	})(Proto);

	exports.Int = Int;

	var Float = (function (_Proto7) {
	    _inherits(Float, _Proto7);

	    function Float() {
	        _classCallCheck(this, Float);

	        _get(Object.getPrototypeOf(Float.prototype), 'constructor', this).apply(this, arguments);
	    }

	    _createClass(Float, [{
	        key: 'decode',
	        value: function decode(val) {
	            if (typeof val != 'number') throw new DecodeError('Float expected got ' + val);
	            return val;
	        }
	    }]);

	    return Float;
	})(Proto);

	exports.Float = Float;

	var Tuple = (function (_Proto8) {
	    _inherits(Tuple, _Proto8);

	    function Tuple() {
	        _classCallCheck(this, Tuple);

	        _get(Object.getPrototypeOf(Tuple.prototype), 'constructor', this).call(this);

	        for (var _len = arguments.length, items = Array(_len), _key = 0; _key < _len; _key++) {
	            items[_key] = arguments[_key];
	        }

	        this.items = items;
	    }

	    _createClass(Tuple, [{
	        key: 'decode',
	        value: function decode(val) {
	            if (val.length != this.items.length) throw new DecodeError("Tuple of wrong length");
	            var res = [];
	            for (var i = 0; i < val.length; ++i) {
	                res.push(decode_value(this.items[i], val[i]));
	            }
	            return res;
	        }
	    }]);

	    return Tuple;
	})(Proto);

	exports.Tuple = Tuple;

	var Struct = (function (_Proto9) {
	    _inherits(Struct, _Proto9);

	    function Struct(fields) {
	        _classCallCheck(this, Struct);

	        _get(Object.getPrototypeOf(Struct.prototype), 'constructor', this).call(this);
	        this.fields = fields;
	    }

	    _createClass(Struct, [{
	        key: 'decode',
	        value: function decode(val, typ) {
	            var props = {};
	            // TODO(tailhook) check for required fields?
	            if (Array.isArray(val)) {
	                var _iteratorNormalCompletion4 = true;
	                var _didIteratorError4 = false;
	                var _iteratorError4 = undefined;

	                try {
	                    for (var _iterator4 = this.fields[Symbol.iterator](), _step4; !(_iteratorNormalCompletion4 = (_step4 = _iterator4.next()).done); _iteratorNormalCompletion4 = true) {
	                        var _step4$value = _slicedToArray(_step4.value, 3);

	                        var k = _step4$value[0];
	                        var n = _step4$value[1];
	                        var v = _step4$value[2];

	                        var curv = val[n];
	                        if (curv) {
	                            props[k] = decode_value(v, curv);
	                        }
	                    }
	                } catch (err) {
	                    _didIteratorError4 = true;
	                    _iteratorError4 = err;
	                } finally {
	                    try {
	                        if (!_iteratorNormalCompletion4 && _iterator4['return']) {
	                            _iterator4['return']();
	                        }
	                    } finally {
	                        if (_didIteratorError4) {
	                            throw _iteratorError4;
	                        }
	                    }
	                }
	            } else {
	                var _iteratorNormalCompletion5 = true;
	                var _didIteratorError5 = false;
	                var _iteratorError5 = undefined;

	                try {
	                    for (var _iterator5 = this.fields[Symbol.iterator](), _step5; !(_iteratorNormalCompletion5 = (_step5 = _iterator5.next()).done); _iteratorNormalCompletion5 = true) {
	                        var _step5$value = _slicedToArray(_step5.value, 3);

	                        var k = _step5$value[0];
	                        var n = _step5$value[1];
	                        var v = _step5$value[2];

	                        if (n in val) {
	                            props[k] = decode_value(v, val[n]);
	                        } else if (k in val) {
	                            props[k] = decode_value(v, val[k]);
	                        }
	                    }
	                } catch (err) {
	                    _didIteratorError5 = true;
	                    _iteratorError5 = err;
	                } finally {
	                    try {
	                        if (!_iteratorNormalCompletion5 && _iterator5['return']) {
	                            _iterator5['return']();
	                        }
	                    } finally {
	                        if (_didIteratorError5) {
	                            throw _iteratorError5;
	                        }
	                    }
	                }
	            }
	            return new typ(props);
	        }
	    }]);

	    return Struct;
	})(Proto);

	exports.Struct = Struct;

	var Optional = (function (_Proto10) {
	    _inherits(Optional, _Proto10);

	    function Optional(typ) {
	        _classCallCheck(this, Optional);

	        _get(Object.getPrototypeOf(Optional.prototype), 'constructor', this).call(this);
	        this.type = typ;
	    }

	    _createClass(Optional, [{
	        key: 'decode',
	        value: function decode(val, typ) {
	            if (val == null) {
	                return null;
	            } else {
	                return decode_value(this.type, val);
	            }
	        }
	    }]);

	    return Optional;
	})(Proto);

	exports.Optional = Optional;

	var DecodeError = (function (_Error) {
	    _inherits(DecodeError, _Error);

	    function DecodeError() {
	        _classCallCheck(this, DecodeError);

	        _get(Object.getPrototypeOf(DecodeError.prototype), 'constructor', this).call(this);
	        var e = Error.apply(undefined, arguments);
	        e.name = this.constructor.name;
	        this.message = e.message;
	        this.stack = e.stack;
	    }

	    return DecodeError;
	})(Error);

	exports.DecodeError = DecodeError;

	function decode(typ, buf) {
	    var val = CBOR.decode(buf);
	    return decode_value(typ, val);
	}

	function decode_value(typ, val) {
	    //console.log("TYPE", typ, val)
	    try {
	        if (typ instanceof Proto) {
	            return typ.decode(val);
	        } else {
	            return typ.probor_protocol.decode(val, typ);
	        }
	    } finally {
	        //console.log("ENDTYPE", typ, val)
	    }
	}

/***/ },
/* 5 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	var _get = function get(_x, _x2, _x3) { var _again = true; _function: while (_again) { var object = _x, property = _x2, receiver = _x3; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x = parent; _x2 = property; _x3 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	var _utilHtml = __webpack_require__(6);

	var _utilTime = __webpack_require__(7);

	var _utilRequest = __webpack_require__(8);

	var _utilBase = __webpack_require__(1);

	var Processes = (function (_Component) {
	    _inherits(Processes, _Component);

	    function Processes() {
	        _classCallCheck(this, Processes);

	        _get(Object.getPrototypeOf(Processes.prototype), 'constructor', this).call(this);
	        this.open_items = {};
	    }

	    _createClass(Processes, [{
	        key: 'init',
	        value: function init() {
	            var _this = this;

	            this.guard('json', new _utilRequest.RefreshJson("/all_processes.json", 5000)).process(function (data, latency) {
	                if (data instanceof Error) {
	                    return { error: data, latency: latency };
	                } else {
	                    var res = _this.build_tree(data);
	                    res['error'] = null;
	                    res['latency'] = latency;
	                    return res;
	                }
	            });
	        }
	    }, {
	        key: 'build_tree',
	        value: function build_tree(data) {
	            var toplevel = [];
	            var by_id = {};
	            var tree = {};
	            var old_open = this.open_items;
	            var new_open = {}; // only existing pids
	            var _iteratorNormalCompletion = true;
	            var _didIteratorError = false;
	            var _iteratorError = undefined;

	            try {
	                for (var _iterator = data.all[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
	                    var p = _step.value;

	                    by_id[p.pid] = p;
	                    var lst = tree[p.ppid];
	                    if (lst === undefined) {
	                        lst = tree[p.ppid] = [];
	                    }
	                    lst.push(p);
	                    if (p.ppid == 1) {
	                        toplevel.push(p);
	                    }
	                    if (p.pid in old_open) {
	                        new_open[p.pid] = true;
	                    }
	                }
	            } catch (err) {
	                _didIteratorError = true;
	                _iteratorError = err;
	            } finally {
	                try {
	                    if (!_iteratorNormalCompletion && _iterator['return']) {
	                        _iterator['return']();
	                    }
	                } finally {
	                    if (_didIteratorError) {
	                        throw _iteratorError;
	                    }
	                }
	            }

	            return {
	                all: data.all,
	                uptime_base: data.boot_time,
	                open_items: new_open,
	                toplevel: toplevel,
	                tree: tree
	            };
	        }
	    }, {
	        key: 'render_process',
	        value: function render_process(level, process) {
	            var _this2 = this;

	            if (level === undefined) level = 0;

	            var children = this.tree[process.pid];
	            var is_open = this.open_items[process.pid];
	            var head = (0, _utilHtml.tag_key)("tr", process.pid, [(0, _utilHtml.td_left)([{ tag: 'div', attrs: {
	                    style: { display: 'inline-block', width: 16 * level + 'px' } } }, children ? is_open ? (0, _utilHtml.button_xs)("default", [(0, _utilHtml.icon)("minus"), ' ' + children.length], function () {
	                delete _this2.open_items[process.pid];
	                _this2.update();
	            }) : (0, _utilHtml.button_xs)("default", [(0, _utilHtml.icon)("plus"), ' ' + children.length], function () {
	                _this2.open_items[process.pid] = true;
	                _this2.update();
	            }) : "", ' ' + process.pid.toString()]), (0, _utilHtml.td_left)((0, _utilHtml.title_span)(process.cmdline.split('\u0000').join(' '), [process.name.toString()])), (0, _utilHtml.td_left)((0, _utilTime.format_uptime)((0, _utilTime.till_now_ms)((0, _utilTime.from_ms)(process.start_time + this.uptime_base * 1000)))), (0, _utilHtml.td_right)((process.rss / 1048576).toFixed(1))]);
	            if (children && this.open_items[process.pid]) {
	                var ch = children.map(this.render_process.bind(this, level + 1));
	                ch.splice(0, 0, head);
	                return { children: ch };
	            } else {
	                return head;
	            }
	        }
	    }, {
	        key: 'render_processes',
	        value: function render_processes() {
	            return (0, _utilHtml.tag_class)("table", "table table-hover", [(0, _utilHtml.tag)("thead", (0, _utilHtml.tag)("tr", [(0, _utilHtml.th_left)('pid'), (0, _utilHtml.th_left)('name'), (0, _utilHtml.th_left)('uptime'), (0, _utilHtml.th_right)('mem (MiB)')])), (0, _utilHtml.tag)("tbody", this.toplevel.map(this.render_process.bind(this, 0)))]);
	        }
	    }, {
	        key: 'render',
	        value: function render() {
	            return (0, _utilHtml.tag_class)("div", "container", [(0, _utilHtml.tag)("h1", ["All Processes"]), (0, _utilHtml.tag_class)("div", "text-right", this.error ? 'Error: ' + this.error.message : 'Fetched in ' + this.latency + 'ms'), this.all ? this.render_processes() : ""]);
	        }
	    }]);

	    return Processes;
	})(_utilBase.Component);

	exports.Processes = Processes;

/***/ },
/* 6 */
/***/ function(module, exports) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});
	exports.tag = tag;
	exports.tag_class = tag_class;
	exports.link = link;
	exports.icon = icon;
	exports.title_span = title_span;
	exports.tag_key = tag_key;
	exports.tag_map = tag_map;
	exports.button_xs = button_xs;
	exports.td_left = td_left;
	exports.td_right = td_right;
	exports.th_left = th_left;
	exports.th_right = th_right;

	function tag(tag, children) {
	    return { tag: tag, children: children };
	}

	function tag_class(tag, classname, children) {
	    return { tag: tag, attrs: { 'class': classname }, children: children };
	}

	function link(classname, href) {
	    for (var _len = arguments.length, args = Array(_len > 2 ? _len - 2 : 0), _key = 2; _key < _len; _key++) {
	        args[_key - 2] = arguments[_key];
	    }

	    return { tag: 'a', attrs: {
	            'class': classname,
	            href: href
	        }, children: args };
	}

	function icon(icon) {
	    return { tag: 'span', attrs: { 'class': 'glyphicon glyphicon-' + icon } };
	}

	function title_span(title, children) {
	    return { tag: 'span', attrs: {
	            title: title,
	            'class': "title"
	        }, children: children };
	}

	function tag_key(tag, key, children) {
	    return { tag: tag, key: key, children: children };
	}

	function tag_map(tagname) {
	    return function (list) {
	        return list.map(tag.bind(null, tagname));
	    };
	}

	function button_xs(kind, children, handler) {
	    return { tag: 'button',
	        attrs: { 'class': 'btn btn-xs btn-' + kind },
	        events: { click: handler },
	        children: children };
	}

	function td_left(value) {
	    return tag_class('td', 'text-left', value);
	}

	function td_right(value) {
	    return tag_class('td', 'text-right', value);
	}

	function th_left(value) {
	    return tag_class('th', 'text-left', value);
	}

	function th_right(value) {
	    return tag_class('th', 'text-right', value);
	}

/***/ },
/* 7 */
/***/ function(module, exports) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});
	exports.from_ms = from_ms;
	exports.till_now_ms = till_now_ms;
	exports.format_datetime = format_datetime;
	exports.format_time = format_time;
	exports.format_uptime = format_uptime;
	exports.format_diff = format_diff;

	function from_ms(ms) {
	    var date = new Date();
	    date.setTime(ms);
	    return date;
	}

	function till_now_ms(dt) {
	    var ms = new Date() - dt.getTime();
	    return ms;
	}

	function _two(n) {
	    if (n < 10) {
	        return '0' + n;
	    }
	    return '' + n;
	}

	function format_datetime(dt) {
	    return dt.getFullYear() + '-' + _two(dt.getMonth()) + '-' + _two(dt.getDate()) + (' ' + _two(dt.getHours()) + ':' + _two(dt.getMinutes())) + (':' + _two(dt.getSeconds()));
	}

	function format_time(dt) {
	    return _two(dt.getHours()) + ':' + _two(dt.getMinutes());
	}

	function format_uptime(ms) {
	    if (ms < 1000) {
	        return "∅";
	    } else if (ms < 90000) {
	        return (ms / 1000 | 0) + 's';
	    } else if (ms < 5400000) {
	        return (ms / 60000 | 0) + 'm' + (ms / 1000 % 60 | 0) + 's';
	    } else if (ms < 86400000) {
	        return (ms / 3600000 | 0) + 'h' + (ms / 60000 % 60 | 0) + 'm';
	    } else {
	        return (ms / 86400000 | 0) + 'd' + (ms / 3600000 % 24 | 0) + 'h';
	    }
	}

	function format_diff(ms) {
	    if (ms < 1000) {
	        return ms + 'ms';
	    } else if (ms < 90000) {
	        return (ms / 1000 | 0) + 's';
	    } else if (ms < 5400000) {
	        return (ms / 60000 | 0) + 'm' + (ms / 1000 % 60 | 0) + 's';
	    } else if (ms < 86400000) {
	        return (ms / 3600000 | 0) + 'h' + (ms / 60000 % 60 | 0) + 'm';
	    } else {
	        return (ms / 86400000 | 0) + 'd' + (ms / 3600000 % 24 | 0) + 'h';
	    }
	}

/***/ },
/* 8 */
/***/ function(module, exports) {

	"use strict";

	Object.defineProperty(exports, "__esModule", {
	    value: true
	});

	var _get = function get(_x2, _x3, _x4) { var _again = true; _function: while (_again) { var object = _x2, property = _x3, receiver = _x4; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x2 = parent; _x3 = property; _x4 = receiver; _again = true; continue _function; } } else if ("value" in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	function _inherits(subClass, superClass) { if (typeof superClass !== "function" && superClass !== null) { throw new TypeError("Super expression must either be null or a function, not " + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

	var RefreshJson = (function () {
	    function RefreshJson(url) {
	        var options = arguments.length <= 1 || arguments[1] === undefined ? {} : arguments[1];

	        _classCallCheck(this, RefreshJson);

	        this.url = url;
	        this.interval = options.interval || 2000;
	        this.post_body = options.post_body || null;
	    }

	    _createClass(RefreshJson, [{
	        key: "set_handler",
	        value: function set_handler(fun) {
	            this.handler = fun;
	        }
	    }, {
	        key: "start",
	        value: function start() {
	            var _this = this;

	            if (this._timer) {
	                clearInterval(this._timer);
	            }
	            this._timer = setInterval(function () {
	                return _this.refresh_now();
	            }, this.interval);
	            this.refresh_now();
	        }
	    }, {
	        key: "stop",
	        value: function stop() {
	            if (this._req) {
	                this._req.abort();
	                this._req = null;
	            }
	            if (this._timer) {
	                clearInterval(this._timer);
	                this._timer = 0;
	            }
	        }
	    }, {
	        key: "replace_with",
	        value: function replace_with(other) {
	            if (other.url != this.url || other.interval != this.interval || other.post_body != this.post_body) {
	                this.stop();
	                other.start();
	                return other;
	            } else {
	                return this;
	            }
	        }
	    }, {
	        key: "refresh_now",
	        value: function refresh_now() {
	            var _this2 = this;

	            if (this._req) {
	                this._req.onreadystatechange = null;
	                this._req.abort();
	            }
	            var req = this._req = new XMLHttpRequest();
	            var time = new Date();
	            req.onreadystatechange = function (ev) {
	                if (req.readyState < 4) {
	                    return;
	                }
	                var lcy = new Date() - time;
	                if (req.status != 200) {
	                    console.error("Error fetching", _this2.url, req);
	                    _this2.handler(Error("Status " + req.status), lcy);
	                    return;
	                }
	                try {
	                    var json = JSON.parse(req.responseText);
	                } catch (e) {
	                    console.error("Error parsing json at", _this2.url, e);
	                    _this2.handler(Error("Bad Json"), lcy);
	                    return;
	                }
	                if (!json || typeof json != "object") {
	                    console.error("Returned json is not an object", _this2.url, req);
	                    _this2.handler(Error("Bad Json"), lcy);
	                    return;
	                }
	                _this2.handler(json, lcy);
	            };
	            if (this.post_body) {
	                req.open('POST', this.url, true);
	                req.send(this.post_body);
	            } else {
	                req.open('GET', this.url, true);
	                req.send();
	            }
	        }
	    }]);

	    return RefreshJson;
	})();

	exports.RefreshJson = RefreshJson;

	var HTTPError = (function (_Error) {
	    _inherits(HTTPError, _Error);

	    function HTTPError(req) {
	        _classCallCheck(this, HTTPError);

	        _get(Object.getPrototypeOf(HTTPError.prototype), "constructor", this).call(this, "HTTP Error: " + req.status);
	        this.status = req.status;
	        this.status_text = req.statusText;
	        this.text = req.responseText;
	    }

	    _createClass(HTTPError, [{
	        key: "toString",
	        value: function toString() {
	            if (this.status == 400) {
	                return "Error: " + this.text;
	            } else {
	                return "HTTP Error: " + this.status + " " + this.status_text;
	            }
	        }
	    }]);

	    return HTTPError;
	})(Error);

	exports.HTTPError = HTTPError;

	var Submit = (function () {
	    function Submit(url, data) {
	        _classCallCheck(this, Submit);

	        this.url = url;
	        this.data = JSON.stringify(data);
	    }

	    _createClass(Submit, [{
	        key: "set_handler",
	        value: function set_handler(fun) {
	            this.handler = fun;
	        }
	    }, {
	        key: "stop",
	        value: function stop() {
	            if (this._req) {
	                this._req.abort();
	                this._req = null;
	            }
	        }
	    }, {
	        key: "replace_with",
	        value: function replace_with(other) {
	            if (this.url != other.url || this.data != other.data || !this._req) {
	                this.stop();
	                other.start();
	            }
	            return other;
	        }
	    }, {
	        key: "start",
	        value: function start() {
	            var _this3 = this;

	            if (this._req) {
	                this._req.abort();
	            }
	            var req = this._req = new XMLHttpRequest();
	            var time = new Date();
	            req.onreadystatechange = function (ev) {
	                _this3._req = null;
	                if (req.readyState < 4) {
	                    return;
	                }
	                var lcy = new Date() - time;
	                if (req.status != 200) {
	                    console.error("Error fetching", _this3.url, req);
	                    _this3.handler(new HTTPError(req), lcy);
	                    return;
	                }
	                try {
	                    var json = JSON.parse(req.responseText);
	                } catch (e) {
	                    console.error("Error parsing json at", _this3.url, e);
	                    _this3.handler(Error("Bad Json"), lcy);
	                    return;
	                }
	                if (!json || typeof json != "object") {
	                    console.error("Returned json is not an object", _this3.url, req);
	                    _this3.handler(Error("Bad Json"), lcy);
	                    return;
	                }
	                _this3.handler(json, lcy);
	            };
	            req.open('POST', this.url, true);
	            req.send(this.data);
	        }
	    }]);

	    return Submit;
	})();

	exports.Submit = Submit;

/***/ },
/* 9 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	var _get = function get(_x, _x2, _x3) { var _again = true; _function: while (_again) { var object = _x, property = _x2, receiver = _x3; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x = parent; _x2 = property; _x3 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { 'default': obj }; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	var _utilTime = __webpack_require__(7);

	var _utilBase = __webpack_require__(1);

	var _utilEvents = __webpack_require__(10);

	var _utilPlot = __webpack_require__(11);

	var _utilRequest = __webpack_require__(8);

	var _templatesStatusMft = __webpack_require__(12);

	var _templatesStatusMft2 = _interopRequireDefault(_templatesStatusMft);

	var _utilCompute = __webpack_require__(23);

	var Status = (function (_Component) {
	    _inherits(Status, _Component);

	    function Status() {
	        _classCallCheck(this, Status);

	        _get(Object.getPrototypeOf(Status.prototype), 'constructor', this).call(this);
	    }

	    _createClass(Status, [{
	        key: 'init',
	        value: function init(elem) {}
	    }, {
	        key: 'render',
	        value: function render() {
	            return _templatesStatusMft2['default'].render();
	        }
	    }]);

	    return Status;
	})(_utilBase.Component);

	exports.Status = Status;

/***/ },
/* 10 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});
	exports.toggle = toggle;

	var _utilBase = __webpack_require__(1);

	function toggle(object, property) {
	    var ctx = _utilBase.Context.current();
	    return function (ev) {
	        object[property] = !object[property];
	        ev.preventDefault();
	        ctx.refresh(object);
	    };
	}

/***/ },
/* 11 */
/***/ function(module, exports) {

	"use strict";

	Object.defineProperty(exports, "__esModule", {
	    value: true
	});

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

	function linear_axis(min, max, nticks) {
	    console.assert(min == 0);
	    var diff = max - min;
	    var tick_delta = diff / nticks;
	    var num_decimals = -Math.floor(Math.log10(tick_delta));
	    var magnitude = Math.pow(10, -num_decimals);

	    var norm = tick_delta / magnitude;
	    var k = undefined;
	    if (norm < 1.5) {
	        k = 1;
	    } else if (norm < 3) {
	        k = 2;
	        if (norm > 2.25) {
	            k = 2.5;
	            num_decimals += 1;
	        }
	    } else if (norm < 7.5) {
	        k = 5;
	    } else {
	        k = 10;
	    }
	    var tick_size = k * magnitude;
	    var tick_start = tick_size * Math.floor(min / tick_size);
	    var num_ticks = Math.ceil((max - tick_start) / tick_size);
	    var height = tick_size * num_ticks;

	    var ticks = [];
	    for (var i = 0; i < num_ticks; ++i) {
	        var value = tick_start + tick_size * i;
	        var label = undefined;
	        if (num_decimals > 0) {
	            label = value.toFixed(num_decimals);
	        } else if (num_decimals <= -6) {
	            label = (value / 1000000).toFixed(0) + "M";
	        } else if (num_decimals <= -3) {
	            label = (value / 1000).toFixed(0) + "k";
	        } else {
	            label = value.toFixed(0);
	        }
	        ticks[i] = { value: value, label: label };
	    }

	    return {
	        min: min, tick_size: tick_size, height: height, tick_start: tick_start, num_ticks: num_ticks, ticks: ticks,
	        max: min + height
	    };
	}

	function time_axis(ts) {
	    var min = ts[0];
	    var max = ts[ts.length - 1];
	    var diff = min - max;
	}

	var Plot = function Plot(ts, data, width, height) {
	    _classCallCheck(this, Plot);

	    var xoff = ts[0].getTime();
	    var max = this.max = Math.max.apply(null, data);
	    var min = this.min = Math.min.apply(null, data);
	    var yaxis = this.yaxis = linear_axis(0, max, 0.3 * Math.sqrt(height));
	    var xaxis = this.xaxis = time_axis(ts);
	    var xscale = width / (xoff - ts[data.length - 1].getTime());
	    var yscale = height / yaxis.height;
	    var path = "M " + width + ", " + (height - data[0] * yscale) + " L";
	    for (var i = 1, il = data.length; i < il; ++i) {
	        path += " " + (width - (xoff - ts[i].getTime()) * xscale) + "\n                      " + (height - data[i] * yscale);
	    }
	    this.xscale = xscale;
	    this.yscale = yscale;
	    this.path = path;
	};

	exports.Plot = Plot;

/***/ },
/* 12 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(1),
	        __webpack_require__(15),
	        __webpack_require__(17),
	        __webpack_require__(19),
	        __webpack_require__(13),
	        __webpack_require__(20),
	        __webpack_require__(21),
	        __webpack_require__(22),
	        __webpack_require__(23),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_base, _mod_util_stores, donut, plot, compact, _mod_util_list, _mod_util_format, _mod_util_query, _mod_util_compute) {
	        var component = _mod_util_base.component;
	        var Toggle = _mod_util_stores.Toggle;
	        var last = _mod_util_list.last;
	        var integral_formatter = _mod_util_format.integral_formatter;
	        var bytes_formatter = _mod_util_format.bytes_formatter;
	        var number_formatter = _mod_util_format.number_formatter;
	        var already_percent_formatter = _mod_util_format.already_percent_formatter;
	        var Query = _mod_util_query.Query;
	        var cpu_usage = _mod_util_compute.cpu_usage;
	        var mem_chart = _mod_util_compute.mem_chart;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(".b-status.sample {\n    display: inline-block;\n    width: 1em;\n    height: 1em;\n}\n\n"))
	        document.head.appendChild(_style)
	        function mem(mem) {
	            return {
	                    key: "b-status:mem",
	                    children: [
	                        {
	                            tag: "h2",
	                            children: "Memory",
	                        },
	                        {
	                            tag: "div",
	                            attrs: {class: "b-status row"},
	                            children: {children: [
	                                {
	                                    tag: "div",
	                                    attrs: {
	                                        style: {margin: "48px 24px"},
	                                        class: "b-status col-xs-4",
	                                    },
	                                    children: donut.render(mem.items, 256, 256, mem.total),
	                                },
	                                {
	                                    tag: "div",
	                                    attrs: {class: "b-status col-xs-4"},
	                                    children: function (old_node) {
	                                        var toggle = old_node && old_node.store_toggle || new Toggle();
	                                        return {
	                                                tag: "table",
	                                                store_toggle: toggle,
	                                                attrs: {class: "b-status table table-condensed table-hover"},
	                                                children: {children: [
	                                                    {
	                                                        tag: "thead",
	                                                        children: {
	                                                            tag: "tr",
	                                                            children: {children: [
	                                                                {tag: "th"},
	                                                                {
	                                                                    tag: "th",
	                                                                    children: "Title",
	                                                                },
	                                                                {
	                                                                    tag: "th",
	                                                                    attrs: {class: "b-status text-right"},
	                                                                    children: "MiB",
	                                                                },
	                                                            ]},
	                                                        },
	                                                    },
	                                                    {
	                                                        tag: "tbody",
	                                                        children: mem.items.map(function (item) {
	                                                            return ((toggle.visible || !item.collapsed)?({
	                                                                    tag: "tr",
	                                                                    children: {children: [
	                                                                        {
	                                                                            tag: "td",
	                                                                            children: ((item.color)?({
	                                                                                tag: "span",
	                                                                                attrs: {
	                                                                                    style: "background-color: " + String(item.color),
	                                                                                    class: "b-status sample",
	                                                                                },
	                                                                            }):("")),
	                                                                        },
	                                                                        {
	                                                                            tag: "td",
	                                                                            children: String(item.title),
	                                                                        },
	                                                                        {
	                                                                            tag: "td",
	                                                                            attrs: {class: "b-status text-right"},
	                                                                            children: item.text,
	                                                                        },
	                                                                    ]},
	                                                                    key: item + ":0",
	                                                                }):({
	                                                                    tag: "tr",
	                                                                    key: item + ":1",
	                                                                }));
	                                                        }),
	                                                    },
	                                                    {
	                                                        tag: "tfoot",
	                                                        children: {
	                                                            tag: "tr",
	                                                            children: {children: [
	                                                                {tag: "td"},
	                                                                {
	                                                                    tag: "td",
	                                                                    attrs: {class: "b-status text-center"},
	                                                                    children: {
	                                                                        tag: "button",
	                                                                        attrs: {class: "b-status btn btn-default btn-xs"},
	                                                                        children: ((toggle.visible)?({
	                                                                            tag: "span",
	                                                                            attrs: {class: "b-status glyphicon glyphicon-chevron-up"},
	                                                                        }):({
	                                                                            tag: "span",
	                                                                            attrs: {class: "b-status glyphicon glyphicon-chevron-down"},
	                                                                        })),
	                                                                        events: {click: toggle.toggle.handle_event},
	                                                                    },
	                                                                },
	                                                                {tag: "td"},
	                                                            ]},
	                                                        },
	                                                    },
	                                                ]},
	                                                events: {"$destroyed": ((toggle.owner_destroyed)?(toggle.owner_destroyed.handle_event):(function () {
	                                                }))},
	                                            };
	                                    },
	                                },
	                            ]},
	                        },
	                    ],
	                };
	        }
	        function cpu(cpu, timestamps) {
	            return {
	                    key: "b-status:cpu",
	                    children: function (old_node) {
	                        var cpu_yaxis = {
	                                height: 40,
	                                bg_color: "rgb(237,248,233)",
	                                skip_color: "white",
	                                format: already_percent_formatter(),
	                                colors: [
	                                    [
	                                        100,
	                                        "rgb(186,228,179)",
	                                    ],
	                                    [
	                                        200,
	                                        "rgb(116,196,118)",
	                                    ],
	                                    [
	                                        800,
	                                        "rgb(49,163,84)",
	                                    ],
	                                    [
	                                        1600,
	                                        "rgb(0,109,44)",
	                                    ],
	                                    [
	                                        6400,
	                                        "black",
	                                    ],
	                                ],
	                            };
	                        return {
	                                tag: "div",
	                                children: {children: [
	                                    {
	                                        tag: "h2",
	                                        children: "CPU",
	                                    },
	                                    compact.render(1100, timestamps, [
	                                        {
	                                            title: "Cpu",
	                                            values: cpu_usage(cpu.TOTAL, cpu.idle),
	                                            yaxis: cpu_yaxis,
	                                        },
	                                        {
	                                            title: "User",
	                                            values: cpu.user.values,
	                                            yaxis: cpu_yaxis,
	                                        },
	                                        {
	                                            title: "System",
	                                            values: cpu.system.values,
	                                            yaxis: cpu_yaxis,
	                                        },
	                                        {
	                                            title: "I/O Wait",
	                                            values: cpu.iowait.values,
	                                            yaxis: cpu_yaxis,
	                                        },
	                                        {
	                                            title: "IRQ",
	                                            values: cpu.irq.values,
	                                            yaxis: cpu_yaxis,
	                                        },
	                                    ]),
	                                ]},
	                            };
	                    },
	                };
	        }
	        function net(net, timestamps) {
	            return {
	                    key: "b-status:net",
	                    children: function (old_node) {
	                        var net_yaxis = {
	                                height: 40,
	                                bg_color: "rgb(237,248,233)",
	                                skip_color: "white",
	                                format: bytes_formatter(),
	                                colors: [
	                                    [
	                                        1024 * 1024 / 8,
	                                        "rgb(186,228,179)",
	                                    ],
	                                    [
	                                        100 * 1024 * 1024 / 8,
	                                        "rgb(116,196,118)",
	                                    ],
	                                    [
	                                        1024 * 1024 * 1024 / 8,
	                                        "rgb(49,163,84)",
	                                    ],
	                                    [
	                                        10 * 1024 * 1024 * 1024 / 8,
	                                        "rgb(0,109,44)",
	                                    ],
	                                    [
	                                        1024 * 1024 * 1024 * 1024,
	                                        "black",
	                                    ],
	                                ],
	                            };
	                        return {
	                                tag: "div",
	                                children: {children: [
	                                    {
	                                        tag: "h2",
	                                        children: "Network",
	                                    },
	                                    compact.render(1100, timestamps, [
	                                        {
	                                            title: "Receive",
	                                            values: net["rx.bytes"].values,
	                                            yaxis: net_yaxis,
	                                        },
	                                        {
	                                            title: "Transfer",
	                                            values: net["tx.bytes"].values,
	                                            yaxis: net_yaxis,
	                                        },
	                                    ]),
	                                ]},
	                            };
	                    },
	                };
	        }
	        function disk(disk, disk_in_progress, timestamps) {
	            return {
	                    key: "b-status:disk",
	                    children: function (old_node) {
	                        var bytes_yaxis = {
	                                height: 40,
	                                bg_color: "rgb(237,248,233)",
	                                skip_color: "white",
	                                format: bytes_formatter(),
	                                colors: [
	                                    [
	                                        1024,
	                                        "rgb(186,228,179)",
	                                    ],
	                                    [
	                                        100 * 1024,
	                                        "rgb(116,196,118)",
	                                    ],
	                                    [
	                                        1024 * 1024,
	                                        "rgb(49,163,84)",
	                                    ],
	                                    [
	                                        1024 * 1024 * 1024,
	                                        "rgb(0,109,44)",
	                                    ],
	                                    [
	                                        1024 * 1024 * 1024 * 1024,
	                                        "black",
	                                    ],
	                                ],
	                            };
	                        var ops_yaxis = {
	                                height: 40,
	                                bg_color: "rgb(237,248,233)",
	                                skip_color: "white",
	                                format: integral_formatter(),
	                                colors: [
	                                    [
	                                        5,
	                                        "rgb(186,228,179)",
	                                    ],
	                                    [
	                                        20,
	                                        "rgb(116,196,118)",
	                                    ],
	                                    [
	                                        100,
	                                        "rgb(49,163,84)",
	                                    ],
	                                    [
	                                        1000,
	                                        "rgb(0,109,44)",
	                                    ],
	                                    [
	                                        100000,
	                                        "black",
	                                    ],
	                                ],
	                            };
	                        var num_yaxis = {
	                                height: 40,
	                                bg_color: "rgb(237,248,233)",
	                                skip_color: "white",
	                                format: integral_formatter(),
	                                colors: [
	                                    [
	                                        5,
	                                        "rgb(186,228,179)",
	                                    ],
	                                    [
	                                        10,
	                                        "rgb(116,196,118)",
	                                    ],
	                                    [
	                                        20,
	                                        "rgb(49,163,84)",
	                                    ],
	                                    [
	                                        100,
	                                        "rgb(0,109,44)",
	                                    ],
	                                    [
	                                        1000,
	                                        "black",
	                                    ],
	                                ],
	                            };
	                        return {
	                                tag: "div",
	                                children: {children: [
	                                    {
	                                        tag: "h2",
	                                        children: "Disks",
	                                    },
	                                    compact.render(1100, timestamps, [
	                                        {
	                                            title: "Disk Read Ops",
	                                            values: disk["read.ops"].values,
	                                            yaxis: ops_yaxis,
	                                        },
	                                        {
	                                            title: "Disk Write Ops",
	                                            values: disk["write.ops"].values,
	                                            yaxis: ops_yaxis,
	                                        },
	                                        {
	                                            title: "Disk Read Bytes",
	                                            values: disk["read.bytes"].values,
	                                            yaxis: bytes_yaxis,
	                                        },
	                                        {
	                                            title: "Disk Write Bytes",
	                                            values: disk["write.bytes"].values,
	                                            yaxis: bytes_yaxis,
	                                        },
	                                        {
	                                            title: "Disk in Progress Ops",
	                                            values: disk_in_progress["in_progress"].values,
	                                            yaxis: num_yaxis,
	                                        },
	                                    ]),
	                                ]},
	                            };
	                    },
	                };
	        }
	        function render() {
	            return {
	                    key: "b-status:render",
	                    children: function (old_node) {
	                        var query = old_node && old_node.store_query || new Query(2000, {
	                                memory: {
	                                    series: {
	                                        source: "Fine",
	                                        condition: [
	                                            "RegexLike",
	                                            "metric",
	                                            "^memory\.",
	                                        ],
	                                    },
	                                    extract: ["Tip"],
	                                    functions: [],
	                                },
	                                cpu: {
	                                    series: {
	                                        source: "Fine",
	                                        condition: [
	                                            "RegexLike",
	                                            "metric",
	                                            "^cpu\.",
	                                        ],
	                                    },
	                                    extract: [
	                                        "HistoryByNum",
	                                        1100,
	                                    ],
	                                    functions: [
	                                        ["NonNegativeDerivative"],
	                                        [
	                                            "SumBy",
	                                            "metric",
	                                            "Ignore",
	                                            true,
	                                        ],
	                                    ],
	                                },
	                                network: {
	                                    series: {
	                                        source: "Fine",
	                                        condition: [
	                                            "And",
	                                            [
	                                                "RegexLike",
	                                                "metric",
	                                                "^net.interface.[rt]x.bytes$",
	                                            ],
	                                            [
	                                                "Not",
	                                                [
	                                                    "Or",
	                                                    [
	                                                        "Eq",
	                                                        "interface",
	                                                        "lo",
	                                                    ],
	                                                    [
	                                                        "RegexLike",
	                                                        "interface",
	                                                        "^tun|^vboxnet",
	                                                    ],
	                                                ],
	                                            ],
	                                        ],
	                                    },
	                                    extract: [
	                                        "HistoryByNum",
	                                        1100,
	                                    ],
	                                    functions: [
	                                        ["NonNegativeDerivative"],
	                                        [
	                                            "SumBy",
	                                            "metric",
	                                            "Ignore",
	                                            true,
	                                        ],
	                                    ],
	                                },
	                                disk: {
	                                    series: {
	                                        source: "Fine",
	                                        condition: [
	                                            "And",
	                                            [
	                                                "RegexLike",
	                                                "metric",
	                                                "^disk.(?:read|write).(:?ops|bytes)$",
	                                            ],
	                                            [
	                                                "RegexLike",
	                                                "device",
	                                                "^sd[a-z]$",
	                                            ],
	                                        ],
	                                    },
	                                    extract: [
	                                        "HistoryByNum",
	                                        1100,
	                                    ],
	                                    functions: [
	                                        ["NonNegativeDerivative"],
	                                        [
	                                            "SumBy",
	                                            "metric",
	                                            "Ignore",
	                                            true,
	                                        ],
	                                    ],
	                                },
	                                disk_in_progress: {
	                                    series: {
	                                        source: "Fine",
	                                        condition: [
	                                            "And",
	                                            [
	                                                "RegexLike",
	                                                "metric",
	                                                "^disk.in_progress$",
	                                            ],
	                                            [
	                                                "RegexLike",
	                                                "device",
	                                                "^sd[a-z]$",
	                                            ],
	                                        ],
	                                    },
	                                    extract: [
	                                        "HistoryByNum",
	                                        1100,
	                                    ],
	                                    functions: [
	                                        ["NonNegativeDerivative"],
	                                        [
	                                            "SumBy",
	                                            "metric",
	                                            "Ignore",
	                                            true,
	                                        ],
	                                    ],
	                                },
	                            });
	                        return {
	                                tag: "div",
	                                store_query: query,
	                                attrs: {class: "b-status container"},
	                                children: {children: [
	                                    {
	                                        tag: "h1",
	                                        children: "System Status",
	                                    },
	                                    ((query.values)?({children: [
	                                        mem(mem_chart(query.values.get("memory"))),
	                                        cpu(query.values.get("cpu").to_dict("metric", "cpu."), query.values.get("cpu").chunks[0][2]),
	                                        net(query.values.get("network").to_dict("metric", "net.interface."), query.values.get("network").chunks[0][2]),
	                                        disk(query.values.get("disk").to_dict("metric", "disk."), query.values.get("disk_in_progress").to_dict("metric", "disk."), query.values.get("disk").chunks[0][2]),
	                                    ]}):("")),
	                                ]},
	                                events: {"$destroyed": ((query.owner_destroyed)?(query.owner_destroyed.handle_event):(function () {
	                                }))},
	                            };
	                    },
	                };
	        }
	        exports.mem = mem
	        exports.cpu = cpu
	        exports.net = net
	        exports.disk = disk
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 13 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(7),
	        __webpack_require__(14),
	        __webpack_require__(15),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_time, compact, _mod_util_stores) {
	        var format_datetime = _mod_util_time.format_datetime;
	        var Follow = _mod_util_stores.Follow;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(".b-compact.bar {\n    height: 41px;\n    border-bottom: solid black 1px;\n    position: relative;\n}\n\n.b-compact.title {\n    font-family: Verdana , Tahoma , sans-serif;\n    text-shadow: 0 1px 0 rgba(255 , 255 , 255 , 0.5);\n    position: absolute;\n    left: 12px;\n    top: 8px;\n    font-size: 18px;\n}\n\n.b-compact.value {\n    font-family: Verdana , Tahoma , sans-serif;\n    text-shadow: 0 1px 0 rgba(255 , 255 , 255 , 0.5);\n    font-size: 18px;\n    position: absolute;\n    right: 0px;\n    top: 0px;\n    padding-top: 8px;\n    padding-right: 8px;\n    height: 41px;\n}\n\n.b-compact.value.follow {\n    border-right: solid black 1px;\n}\n\n.b-compact.footer {\n    position: relative;\n}\n\n.b-compact.footer-time {\n    position: absolute;\n    right: 0px;\n    top: 0px;\n    padding-top: 8px;\n    padding-right: 8px;\n    height: 41px;\n}\n\n.b-compact.xaxis {\n    height: 40px;\n    position: relative;\n    border-bottom: solid black 1px;\n    padding-bottom: 0px;\n}\n\nline.tick.b-compact {\n    stroke: black;\n}\n\ntext.tick.b-compact {\n    font-family: Verdana , Tahoma , sans-serif;\n    text-anchor: middle;\n    font-size: 12px;\n}\n\n"))
	        document.head.appendChild(_style)
	        function render(width, timestamps, items) {
	            return {
	                    key: "b-compact:render",
	                    children: function (old_node) {
	                        var mouse_position = old_node && old_node.store_mouse_position || new Follow();
	                        var _stream_1 = mouse_position;
	                        var xaxis = compact.xaxis(timestamps, width);
	                        return {
	                                tag: "div",
	                                store_mouse_position: mouse_position,
	                                children: {children: [
	                                    {
	                                        tag: "div",
	                                        attrs: {
	                                            style: {width: width + "px"},
	                                            class: "b-compact xaxis",
	                                        },
	                                        children: {
	                                            tag: "svg",
	                                            attrs: {
	                                                width: String(width),
	                                                height: "40",
	                                            },
	                                            children: xaxis.ticks.map(function (tick) {
	                                                return {
	                                                        tag: "g",
	                                                        attrs: {transform: "translate(" + String(tick.x) + ", 40)"},
	                                                        children: {children: [
	                                                            {
	                                                                tag: "line",
	                                                                attrs: {
	                                                                    y2: "-6",
	                                                                    x2: "0",
	                                                                    class: "b-compact tick",
	                                                                },
	                                                            },
	                                                            {
	                                                                tag: "text",
	                                                                attrs: {
	                                                                    y: "-8",
	                                                                    class: "b-compact tick",
	                                                                },
	                                                                children: String(tick.text),
	                                                            },
	                                                        ]},
	                                                        key: tick.text,
	                                                    };
	                                            }),
	                                        },
	                                    },
	                                    {
	                                        tag: "div",
	                                        children: items.map(function (item) {
	                                            return ((item.values)?({
	                                                    tag: "div",
	                                                    attrs: {
	                                                        style: {width: width + "px"},
	                                                        class: "b-compact bar",
	                                                    },
	                                                    children: {children: [
	                                                        compact.draw(xaxis, item.yaxis, item.values),
	                                                        {
	                                                            tag: "div",
	                                                            attrs: {class: "b-compact title"},
	                                                            children: String(item.title),
	                                                        },
	                                                        ((mouse_position.x !== null && mouse_position.x < width)?(function (old_node) {
	                                                            var px = xaxis.pixels[mouse_position.x];
	                                                            return {
	                                                                    tag: "div",
	                                                                    attrs: {
	                                                                        style: {right: width - mouse_position.x + "px"},
	                                                                        class: "b-compact value follow",
	                                                                    },
	                                                                    children: ((px)?(((!isNaN(item.values[px.index]))?(((item.yaxis.format)?(String(item.yaxis.format(item.values[px.index]))):(String(item.values[px.index].toFixed(2))))):(""))):("--")),
	                                                                };
	                                                        }):({
	                                                            tag: "div",
	                                                            attrs: {class: "b-compact value"},
	                                                            children: ((!isNaN(item.values[0]))?(((item.yaxis.format)?(String(item.yaxis.format(item.values[0]))):(String(item.values[0].toFixed(2))))):("")),
	                                                        })),
	                                                    ]},
	                                                    key: item.title + ":0",
	                                                }):({
	                                                    tag: "div",
	                                                    attrs: {
	                                                        style: {width: width + "px"},
	                                                        class: "b-compact bar nodata",
	                                                    },
	                                                    children: "-- no data --",
	                                                    key: item.title + ":1",
	                                                }));
	                                        }),
	                                    },
	                                    ((mouse_position.x !== null && mouse_position.x < width)?({
	                                        tag: "div",
	                                        attrs: {
	                                            style: {width: width + "px"},
	                                            class: "b-compact footer",
	                                        },
	                                        children: function (old_node) {
	                                            var px = xaxis.pixels[mouse_position.x];
	                                            return {
	                                                    tag: "div",
	                                                    attrs: {
	                                                        style: {right: width - mouse_position.x + "px"},
	                                                        class: "b-compact footer-time follow",
	                                                    },
	                                                    children: ((px)?(String(format_datetime(px.exact_time))):("--")),
	                                                };
	                                        },
	                                    }):(function (old_node) {
	                                        var px = xaxis.pixels[width - 1];
	                                        return {
	                                                tag: "div",
	                                                attrs: {
	                                                    style: {width: width + "px"},
	                                                    class: "b-compact footer",
	                                                },
	                                                children: ((px)?({
	                                                    tag: "div",
	                                                    attrs: {class: "b-compact footer-time"},
	                                                    children: String(format_datetime(px.exact_time)),
	                                                }):("")),
	                                            };
	                                    })),
	                                ]},
	                                events: {
	                                    mousemove: _stream_1.mousemove.handle_event,
	                                    "$destroyed": ((mouse_position.owner_destroyed)?(mouse_position.owner_destroyed.handle_event):(function () {
	                                    })),
	                                    mouseleave: _stream_1.mouseleave.handle_event,
	                                    mouseenter: _stream_1.mouseenter.handle_event,
	                                },
	                            };
	                    },
	                };
	        }
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 14 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _slicedToArray = (function () { function sliceIterator(arr, i) { var _arr = []; var _n = true; var _d = false; var _e = undefined; try { for (var _i = arr[Symbol.iterator](), _s; !(_n = (_s = _i.next()).done); _n = true) { _arr.push(_s.value); if (i && _arr.length === i) break; } } catch (err) { _d = true; _e = err; } finally { try { if (!_n && _i['return']) _i['return'](); } finally { if (_d) throw _e; } } return _arr; } return function (arr, i) { if (Array.isArray(arr)) { return arr; } else if (Symbol.iterator in Object(arr)) { return sliceIterator(arr, i); } else { throw new TypeError('Invalid attempt to destructure non-iterable instance'); } }; })();

	exports.xaxis = xaxis;
	exports.draw = draw;

	var _utilTime = __webpack_require__(7);

	function xaxis(timestamps, width) {
	    var step = arguments.length <= 2 || arguments[2] === undefined ? 2000 : arguments[2];

	    var ticks = [];
	    var tick_pixels = 60;
	    var now = new Date().getTime();
	    var tick_step = step * tick_pixels;
	    var pixels = new Array(width);
	    var tick = Math.floor(now / tick_step) * tick_step;
	    var px = width - Math.floor((now - tick) / step);
	    while (px > 0) {
	        ticks.push({
	            x: px,
	            text: (0, _utilTime.format_time)((0, _utilTime.from_ms)(tick))
	        });
	        px -= 60;
	        tick -= tick_step;
	    }
	    var start = Math.floor(now / step) * step;
	    for (var i = timestamps.length - 1; i >= 0; --i) {
	        var tx = timestamps[i];
	        var _px = width - Math.round((start - tx) / step);
	        if (_px < 0 || _px >= width) {
	            continue;
	        }
	        if (pixels[_px]) {
	            //console.warn("Duplicate pixel", px, tx, start)
	        }
	        pixels[_px] = {
	            index: i,
	            exact_time: tx
	        };
	    }
	    return { ticks: ticks, pixels: pixels, width: width };
	}

	function draw_on(canvas, xaxis, yaxis, data) {
	    canvas.width = xaxis.width;
	    canvas.height = yaxis.height;
	    var ctx = canvas.getContext("2d");
	    for (var i = 0, il = xaxis.pixels.length; i < il; ++i) {
	        var px = xaxis.pixels[i];
	        var val = px ? data[px.index] : null;
	        if (px == null || val == null) {
	            ctx.fillStyle = yaxis.skip_color;
	            ctx.fillRect(i, 0, 1, yaxis.height);
	            continue;
	        }
	        var prev_thresh = 0;
	        var prev_color = yaxis.bg_color;
	        var idx = 0;
	        var _iteratorNormalCompletion = true;
	        var _didIteratorError = false;
	        var _iteratorError = undefined;

	        try {
	            for (var _iterator = yaxis.colors[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
	                var _step$value = _slicedToArray(_step.value, 2);

	                var thresh = _step$value[0];
	                var color = _step$value[1];

	                if (val < thresh) {
	                    break;
	                }
	                prev_thresh = thresh;
	                prev_color = color;
	                idx += 1;
	            }
	            //let h = Math.ceil(val/thresh * yaxis.height)
	        } catch (err) {
	            _didIteratorError = true;
	            _iteratorError = err;
	        } finally {
	            try {
	                if (!_iteratorNormalCompletion && _iterator['return']) {
	                    _iterator['return']();
	                }
	            } finally {
	                if (_didIteratorError) {
	                    throw _iteratorError;
	                }
	            }
	        }

	        var h = Math.ceil((val - prev_thresh) / (thresh - prev_thresh) * yaxis.height);
	        ctx.fillStyle = color;
	        ctx.fillRect(i, yaxis.height - h, 1, h);
	        ctx.fillStyle = prev_color;
	        ctx.fillRect(i, 0, 1, yaxis.height - h);
	    }
	}

	function draw(xaxis, yaxis, data) {
	    return function drawer(old_elem) {
	        if (old_elem) {
	            draw_on(old_elem.dom, xaxis, yaxis, data);
	        } else {
	            return { 'tag': 'canvas', 'attr': {
	                    'width': String(xaxis.width),
	                    'height': String(yaxis.height)
	                }, 'events': {
	                    '$created': function $created(ev) {
	                        draw_on(ev.target, xaxis, yaxis, data);
	                    }
	                } };
	        }
	    };
	}

/***/ },
/* 15 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	var _utilStreams = __webpack_require__(16);

	var Tooltip = (function () {
	    function Tooltip() {
	        _classCallCheck(this, Tooltip);

	        this.mouseenter = new _utilStreams.Stream('tooltip_hover');
	        this.enter = new _utilStreams.Stream('tooltip_hover');
	        this.mouseleave = new _utilStreams.Stream('tooltip_leave');
	        this.mouseenter.handle(this.show.bind(this));
	        this.mouseleave.handle(this.hide.bind(this));
	        this.enter.handle(this.show_with_data.bind(this));
	        this.visible = false;
	    }

	    _createClass(Tooltip, [{
	        key: 'show',
	        value: function show(ev) {
	            this.x = ev.pageX;
	            this.y = ev.pageY;
	            this.visible = true;
	        }
	    }, {
	        key: 'show_with_data',
	        value: function show_with_data(data) {
	            this.data = data;
	            this.visible = true;
	        }
	    }, {
	        key: 'hide',
	        value: function hide(ev) {
	            this.visible = false;
	        }
	    }, {
	        key: 'style',
	        value: function style() {
	            return {
	                position: 'fixed',
	                left: this.x + 'px',
	                top: this.y + 'px'
	            };
	        }
	    }]);

	    return Tooltip;
	})();

	exports.Tooltip = Tooltip;

	var Toggle = (function () {
	    function Toggle() {
	        _classCallCheck(this, Toggle);

	        this.toggle = new _utilStreams.Stream('toggle_event');
	        this.toggle.handle(this.do_toggle.bind(this));
	        this.visible = false;
	    }

	    _createClass(Toggle, [{
	        key: 'do_toggle',
	        value: function do_toggle() {
	            this.visible = !this.visible;
	        }
	    }]);

	    return Toggle;
	})();

	exports.Toggle = Toggle;

	var Value = (function () {
	    function Value() {
	        _classCallCheck(this, Value);

	        this.keydown = new _utilStreams.Stream('set_value');
	        this.keydown.handle(this.store.bind(this));
	        this.change = this.keydown;
	        this.keyup = this.keydown;
	        this.value = null;
	    }

	    _createClass(Value, [{
	        key: 'store',
	        value: function store(ev) {
	            this.value = ev.target.value;
	        }
	    }]);

	    return Value;
	})();

	exports.Value = Value;

	var Follow = (function () {
	    function Follow() {
	        _classCallCheck(this, Follow);

	        this.mousemove = new _utilStreams.Stream('mousemove');
	        this.mouseenter = new _utilStreams.Stream('mouseenter');
	        this.mouseleave = new _utilStreams.Stream('mouseleave');
	        this.owner_destroyed = new _utilStreams.Stream('owner_destroyed');
	        this.mousemove.handle(this.set_coords.bind(this));
	        this.mouseenter.handle(this.set_coords.bind(this));
	        this.mouseleave.handle(this.do_mouseleave.bind(this));
	        this.owner_destroyed.handle(this.cleanup.bind(this));
	        this.x = null;
	        this.y = null;
	        this._timer = null;
	    }

	    _createClass(Follow, [{
	        key: 'set_coords',
	        value: function set_coords(ev) {
	            this._reset_timer();
	            var rect = ev.currentTarget.getBoundingClientRect();
	            this.x = Math.floor(ev.clientX - rect.left);
	            this.y = Math.floor(ev.clientY - rect.top);
	        }
	    }, {
	        key: 'do_mouseleave',
	        value: function do_mouseleave() {
	            this._timer = setTimeout(this.reset_coords.bind(this), 500);
	        }
	    }, {
	        key: 'reset_coords',
	        value: function reset_coords() {
	            this.x = null;
	            this.y = null;
	        }
	    }, {
	        key: '_reset_timer',
	        value: function _reset_timer() {
	            if (this._timer) {
	                clearInterval(this._timer);
	                this._timer = null;
	            }
	        }
	    }, {
	        key: 'cleanup',
	        value: function cleanup() {
	            this.reset_coords();
	            this._reset_timer();
	        }
	    }]);

	    return Follow;
	})();

	exports.Follow = Follow;

/***/ },
/* 16 */
/***/ function(module, exports, __webpack_require__) {

	"use strict";

	Object.defineProperty(exports, "__esModule", {
	    value: true
	});

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { "default": obj }; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

	var _utilRender = __webpack_require__(2);

	var _utilRender2 = _interopRequireDefault(_utilRender);

	var Stream = (function () {
	    function Stream(name) {
	        _classCallCheck(this, Stream);

	        this.name = name;
	        this.handle_event = this.handle_event.bind(this);
	        this._handlers = [];
	    }

	    _createClass(Stream, [{
	        key: "handle_event",
	        value: function handle_event(ev) {
	            console.log("EVENT", this.name, ev, this._handlers);
	            var h = this._handlers;
	            for (var i = 0, li = h.length; i < li; ++i) {
	                try {
	                    h[i](ev);
	                } catch (e) {
	                    console.error("Error handing event", ev, "in stream", this.name, e);
	                }
	            }
	            _utilRender2["default"].update();
	        }
	    }, {
	        key: "handle",
	        value: function handle(fun) {
	            this._handlers.push(fun);
	        }
	    }, {
	        key: "map",
	        value: function map(fun) {
	            var result = new Stream(this.name + '/' + fun.name);
	            result.handle((function (ev) {
	                return this.handle_event(fun(ev));
	            }).bind(this));
	            return result;
	        }
	    }]);

	    return Stream;
	})();

	exports.Stream = Stream;

/***/ },
/* 17 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(18),
	        __webpack_require__(15),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, donut, _mod_util_stores) {
	        var Tooltip = _mod_util_stores.Tooltip;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode("svg.b-donut {\n    vertical-align: middle;\n}\n\n.b-donut.hover {\n    stroke: black;\n}\n\n.b-donut.hint {\n    position: absolute;\n    top: 10px;\n    left: 10px;\n    pointer-events: none;\n    font-family: Verdana , Tahoma , sans-serif;\n    text-shadow: 0 1px 0 rgba(255 , 255 , 255 , 0.5);\n}\n\n.b-donut.canvas {\n    display: inline-block;\n}\n\n"))
	        document.head.appendChild(_style)
	        function render(items, width, height, total) {
	            return {
	                    key: "b-donut:render",
	                    children: function (old_node) {
	                        var tooltip = old_node && old_node.store_tooltip || new Tooltip();
	                        return {
	                                tag: "div",
	                                store_tooltip: tooltip,
	                                attrs: {class: "b-donut canvas"},
	                                children: {children: [
	                                    function (old_node) {
	                                        var paths = donut.with_paths(items, total, width);
	                                        return {
	                                                tag: "svg",
	                                                attrs: {
	                                                    style: {
	                                                        width: width + "px",
	                                                        height: height + "px",
	                                                    },
	                                                    class: "b-donut",
	                                                },
	                                                children: {
	                                                    tag: "g",
	                                                    children: paths.map(function (item) {
	                                                        return {
	                                                                key: item.title,
	                                                                children: function (old_node) {
	                                                                    var _stream_0 = tooltip;
	                                                                    return {
	                                                                            tag: "path",
	                                                                            attrs: {
	                                                                                fill: item.color,
	                                                                                title: item.title,
	                                                                                d: item.path,
	                                                                            },
	                                                                            children: {children: []},
	                                                                            events: {
	                                                                                mouseenter: tooltip.enter.map(function (ev) {
	                                                                                    return item;
	                                                                                }).handle_event,
	                                                                                mouseleave: _stream_0.mouseleave.handle_event,
	                                                                            },
	                                                                        };
	                                                                },
	                                                            };
	                                                    }),
	                                                },
	                                            };
	                                    },
	                                    ((tooltip.visible)?({
	                                        tag: "div",
	                                        attrs: {class: "b-donut hint"},
	                                        children: String(tooltip.data.title) + ": " + String(tooltip.data.text),
	                                    }):("")),
	                                ]},
	                                events: {"$destroyed": ((tooltip.owner_destroyed)?(tooltip.owner_destroyed.handle_event):(function () {
	                                }))},
	                            };
	                    },
	                };
	        }
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 18 */
/***/ function(module, exports) {

	"use strict";

	Object.defineProperty(exports, "__esModule", {
	    value: true
	});

	var _extends = Object.assign || function (target) { for (var i = 1; i < arguments.length; i++) { var source = arguments[i]; for (var key in source) { if (Object.prototype.hasOwnProperty.call(source, key)) { target[key] = source[key]; } } } return target; };

	exports.with_paths = with_paths;
	var RAD = Math.PI / 180;

	// D65 standard referent
	var LAB_X = 0.950470;
	var LAB_Z = 1.088830;

	function _lab_xyz(v) {
	    return v > 0.206893034 ? v * v * v : (v - 4 / 29) / 7.787037;
	}

	function _xyz_rgb(v) {
	    return Math.round(255 * (v <= 0.00304 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055));
	}

	function hcl_color(h, c, l) {
	    // HCL -> LAB
	    h *= RAD;
	    var a = Math.cos(h) * c;
	    var b = Math.sin(h) * c;
	    // LAB -> XYZ
	    var y = (l + 16) / 116;
	    var x = y + a / 500;
	    var z = y - b / 200;
	    x = lab_xyz(x) * LAB_X;
	    y = lab_xyz(y); // * one
	    z = lab_xyz(z) * LAB_Z;
	    // XYZ -> RGB
	    var r = _xyz_rgb(3.2404542 * x - 1.5371385 * y - 0.4985314 * z);
	    var g = _xyz_rgb(-0.9692660 * x + 1.8760108 * y + 0.0415560 * z);
	    var b = _xyz_rgb(0.0556434 * x - 0.2040259 * y + 1.0572252 * z);
	    return "rgb(" + r + "," + g + "," + b + ")";
	}

	function sector(cx, cy, r1, r2, sa, ea) {
	    var c1 = Math.cos(-sa * RAD);
	    var c2 = Math.cos(-ea * RAD);
	    var s1 = Math.sin(-sa * RAD);
	    var s2 = Math.sin(-ea * RAD);

	    var x1 = cx + r2 * c1;
	    var y1 = cy + r2 * s1;
	    var large = +(Math.abs(ea - sa) > 180);
	    return "M " + (cx + r2 * c1) + ", " + (cy + r2 * s1) + "\n            A " + r2 + ", " + r2 + ", 0, " + large + ", 1, " + (cx + r2 * c2) + ", " + (cy + r2 * s2) + "\n            L " + (cx + r1 * c2) + ", " + (cy + r1 * s2) + "\n            A " + r1 + ", " + r1 + ", 0, " + large + ", 0, " + (cx + r1 * c1) + ", " + (cy + r1 * s1) + "\n            z";
	}

	function with_paths(items, total, size) {
	    var result = [];
	    var angle = 0;
	    var cx = size >> 1;
	    var cy = size >> 1;
	    var r = Math.min(cx, cy);
	    for (var i = 0, il = items.length; i < il; ++i) {
	        var it = items[i];
	        if (it.value == 0 || !it.color) {
	            continue;
	        }
	        var sangle = angle;
	        if (total == 0) {
	            angle = sangle + 360;
	        } else if (it.value == total) {
	            angle -= 360 * it.value / total - 0.01;
	        } else {
	            angle -= 360 * it.value / total;
	        }
	        var path = sector(cx, cy,
	        // TODO(tailhook) use some interpolation
	        r > 120 ? r * 0.50 : r > 50 ? r * 0.4 : r * 0.2, r, sangle, angle);
	        result.push(_extends({ path: path }, it));
	    }
	    return result;
	}

/***/ },
/* 19 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(15),
	        __webpack_require__(11),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_stores, _mod_util_plot) {
	        var Toggle = _mod_util_stores.Toggle;
	        var Plot = _mod_util_plot.Plot;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(".b-plot.border {\n    fill: none;\n    stroke: silver;\n    stroke-width: 1px;\n}\n\n.b-plot.line {\n    fill: none;\n    stroke: blue;\n}\n\n.b-plot.value {\n    display: inline-block;\n    width: 125px;\n    background: lightgray;\n    padding: 4px 16px;\n    margin: 2px;\n    border-radius: 4px;\n}\n\npath.y.tick.b-plot {\n    stroke: black;\n    stroke-width: 1px;\n}\n\ntext.y.tick.b-plot {\n    text-anchor: end;\n}\n\n"))
	        document.head.appendChild(_style)
	        function render(timestamps, values) {
	            return {
	                    key: "b-plot:render",
	                    children: function (old_node) {
	                        var toggle = old_node && old_node.store_toggle || new Toggle();
	                        var height = toggle.visible && 256 || 128;
	                        var plot = new Plot(timestamps, values, 512, height);
	                        return {
	                                tag: "div",
	                                store_toggle: toggle,
	                                attrs: {class: "b-plot row"},
	                                children: {children: [
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-plot col-xs-6"},
	                                        children: {
	                                            tag: "svg",
	                                            attrs: {style: {
	                                                width: "564px",
	                                                height: height + 14 + "px",
	                                            }},
	                                            children: {
	                                                tag: "g",
	                                                attrs: {transform: "translate(50, 8)"},
	                                                children: {children: [
	                                                    {
	                                                        tag: "rect",
	                                                        attrs: {
	                                                            width: 514,
	                                                            height: height,
	                                                            x: "-1",
	                                                            y: "-1",
	                                                            class: "b-plot border",
	                                                        },
	                                                    },
	                                                    {
	                                                        tag: "path",
	                                                        attrs: {
	                                                            d: plot.path,
	                                                            class: "b-plot line",
	                                                        },
	                                                    },
	                                                    {
	                                                        tag: "g",
	                                                        attrs: {class: "b-plot y ticks"},
	                                                        children: plot.yaxis.ticks.map(function (tick) {
	                                                            return {
	                                                                    key: tick,
	                                                                    children: function (old_node) {
	                                                                        var ticky = height - tick.value * plot.yscale;
	                                                                        return {
	                                                                                tag: "g",
	                                                                                attrs: {
	                                                                                    transform: "translate(0, " + String(ticky) + ")",
	                                                                                    class: "b-plot y tick",
	                                                                                },
	                                                                                children: {children: [
	                                                                                    {
	                                                                                        tag: "path",
	                                                                                        attrs: {
	                                                                                            d: "M -6,0 L 0,0",
	                                                                                            class: "b-plot y tick",
	                                                                                        },
	                                                                                    },
	                                                                                    {
	                                                                                        tag: "g",
	                                                                                        attrs: {transform: "translate(-8, 0)"},
	                                                                                        children: {
	                                                                                            tag: "text",
	                                                                                            attrs: {class: "b-plot y tick"},
	                                                                                            children: String(tick.label),
	                                                                                        },
	                                                                                    },
	                                                                                ]},
	                                                                            };
	                                                                    },
	                                                                };
	                                                        }),
	                                                    },
	                                                ]},
	                                            },
	                                        },
	                                    },
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-plot col-xs-4"},
	                                        children: {children: [
	                                            {
	                                                tag: "div",
	                                                attrs: {class: "b-plot value"},
	                                                children: {children: [
	                                                    "Max: ",
	                                                    String(plot.max),
	                                                ]},
	                                            },
	                                            {
	                                                tag: "div",
	                                                attrs: {class: "b-plot value"},
	                                                children: {children: [
	                                                    "Min: ",
	                                                    String(plot.min),
	                                                ]},
	                                            },
	                                            {
	                                                tag: "button",
	                                                attrs: {class: "b-plot btn btn-default"},
	                                                children: ((toggle.visible)?({
	                                                    tag: "span",
	                                                    attrs: {class: "b-plot glyphicon glyphicon-chevron-up"},
	                                                }):({
	                                                    tag: "span",
	                                                    attrs: {class: "b-plot glyphicon glyphicon-chevron-down"},
	                                                })),
	                                                events: {click: toggle.toggle.handle_event},
	                                            },
	                                        ]},
	                                    },
	                                ]},
	                                events: {"$destroyed": ((toggle.owner_destroyed)?(toggle.owner_destroyed.handle_event):(function () {
	                                }))},
	                            };
	                    },
	                };
	        }
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 20 */
/***/ function(module, exports) {

	"use strict";

	Object.defineProperty(exports, "__esModule", {
	    value: true
	});
	exports.last = last;
	exports.from_obj = from_obj;

	function last(lst) {
	    return lst[lst.length - 1];
	}

	function from_obj(obj) {
	    var res = [];
	    var _iteratorNormalCompletion = true;
	    var _didIteratorError = false;
	    var _iteratorError = undefined;

	    try {
	        for (var _iterator = Object.keys(obj)[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
	            var k = _step.value;

	            res.push({});
	        }
	    } catch (err) {
	        _didIteratorError = true;
	        _iteratorError = err;
	    } finally {
	        try {
	            if (!_iteratorNormalCompletion && _iterator["return"]) {
	                _iterator["return"]();
	            }
	        } finally {
	            if (_didIteratorError) {
	                throw _iteratorError;
	            }
	        }
	    }

	    return res;
	}

/***/ },
/* 21 */
/***/ function(module, exports) {

	/*
	const RE_PATTERN = /\{([a-zA-Z_0-9]+)([^:}]+)(?::([^}]+)\}/

	export function format(pattern, ...replacements) {
	    RE_PATTERN.sub(pattern, function(match) {
	    })
	}
	*/

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});
	exports.number_formatter = number_formatter;
	exports.percent_formatter = percent_formatter;
	exports.already_percent_formatter = already_percent_formatter;
	exports.bytes_formatter = bytes_formatter;
	exports.integral_formatter = integral_formatter;

	function number_formatter() {
	    var decimals = arguments.length <= 0 || arguments[0] === undefined ? 0 : arguments[0];

	    return function (x) {
	        return x.toFixed(decimals);
	    };
	}

	function percent_formatter() {
	    var decimals = arguments.length <= 0 || arguments[0] === undefined ? 0 : arguments[0];

	    return function (x) {
	        return (x * 100).toFixed(decimals) + '%';
	    };
	}

	function already_percent_formatter() {
	    var decimals = arguments.length <= 0 || arguments[0] === undefined ? 0 : arguments[0];

	    return function (x) {
	        return x.toFixed(decimals) + '%';
	    };
	}

	function bytes_formatter() {
	    return function (x) {
	        if (x >= 10737418240) {
	            return (x / 10737418240).toFixed(0) + 'Gi';
	        } else if (x >= 5368709120) {
	            return (x / 10737418240).toFixed(1) + 'Gi';
	        } else if (x >= 10 << 19) {
	            return (x >> 20) + 'Mi';
	        } else if (x >= 1 << 19) {
	            return (x / (1 << 20)).toFixed(1) + 'Mi';
	        } else if (x >= 10 << 9) {
	            return (x >> 10) + 'ki';
	        } else if (x >= 1 << 9) {
	            return (x / (1 << 10)).toFixed(1) + 'ki';
	        } else {
	            return (x | 0) + 'b';
	        }
	    };
	}

	function integral_formatter() {
	    return function (x) {
	        var res = (x | 0) % 1000;
	        var nlen = 3;
	        x = x / 1000 | 0;
	        while (x > 0) {
	            switch (nlen - res.length) {
	                case 0:
	                    break;
	                case 1:
	                    res = ",0" + res;break;
	                case 2:
	                    res = ",00" + res;break;
	            }
	            res = x % 1000 + "," + res;
	            x = x / 1000 | 0;
	            nlen += 4;
	        }
	        return res;
	    };
	}

/***/ },
/* 22 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _slicedToArray = (function () { function sliceIterator(arr, i) { var _arr = []; var _n = true; var _d = false; var _e = undefined; try { for (var _i = arr[Symbol.iterator](), _s; !(_n = (_s = _i.next()).done); _n = true) { _arr.push(_s.value); if (i && _arr.length === i) break; } } catch (err) { _d = true; _e = err; } finally { try { if (!_n && _i['return']) _i['return'](); } finally { if (_d) throw _e; } } return _arr; } return function (arr, i) { if (Array.isArray(arr)) { return arr; } else if (Symbol.iterator in Object(arr)) { return sliceIterator(arr, i); } else { throw new TypeError('Invalid attempt to destructure non-iterable instance'); } }; })();

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	var _get = function get(_x10, _x11, _x12) { var _again = true; _function: while (_again) { var object = _x10, property = _x11, receiver = _x12; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x10 = parent; _x11 = property; _x12 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	var _utilTime = __webpack_require__(7);

	var _utilRender = __webpack_require__(2);

	var _utilRequest = __webpack_require__(8);

	var _utilStreams = __webpack_require__(16);

	var _utilProbor = __webpack_require__(4);

	var EMPTY_KEY = {};

	exports.EMPTY_KEY = EMPTY_KEY;

	var Key = (function (_Proto) {
	    _inherits(Key, _Proto);

	    function Key() {
	        _classCallCheck(this, Key);

	        _get(Object.getPrototypeOf(Key.prototype), 'constructor', this).apply(this, arguments);
	    }

	    _createClass(Key, [{
	        key: 'decode',
	        value: function decode(val) {
	            if (!val.length) {
	                return EMPTY_KEY;
	            } else {
	                return CBOR.decode(val.buffer.slice(val.byteOffset, val.byteOffset + val.byteLength));
	            }
	        }
	    }]);

	    return Key;
	})(_utilProbor.Proto);

	var Timestamp = (function (_Proto2) {
	    _inherits(Timestamp, _Proto2);

	    function Timestamp() {
	        _classCallCheck(this, Timestamp);

	        _get(Object.getPrototypeOf(Timestamp.prototype), 'constructor', this).apply(this, arguments);
	    }

	    _createClass(Timestamp, [{
	        key: 'decode',
	        value: function decode(val) {
	            var dt = new Date();
	            dt.setTime(val);
	            return dt;
	        }
	    }]);

	    return Timestamp;
	})(_utilProbor.Proto);

	var BaseQuery = (function () {
	    function BaseQuery(url) {
	        var post_data = arguments.length <= 1 || arguments[1] === undefined ? null : arguments[1];
	        var response_type = arguments.length <= 2 || arguments[2] === undefined ? 'json' : arguments[2];
	        var interval = arguments.length <= 3 || arguments[3] === undefined ? 2000 : arguments[3];

	        _classCallCheck(this, BaseQuery);

	        this._timer = null;
	        this.owner_destroyed = new _utilStreams.Stream('query_remote_destroyed');
	        this.owner_destroyed.handle(this.stop.bind(this));
	        this.url = url;
	        this.interval = interval;
	        this.post_data = post_data;
	        this.response_type = response_type;
	    }

	    _createClass(BaseQuery, [{
	        key: 'start',
	        value: function start() {
	            var _this = this;

	            if (this._timer) {
	                clearInterval(this._timer);
	            }
	            this._timer = setInterval(function () {
	                return _this.refresh_now();
	            }, this.interval);
	            this.refresh_now();
	        }
	    }, {
	        key: 'stop',
	        value: function stop() {
	            if (this._req) {
	                this._req.abort();
	                this._req = null;
	            }
	            if (this._timer) {
	                clearInterval(this._timer);
	                this._timer = 0;
	            }
	        }
	    }, {
	        key: 'refresh_now',
	        value: function refresh_now() {
	            var _this2 = this;

	            if (this._req) {
	                this._req.onreadystatechange = null;
	                this._req.abort();
	            }
	            var req = this._req = new XMLHttpRequest();
	            var time = new Date();
	            req.onreadystatechange = function (ev) {
	                if (req.readyState < 4) {
	                    return;
	                }
	                _this2.latency = new Date() - time;
	                if (req.status != 200) {
	                    console.error("Error fetching", _this2.url, req);
	                    _this2.error = Error('Status ' + req.status);
	                    return;
	                }
	                var data = _this2.decode(req.response);
	                console.log("Query", _this2.url, "returned", data);
	                _this2.apply(data);
	                (0, _utilRender.update)();
	            };
	            var post_data = this.post_data;
	            if (post_data) {
	                req.open('POST', this.url, true);
	                req.responseType = this.response_type;
	                req.send(post_data);
	            } else {
	                req.open('GET', this.url, true);
	                req.responseType = this.response_type;
	                req.send();
	            }
	        }
	    }]);

	    return BaseQuery;
	})();

	exports.BaseQuery = BaseQuery;

	var CborQuery = (function (_BaseQuery) {
	    _inherits(CborQuery, _BaseQuery);

	    function CborQuery(url, schema) {
	        var post_data = arguments.length <= 2 || arguments[2] === undefined ? null : arguments[2];
	        var interval = arguments.length <= 3 || arguments[3] === undefined ? 2000 : arguments[3];

	        _classCallCheck(this, CborQuery);

	        _get(Object.getPrototypeOf(CborQuery.prototype), 'constructor', this).call(this, url, post_data, 'arraybuffer', interval);
	        this.schema = schema;
	    }

	    _createClass(CborQuery, [{
	        key: 'decode',
	        value: function decode(response) {
	            try {
	                return (0, _utilProbor.decode)(this.schema, response);
	            } catch (e) {
	                console.error("Error parsing cbor at", this.url, e.stack);
	                this.error = Error("Bad Cbor");
	                return;
	            }
	        }
	    }]);

	    return CborQuery;
	})(BaseQuery);

	exports.CborQuery = CborQuery;

	var JsonQuery = (function (_BaseQuery2) {
	    _inherits(JsonQuery, _BaseQuery2);

	    function JsonQuery(url) {
	        var post_data = arguments.length <= 1 || arguments[1] === undefined ? null : arguments[1];
	        var interval = arguments.length <= 2 || arguments[2] === undefined ? 2000 : arguments[2];

	        _classCallCheck(this, JsonQuery);

	        _get(Object.getPrototypeOf(JsonQuery.prototype), 'constructor', this).call(this, url, post_data, 'json', interval);
	    }

	    _createClass(JsonQuery, [{
	        key: 'decode',
	        value: function decode(data) {
	            return data;
	        }
	    }]);

	    return JsonQuery;
	})(BaseQuery);

	exports.JsonQuery = JsonQuery;

	var chunk = new _utilProbor.Enum((function () {
	    var State = function State(_ref) {
	        var _ref2 = _slicedToArray(_ref, 2);

	        var ts = _ref2[0];
	        var value = _ref2[1];

	        _classCallCheck(this, State);

	        this.ts = ts;
	        this.value = value;
	    };

	    State.probor_enum_protocol = [new _utilProbor.Tuple(new Timestamp(), new _utilProbor.Str())];

	    var Counter = function Counter(values) {
	        _classCallCheck(this, Counter);

	        this.values = values;
	    };

	    Counter.probor_enum_protocol = [new _utilProbor.List(new _utilProbor.Optional(new _utilProbor.Int()))];

	    var Integer = function Integer(values) {
	        _classCallCheck(this, Integer);

	        this.values = values;
	    };

	    Integer.probor_enum_protocol = [new _utilProbor.List(new _utilProbor.Optional(new _utilProbor.Int()))];

	    var Float = function Float(values) {
	        _classCallCheck(this, Float);

	        this.values = values;
	    };

	    Float.probor_enum_protocol = [new _utilProbor.List(new _utilProbor.Optional(new _utilProbor.Float()))];

	    return {
	        0: State,
	        1: Counter,
	        2: Integer,
	        3: Float
	    };
	})());

	var tip = new _utilProbor.Enum((function () {
	    var State = function State(_ref3) {
	        var _ref32 = _slicedToArray(_ref3, 2);

	        var ts = _ref32[0];
	        var value = _ref32[1];

	        _classCallCheck(this, State);

	        this.ts = ts;
	        this.value = value;
	    };

	    State.probor_enum_protocol = [new Timestamp(), new _utilProbor.Str()];

	    var Counter = function Counter(value) {
	        _classCallCheck(this, Counter);

	        this.value = value;
	    };

	    Counter.probor_enum_protocol = [new _utilProbor.Int()];

	    var Integer = function Integer(value) {
	        _classCallCheck(this, Integer);

	        this.value = value;
	    };

	    Integer.probor_enum_protocol = [new _utilProbor.Int()];

	    var Float = function Float(value) {
	        _classCallCheck(this, Float);

	        this.value = value;
	    };

	    Float.probor_enum_protocol = [new _utilProbor.Float()];

	    return {
	        0: State,
	        1: Counter,
	        2: Integer,
	        3: Float
	    };
	})());

	var SingleSeries = function SingleSeries(key, chunk, timestamps) {
	    _classCallCheck(this, SingleSeries);

	    this.key = key;
	    this.chunk = chunk;
	    this.timestamps = timestamps;
	};

	SingleSeries.probor_enum_protocol = [new Key(), chunk, new _utilProbor.List(new Timestamp())];

	var MultiSeries = (function () {
	    function MultiSeries(chunks) {
	        _classCallCheck(this, MultiSeries);

	        this.chunks = chunks;
	    }

	    _createClass(MultiSeries, [{
	        key: 'to_dict',
	        value: function to_dict(prop, prefix) {
	            var res = {};
	            if (prefix) {
	                var prefix_len = prefix.length;
	                var _iteratorNormalCompletion = true;
	                var _didIteratorError = false;
	                var _iteratorError = undefined;

	                try {
	                    for (var _iterator = this.chunks[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
	                        var _step$value = _slicedToArray(_step.value, 2);

	                        var key = _step$value[0];
	                        var _value = _step$value[1];

	                        var rkey = key[prop];
	                        if (rkey && rkey.substr(0, prefix.length) == prefix) {
	                            res[rkey.substr(prefix.length)] = _value;
	                        } else if (key == EMPTY_KEY) {
	                            res.TOTAL = _value;
	                        }
	                    }
	                } catch (err) {
	                    _didIteratorError = true;
	                    _iteratorError = err;
	                } finally {
	                    try {
	                        if (!_iteratorNormalCompletion && _iterator['return']) {
	                            _iterator['return']();
	                        }
	                    } finally {
	                        if (_didIteratorError) {
	                            throw _iteratorError;
	                        }
	                    }
	                }
	            } else {
	                var _iteratorNormalCompletion2 = true;
	                var _didIteratorError2 = false;
	                var _iteratorError2 = undefined;

	                try {
	                    for (var _iterator2 = this.chunks[Symbol.iterator](), _step2; !(_iteratorNormalCompletion2 = (_step2 = _iterator2.next()).done); _iteratorNormalCompletion2 = true) {
	                        var _step2$value = _slicedToArray(_step2.value, 2);

	                        var key = _step2$value[0];
	                        var _value2 = _step2$value[1];

	                        res[key[prop]] = _value2;
	                    }
	                } catch (err) {
	                    _didIteratorError2 = true;
	                    _iteratorError2 = err;
	                } finally {
	                    try {
	                        if (!_iteratorNormalCompletion2 && _iterator2['return']) {
	                            _iterator2['return']();
	                        }
	                    } finally {
	                        if (_didIteratorError2) {
	                            throw _iteratorError2;
	                        }
	                    }
	                }
	            }
	            return res;
	        }
	    }]);

	    return MultiSeries;
	})();

	MultiSeries.probor_enum_protocol = [new _utilProbor.List(new _utilProbor.Tuple(new Key(), chunk, new _utilProbor.List(new Timestamp())))];

	var SingleTip = (function () {
	    function SingleTip(key, value, timestamps) {
	        _classCallCheck(this, SingleTip);

	        this.key = key;
	        this.value = value;
	        this.timestamps = timestamps;
	    }

	    _createClass(SingleTip, [{
	        key: 'delta_sec',
	        value: function delta_sec() {
	            return (this.timestamps[0] - this.timestamps[1]) / 1000;
	        }
	    }]);

	    return SingleTip;
	})();

	SingleTip.probor_enum_protocol = [new Key(), tip, new _utilProbor.Tuple(new Timestamp(), new Timestamp())];

	var MultiTip = (function () {
	    function MultiTip(values) {
	        _classCallCheck(this, MultiTip);

	        this.values = values;
	    }

	    _createClass(MultiTip, [{
	        key: 'to_dict',
	        value: function to_dict(prop, prefix) {
	            var res = {};
	            if (prefix) {
	                var prefix_len = prefix.length;
	                var _iteratorNormalCompletion3 = true;
	                var _didIteratorError3 = false;
	                var _iteratorError3 = undefined;

	                try {
	                    for (var _iterator3 = this.values[Symbol.iterator](), _step3; !(_iteratorNormalCompletion3 = (_step3 = _iterator3.next()).done); _iteratorNormalCompletion3 = true) {
	                        var _step3$value = _slicedToArray(_step3.value, 2);

	                        var key = _step3$value[0];
	                        var _value3 = _step3$value[1];

	                        var rkey = key[prop];
	                        if (rkey.substr(0, prefix.length) == prefix) {
	                            res[rkey.substr(prefix.length)] = _value3.value;
	                        }
	                    }
	                } catch (err) {
	                    _didIteratorError3 = true;
	                    _iteratorError3 = err;
	                } finally {
	                    try {
	                        if (!_iteratorNormalCompletion3 && _iterator3['return']) {
	                            _iterator3['return']();
	                        }
	                    } finally {
	                        if (_didIteratorError3) {
	                            throw _iteratorError3;
	                        }
	                    }
	                }
	            } else {
	                var _iteratorNormalCompletion4 = true;
	                var _didIteratorError4 = false;
	                var _iteratorError4 = undefined;

	                try {
	                    for (var _iterator4 = this.values[Symbol.iterator](), _step4; !(_iteratorNormalCompletion4 = (_step4 = _iterator4.next()).done); _iteratorNormalCompletion4 = true) {
	                        var _step4$value = _slicedToArray(_step4.value, 2);

	                        var key = _step4$value[0];
	                        var _value4 = _step4$value[1];

	                        res[key[prop]] = _value4.value;
	                    }
	                } catch (err) {
	                    _didIteratorError4 = true;
	                    _iteratorError4 = err;
	                } finally {
	                    try {
	                        if (!_iteratorNormalCompletion4 && _iterator4['return']) {
	                            _iterator4['return']();
	                        }
	                    } finally {
	                        if (_didIteratorError4) {
	                            throw _iteratorError4;
	                        }
	                    }
	                }
	            }
	            return res;
	        }
	    }]);

	    return MultiTip;
	})();

	MultiTip.probor_enum_protocol = [new _utilProbor.List(new _utilProbor.Tuple(new Key(), tip, new _utilProbor.Tuple(new Timestamp(), new Timestamp())))];

	var Chart = function Chart(chart) {
	    _classCallCheck(this, Chart);

	    this.chart = chart;
	};

	Chart.probor_enum_protocol = [new _utilProbor.Dict(new _utilProbor.Str(), new _utilProbor.Int())];

	var Empty = function Empty() {
	    _classCallCheck(this, Empty);
	};

	Empty.probor_enum_protocol = [];

	var Incompatible = function Incompatible(reason) {
	    _classCallCheck(this, Incompatible);

	    this.reason = reason;
	};

	Incompatible.probor_enum_protocol = [new _utilProbor.Enum({
	    100: "CantSumChart",
	    101: "Dissimilar",
	    102: "CantSumTimestamps",
	    103: "CantSumStates",
	    104: "CantDerive"
	})];

	var dataset = new _utilProbor.Enum({
	    100: SingleSeries,
	    101: MultiSeries,
	    200: SingleTip,
	    201: MultiTip,
	    300: Chart,
	    998: Empty,
	    999: Incompatible
	});

	var QueryResponse = (function (_SimpleStruct) {
	    _inherits(QueryResponse, _SimpleStruct);

	    function QueryResponse() {
	        _classCallCheck(this, QueryResponse);

	        _get(Object.getPrototypeOf(QueryResponse.prototype), 'constructor', this).apply(this, arguments);
	    }

	    return QueryResponse;
	})(_utilProbor.SimpleStruct);

	QueryResponse.probor_protocol = new _utilProbor.Struct([["values", null, new _utilProbor.Dict(new _utilProbor.Str(), dataset)]]);

	var Query = (function (_CborQuery) {
	    _inherits(Query, _CborQuery);

	    function Query(interval, rules) {
	        _classCallCheck(this, Query);

	        _get(Object.getPrototypeOf(Query.prototype), 'constructor', this).call(this, '/query.cbor', QueryResponse, JSON.stringify({
	            'rules': rules
	        }), interval);
	        this.start();
	    }

	    _createClass(Query, [{
	        key: 'apply',
	        value: function apply(response) {
	            this.values = response.values;
	        }
	    }]);

	    return Query;
	})(CborQuery);

	exports.Query = Query;

	var hosts_response = new _utilProbor.Dict(new _utilProbor.Str(), new _utilProbor.Dict(new _utilProbor.Str(), dataset));

	var QueryRemote = (function (_CborQuery2) {
	    _inherits(QueryRemote, _CborQuery2);

	    function QueryRemote(rules) {
	        _classCallCheck(this, QueryRemote);

	        _get(Object.getPrototypeOf(QueryRemote.prototype), 'constructor', this).call(this, '/remote/query_by_host.cbor', hosts_response, JSON.stringify({
	            'rules': rules
	        }), 6000);
	        this.start();
	    }

	    _createClass(QueryRemote, [{
	        key: 'apply',
	        value: function apply(obj) {
	            this.response = obj;
	        }
	    }]);

	    return QueryRemote;
	})(CborQuery);

	exports.QueryRemote = QueryRemote;

	var value = new _utilProbor.Enum((function () {
	    var State = function State(_ref4) {
	        var _ref42 = _slicedToArray(_ref4, 2);

	        var ts = _ref42[0];
	        var value = _ref42[1];

	        _classCallCheck(this, State);

	        this.ts = ts;
	        this.value = value;
	    };

	    State.probor_enum_protocol = [new _utilProbor.Tuple(new Timestamp(), new _utilProbor.Str())];

	    var Counter = function Counter(v) {
	        _classCallCheck(this, Counter);

	        this.value = v;
	    };

	    Counter.probor_enum_protocol = [new _utilProbor.Int()];

	    var Integer = function Integer(value) {
	        _classCallCheck(this, Integer);

	        this.value = value;
	    };

	    Integer.probor_enum_protocol = [new _utilProbor.Int()];

	    var Float = function Float(value) {
	        _classCallCheck(this, Float);

	        this.value = value;
	    };

	    Float.probor_enum_protocol = [new _utilProbor.Float()];

	    return {
	        0: State,
	        1: Counter,
	        2: Integer,
	        3: Float
	    };
	})());

	var MetricsResponse = (function (_SimpleStruct2) {
	    _inherits(MetricsResponse, _SimpleStruct2);

	    function MetricsResponse() {
	        _classCallCheck(this, MetricsResponse);

	        _get(Object.getPrototypeOf(MetricsResponse.prototype), 'constructor', this).apply(this, arguments);
	    }

	    return MetricsResponse;
	})(_utilProbor.SimpleStruct);

	MetricsResponse.probor_protocol = new _utilProbor.Struct([["metrics", null, new _utilProbor.List(new _utilProbor.Tuple(new Key(), new Timestamp(), value))]]);

	var MetricsQuery = (function (_CborQuery3) {
	    _inherits(MetricsQuery, _CborQuery3);

	    function MetricsQuery(rules) {
	        _classCallCheck(this, MetricsQuery);

	        _get(Object.getPrototypeOf(MetricsQuery.prototype), 'constructor', this).call(this, '/all_metrics.cbor', MetricsResponse, null, 120000);
	        this.start();
	    }

	    _createClass(MetricsQuery, [{
	        key: 'apply',
	        value: function apply(obj) {
	            this.metrics = obj.metrics;
	        }
	    }]);

	    return MetricsQuery;
	})(CborQuery);

	exports.MetricsQuery = MetricsQuery;

	var RemoteStats = (function (_JsonQuery) {
	    _inherits(RemoteStats, _JsonQuery);

	    function RemoteStats() {
	        var interval = arguments.length <= 0 || arguments[0] === undefined ? 5000 : arguments[0];

	        _classCallCheck(this, RemoteStats);

	        _get(Object.getPrototypeOf(RemoteStats.prototype), 'constructor', this).call(this, '/remote_stats.json', null, interval);
	        this.start();
	    }

	    _createClass(RemoteStats, [{
	        key: 'apply',
	        value: function apply(response) {
	            this.response = response;
	        }
	    }]);

	    return RemoteStats;
	})(JsonQuery);

	exports.RemoteStats = RemoteStats;

	var PeersRequest = (function (_JsonQuery2) {
	    _inherits(PeersRequest, _JsonQuery2);

	    function PeersRequest(only_remote) {
	        var interval = arguments.length <= 1 || arguments[1] === undefined ? 5000 : arguments[1];

	        _classCallCheck(this, PeersRequest);

	        _get(Object.getPrototypeOf(PeersRequest.prototype), 'constructor', this).call(this, '/peers_with_remote.json', null, interval);
	        this.start();
	    }

	    _createClass(PeersRequest, [{
	        key: 'apply',
	        value: function apply(json) {
	            this.peers = json.peers;
	        }
	    }]);

	    return PeersRequest;
	})(JsonQuery);

	exports.PeersRequest = PeersRequest;

/***/ },
/* 23 */
/***/ function(module, exports) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});
	exports.mem_chart = mem_chart;
	exports.cpu_usage = cpu_usage;
	var MEM_COLORS = {
	    MemFree: '#e5f5f9',
	    Buffers: '#99d8c9',
	    Cached: '#2ca25f',
	    Used: '#a0a0a0'
	};

	var MEM_ORDER = {
	    MemTotal: 1,
	    Used: 2,
	    Cached: 3,
	    Buffers: 4,
	    MemFree: 5,
	    Dirty: 6,
	    Writeback: 7,
	    SwapUsed: 8,
	    Committed_AS: 9,
	    CommitLimit: 10
	};

	function mem_chart(metrics) {
	    metrics = metrics.to_dict('metric', 'memory.');
	    metrics.Used = metrics.MemTotal - metrics.MemFree - metrics.Buffers - metrics.Cached;
	    metrics.SwapUsed = metrics.SwapTotal - metrics.SwapFree;
	    return {
	        title: 'Memory',
	        unit: 'MiB',
	        total: metrics.MemTotal,
	        items: Object.keys(metrics).map(function (metricname) {
	            var value = metrics[metricname];
	            return {
	                color: MEM_COLORS[metricname],
	                title: metricname,
	                value: value,
	                text: (value / 1048576).toFixed(1),
	                collapsed: MEM_ORDER[metricname] === undefined
	            };
	        }).sort(function (a, b) {
	            return (MEM_ORDER[a.title] || 10000) - (MEM_ORDER[b.title] || 10000);
	        })
	    };
	}

	function cpu_usage(cpu_total, cpu_idle) {
	    return cpu_idle.values.map(function (x, i) {
	        return cpu_total.values[i] - x;
	    });
	}

/***/ },
/* 24 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _slicedToArray = (function () { function sliceIterator(arr, i) { var _arr = []; var _n = true; var _d = false; var _e = undefined; try { for (var _i = arr[Symbol.iterator](), _s; !(_n = (_s = _i.next()).done); _n = true) { _arr.push(_s.value); if (i && _arr.length === i) break; } } catch (err) { _d = true; _e = err; } finally { try { if (!_n && _i['return']) _i['return'](); } finally { if (_d) throw _e; } } return _arr; } return function (arr, i) { if (Array.isArray(arr)) { return arr; } else if (Symbol.iterator in Object(arr)) { return sliceIterator(arr, i); } else { throw new TypeError('Invalid attempt to destructure non-iterable instance'); } }; })();

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	var _get = function get(_x, _x2, _x3) { var _again = true; _function: while (_again) { var object = _x, property = _x2, receiver = _x3; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x = parent; _x2 = property; _x3 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	var _utilHtml = __webpack_require__(6);

	var _utilTime = __webpack_require__(7);

	var _utilRequest = __webpack_require__(8);

	var _utilBase = __webpack_require__(1);

	var TYPE_TO_ICON = {
	    'Float': (0, _utilHtml.icon)('equalizer'),
	    'Integer': (0, _utilHtml.icon)('stats'),
	    'Counter': (0, _utilHtml.icon)('cd'),
	    'State': (0, _utilHtml.icon)('dashboard')
	};

	var Values = (function (_Component) {
	    _inherits(Values, _Component);

	    function Values() {
	        _classCallCheck(this, Values);

	        _get(Object.getPrototypeOf(Values.prototype), 'constructor', this).apply(this, arguments);
	    }

	    _createClass(Values, [{
	        key: 'init',
	        value: function init() {
	            this.guard('json', new _utilRequest.RefreshJson('/process_values.json')).process(function (data, latency) {
	                var error = null;
	                if (data instanceof Error) {
	                    error = data;
	                }
	                return { error: error, data: data, latency: latency };
	            });
	        }
	    }, {
	        key: 'render',
	        value: function render() {
	            return (0, _utilHtml.tag_class)("div", "container", [(0, _utilHtml.tag)("h1", ["All Values"]), (0, _utilHtml.tag_class)("div", "text-right", this.error ? 'Error: ' + this.error.message : 'Fetched in ' + this.latency + 'ms')].concat(this.data ? this.data.processes.map(this.render_process.bind(this)) : []));
	        }
	    }, {
	        key: 'render_value',
	        value: function render_value(pair) {
	            var _pair = _slicedToArray(pair, 2);

	            var name = _pair[0];
	            var value = _pair[1];

	            delete name.pid;
	            if (value.length !== undefined) {
	                var time = value[0];
	                if (time == 0) {
	                    return { children: [(0, _utilHtml.tag)('tr', [(0, _utilHtml.td_left)(JSON.stringify(name)), (0, _utilHtml.td_left)(TYPE_TO_ICON[value.variant] || value.variant), (0, _utilHtml.td_right)('--')])] };
	                } else {
	                    return { children: [(0, _utilHtml.tag)('tr', [(0, _utilHtml.td_left)(JSON.stringify(name)), (0, _utilHtml.td_left)(TYPE_TO_ICON[value.variant] || value.variant), (0, _utilHtml.td_right)((0, _utilTime.format_uptime)((0, _utilTime.till_now_ms)((0, _utilTime.from_ms)(time))))]), (0, _utilHtml.tag_class)('tr', 'bg-info', { tag: 'td', attrs: { colspan: 100 }, children: [(0, _utilHtml.icon)('arrow-up'), ' ', value[1]] })] };
	                }
	            } else {
	                return (0, _utilHtml.tag)('tr', [(0, _utilHtml.td_left)(JSON.stringify(name)), (0, _utilHtml.td_left)(TYPE_TO_ICON[value.variant] || value.variant), (0, _utilHtml.td_right)(value.toString())]);
	            }
	        }
	    }, {
	        key: 'render_process',
	        value: function render_process(item) {
	            return (0, _utilHtml.tag_class)("div", "col-xs-12", [(0, _utilHtml.tag)("h2", item.pid + ' ' + item.process.name), (0, _utilHtml.tag)("p", item.process.cmdline.split('\u0000').join(' ')), (0, _utilHtml.tag_class)("table", "table table-hover", [(0, _utilHtml.tag)("thead", (0, _utilHtml.tag)("tr", [(0, _utilHtml.th_left)('name'), (0, _utilHtml.th_left)((0, _utilHtml.icon)('asterisk')), (0, _utilHtml.th_right)('value')])), (0, _utilHtml.tag)("tbody", item.values.map(this.render_value.bind(this)))])]);
	        }
	    }]);

	    return Values;
	})(_utilBase.Component);

	exports.Values = Values;

/***/ },
/* 25 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _slicedToArray = (function () { function sliceIterator(arr, i) { var _arr = []; var _n = true; var _d = false; var _e = undefined; try { for (var _i = arr[Symbol.iterator](), _s; !(_n = (_s = _i.next()).done); _n = true) { _arr.push(_s.value); if (i && _arr.length === i) break; } } catch (err) { _d = true; _e = err; } finally { try { if (!_n && _i['return']) _i['return'](); } finally { if (_d) throw _e; } } return _arr; } return function (arr, i) { if (Array.isArray(arr)) { return arr; } else if (Symbol.iterator in Object(arr)) { return sliceIterator(arr, i); } else { throw new TypeError('Invalid attempt to destructure non-iterable instance'); } }; })();

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	var _get = function get(_x, _x2, _x3) { var _again = true; _function: while (_again) { var object = _x, property = _x2, receiver = _x3; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x = parent; _x2 = property; _x3 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	var _utilHtml = __webpack_require__(6);

	var _utilTime = __webpack_require__(7);

	var _utilBase = __webpack_require__(1);

	var _utilDonut = __webpack_require__(18);

	var _utilRequest = __webpack_require__(8);

	var COLORS = ["#4D4D4D", // (gray)
	"#5DA5DA", // (blue)
	"#FAA43A", // (orange)
	"#60BD68", // (green)
	"#F17CB0", // (pink)
	"#B2912F", // (brown)
	"#B276B2", // (purple)
	"#DECF3F", // (yellow)
	"#F15854"];

	// (red)

	var StateText = (function (_Component) {
	    _inherits(StateText, _Component);

	    function StateText() {
	        _classCallCheck(this, StateText);

	        _get(Object.getPrototypeOf(StateText.prototype), 'constructor', this).apply(this, arguments);
	    }

	    _createClass(StateText, [{
	        key: 'init',
	        value: function init(title, value) {
	            this.title = title;
	            this.value = value;
	        }
	    }, {
	        key: 'render',
	        value: function render() {
	            return { children: [(0, _utilHtml.tag)('h2', this.title), (0, _utilHtml.tag)('p', this.value)] };
	        }
	    }]);

	    return StateText;
	})(_utilBase.Component);

	function aggregate(data) {
	    var start = new Date();
	    var states = {};
	    var _iteratorNormalCompletion = true;
	    var _didIteratorError = false;
	    var _iteratorError = undefined;

	    try {
	        for (var _iterator = data.latest[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
	            var item = _step.value;

	            var _item = _slicedToArray(item, 2);

	            var dim = _item[0];
	            var metric = _item[1];

	            if (dim.state && dim.state.indexOf('.') > 0) {
	                var stchunks = dim.state.split('.');
	                var sub = stchunks.pop();
	                var stname = stchunks.join('.');
	                var st = states[stname];
	                if (!st) {
	                    states[stname] = st = {
	                        counters: {},
	                        durations: {},
	                        states: {}
	                    };
	                }
	                if (dim.metric == 'count') {
	                    st.counters[sub] = (st.counters[sub] || 0) + metric;
	                } else if (dim.metric == 'duration') {
	                    st.durations[sub] = (st.durations[sub] || 0) + metric;
	                }
	            }
	            if (dim.state && metric.length !== undefined && metric[0] != 0) {
	                var st = states[dim.state];
	                if (!st) {
	                    states[dim.state] = st = {
	                        counters: {},
	                        durations: {},
	                        states: {}
	                    };
	                }
	                var state = metric[1];
	                st.states[state] = (st.states[state] || 0) + 1;
	                st.durations[state] = (st.durations[state] || 0) + (0, _utilTime.till_now_ms)((0, _utilTime.from_ms)(metric[0]));
	            }
	        }
	    } catch (err) {
	        _didIteratorError = true;
	        _iteratorError = err;
	    } finally {
	        try {
	            if (!_iteratorNormalCompletion && _iterator['return']) {
	                _iterator['return']();
	            }
	        } finally {
	            if (_didIteratorError) {
	                throw _iteratorError;
	            }
	        }
	    }

	    var charts = [];
	    for (var name in states) {
	        var state = states[name];
	        var keys = Object.keys(state.durations);
	        if (keys.length > 1) {
	            var items = [];
	            var total = 0;
	            var dur = states[name].durations;
	            var colors = COLORS.concat();
	            for (var k in dur) {
	                var val = dur[k];
	                items.push({
	                    'title': k,
	                    value: dur[k],
	                    color: colors.pop()
	                });
	                total += val;
	            }
	            var chart = { total: total, items: items, title: name, unit: 'ms' };
	            charts.push(chart);
	        } else {
	            charts.push({ title: name, text: keys[0] });
	        }
	    }
	    charts.sort(function (a, b) {
	        return a.title.localeCompare(b.title);
	    });

	    return charts;
	}

	var Totals = (function (_Component2) {
	    _inherits(Totals, _Component2);

	    function Totals() {
	        _classCallCheck(this, Totals);

	        _get(Object.getPrototypeOf(Totals.prototype), 'constructor', this).call(this);
	        this.charts = [];
	    }

	    _createClass(Totals, [{
	        key: 'init',
	        value: function init() {
	            this.guard('json', new _utilRequest.RefreshJson("/states.json")).process(function (data, latency) {
	                var error = null;
	                if (data instanceof Error) {
	                    error = data;
	                }
	                return { error: error, latency: latency, charts: aggregate(data) };
	            });
	        }
	    }, {
	        key: 'render',
	        value: function render() {
	            return (0, _utilHtml.tag_class)("div", "container", [(0, _utilHtml.tag)("h1", ["States"]), (0, _utilHtml.tag_class)("div", "text-right", this.error ? 'Error: ' + this.error.message : 'Fetched in ' + this.latency + 'ms')].concat(this.charts.map(function (item) {
	                if (item.hasOwnProperty('text')) {
	                    return (0, _utilBase.component)(StateText, item.title, item.text);
	                } else {
	                    return (0, _utilBase.component)(Chart, (0, _utilBase.component)(_utilDonut.DonutChart, item), item);
	                }
	            })));
	        }
	    }]);

	    return Totals;
	})(_utilBase.Component);

	exports.Totals = Totals;

/***/ },
/* 26 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	var _get = function get(_x, _x2, _x3) { var _again = true; _function: while (_again) { var object = _x, property = _x2, receiver = _x3; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x = parent; _x2 = property; _x3 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { 'default': obj }; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	var _utilBase = __webpack_require__(1);

	var _utilRequest = __webpack_require__(8);

	var _templatesMetricsMft = __webpack_require__(27);

	var _templatesMetricsMft2 = _interopRequireDefault(_templatesMetricsMft);

	var Metrics = (function (_Component) {
	    _inherits(Metrics, _Component);

	    function Metrics() {
	        _classCallCheck(this, Metrics);

	        _get(Object.getPrototypeOf(Metrics.prototype), 'constructor', this).apply(this, arguments);
	    }

	    _createClass(Metrics, [{
	        key: 'init',
	        value: function init() {}
	    }, {
	        key: 'render',
	        value: function render() {
	            return _templatesMetricsMft2['default'].render();
	        }
	    }]);

	    return Metrics;
	})(_utilBase.Component);

	exports.Metrics = Metrics;

/***/ },
/* 27 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(22),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_query) {
	        var MetricsQuery = _mod_util_query.MetricsQuery;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(".b-metrics.date {\n    font-size: xx-small;\n    float: right;\n    position: absolute;\n    right: 4px;\n    bottom: 0px;\n}\n\npre.b-metrics {\n    position: relative;\n}\n\n"))
	        document.head.appendChild(_style)
	        function render() {
	            return {
	                    key: "b-metrics:render",
	                    children: function (old_node) {
	                        var query = old_node && old_node.store_query || new MetricsQuery();
	                        return {
	                                tag: "div",
	                                store_query: query,
	                                attrs: {class: "b-metrics container"},
	                                children: {children: [
	                                    {
	                                        tag: "h1",
	                                        children: {children: [
	                                            "All metrics",
	                                            ((query.metrics)?(" (" + String(query.metrics.length) + ")"):("")),
	                                        ]},
	                                    },
	                                    {
	                                        tag: "div",
	                                        children: ((query.metrics)?(query.metrics.map(function (m) {
	                                            return {
	                                                    tag: "pre",
	                                                    attrs: {class: "b-metrics "},
	                                                    children: {children: [
	                                                        {
	                                                            tag: "div",
	                                                            attrs: {class: "b-metrics date"},
	                                                            children: String(m[1]),
	                                                        },
	                                                        {
	                                                            tag: "code",
	                                                            children: JSON.stringify(m[0]),
	                                                        },
	                                                        {
	                                                            tag: "code",
	                                                            attrs: {style: {float: "right"}},
	                                                            children: String(m[2].constructor.name) + " (" + String(m[2].value) + ")",
	                                                        },
	                                                    ]},
	                                                    key: m,
	                                                };
	                                        })):("")),
	                                    },
	                                ]},
	                                events: {"$destroyed": ((query.owner_destroyed)?(query.owner_destroyed.handle_event):(function () {
	                                }))},
	                            };
	                    },
	                };
	        }
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 28 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	var _get = function get(_x, _x2, _x3) { var _again = true; _function: while (_again) { var object = _x, property = _x2, receiver = _x3; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x = parent; _x2 = property; _x3 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { 'default': obj }; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	var _utilBase = __webpack_require__(1);

	var _utilStreams = __webpack_require__(16);

	var _utilRequest = __webpack_require__(8);

	var _templatesPeersMft = __webpack_require__(29);

	var _templatesPeersMft2 = _interopRequireDefault(_templatesPeersMft);

	var Peers = (function (_Component) {
	    _inherits(Peers, _Component);

	    function Peers() {
	        _classCallCheck(this, Peers);

	        _get(Object.getPrototypeOf(Peers.prototype), 'constructor', this).call(this);
	        this.add_host = new _utilStreams.Stream("add_host");
	        this.add_host.handle(this.call_add_host.bind(this));
	    }

	    _createClass(Peers, [{
	        key: 'init',
	        value: function init() {
	            var _this = this;

	            this.guard('json', new _utilRequest.RefreshJson('/all_peers.json', { interval: 5000 })).process(function (data, latency) {
	                var error = null;
	                var peers = _this.peers;
	                if (data instanceof Error) {
	                    error = data;
	                } else {
	                    peers = data.peers;
	                }
	                console.log("Peers", peers);
	                return { error: error, peers: peers, latency: latency };
	            });
	        }
	    }, {
	        key: 'render',
	        value: function render() {
	            return _templatesPeersMft2['default'].render(this.peers, this);
	        }
	    }, {
	        key: 'call_add_host',
	        value: function call_add_host(value) {
	            this.last_add = { progress: true };
	            if (value.indexOf(':') < 0) {
	                value = value + ":" + (location.port || "22682");
	            }
	            this.guard('add_host', new _utilRequest.Submit('/add_host.json', {
	                'addr': value
	            })).process(function (data, latency) {
	                return { last_add: data instanceof Error ? { result: 'error', error: data } : { result: 'success' } };
	            });
	        }
	    }]);

	    return Peers;
	})(_utilBase.Component);

	exports.Peers = Peers;

/***/ },
/* 29 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(15),
	        __webpack_require__(7),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_stores, _mod_util_time) {
	        var Toggle = _mod_util_stores.Toggle;
	        var Store = _mod_util_stores.Store;
	        var Value = _mod_util_stores.Value;
	        var format_uptime = _mod_util_time.format_uptime;
	        var format_diff = _mod_util_time.format_diff;
	        var till_now_ms = _mod_util_time.till_now_ms;
	        var from_ms = _mod_util_time.from_ms;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(".b-peers.tiny {\n    font-size: xx-small;\n    color: gray;\n}\n\n.b-peers.tinyline {\n    font-size: xx-small;\n    color: gray;\n    white-space: nowrap;\n    overflow: hidden;\n    text-overflow: ellipsis;\n    max-width: 128px;\n}\n\n.b-peers.id {\n    white-space: nowrap;\n    overflow: hidden;\n    text-overflow: ellipsis;\n    max-width: 64px;\n}\n\n"))
	        document.head.appendChild(_style)
	        function millis_time(tm) {
	            return ((tm)?({
	                    key: "b-peers:millis_time" + ":0",
	                    children: function (old_node) {
	                        var delta = till_now_ms(from_ms(tm));
	                        return {
	                                tag: "span",
	                                children: ((delta < 10000)?(String(delta) + " ms"):(format_diff(delta))),
	                            };
	                    },
	                }):("never"));
	        }
	        function render(peers, ctr) {
	            return {
	                    key: "b-peers:render",
	                    children: function (old_node) {
	                        var toggle = old_node && old_node.store_toggle || new Toggle();
	                        return {
	                                tag: "div",
	                                store_toggle: toggle,
	                                attrs: {class: "b-peers container"},
	                                children: {children: [
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-peers row"},
	                                        children: {
	                                            tag: "div",
	                                            attrs: {class: "b-peers col-xs-12"},
	                                            children: {
	                                                tag: "h1",
	                                                children: {children: [
	                                                    "All peers ",
	                                                    ((peers)?("(" + String(peers.length) + ") "):("")),
	                                                    ((!toggle.visible)?({
	                                                        tag: "button",
	                                                        attrs: {class: "b-peers btn btn-default"},
	                                                        children: {
	                                                            tag: "span",
	                                                            attrs: {class: "b-peers glyphicon glyphicon-plus"},
	                                                        },
	                                                        events: {click: toggle.toggle.handle_event},
	                                                    }):("")),
	                                                ]},
	                                            },
	                                        },
	                                    },
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-peers row"},
	                                        children: {
	                                            tag: "p",
	                                            attrs: {class: "b-peers col-xs-12"},
	                                            children: ((ctr.last_add && ctr.last_add.progress)?("Adding ..."):(((toggle.visible)?({children: [
	                                                function (old_node) {
	                                                    var new_host = old_node && old_node.store_new_host || new Value();
	                                                    return {
	                                                            tag: "div",
	                                                            store_new_host: new_host,
	                                                            attrs: {class: "b-peers form-inline"},
	                                                            children: {children: [
	                                                                {
	                                                                    tag: "div",
	                                                                    attrs: {class: "b-peers form-group"},
	                                                                    children: {
	                                                                        tag: "button",
	                                                                        attrs: {class: "b-peers btn btn-default"},
	                                                                        children: {
	                                                                            tag: "span",
	                                                                            attrs: {class: "b-peers glyphicon glyphicon-chevron-up"},
	                                                                        },
	                                                                        events: {click: toggle.toggle.handle_event},
	                                                                    },
	                                                                },
	                                                                " ",
	                                                                {
	                                                                    tag: "div",
	                                                                    attrs: {class: "b-peers form-group"},
	                                                                    children: {children: [
	                                                                        {
	                                                                            tag: "label",
	                                                                            children: "IP",
	                                                                        },
	                                                                        " ",
	                                                                        function (old_node) {
	                                                                            var _stream_0 = new_host;
	                                                                            return {
	                                                                                    tag: "input",
	                                                                                    attrs: {
	                                                                                        type: "ip",
	                                                                                        class: "b-peers form-control",
	                                                                                    },
	                                                                                    children: {children: []},
	                                                                                    events: {
	                                                                                        keyup: _stream_0.keyup.handle_event,
	                                                                                        change: _stream_0.change.handle_event,
	                                                                                    },
	                                                                                };
	                                                                        },
	                                                                    ]},
	                                                                },
	                                                                " ",
	                                                                {
	                                                                    tag: "div",
	                                                                    attrs: {class: "b-peers form-group"},
	                                                                    children: {
	                                                                        tag: "button",
	                                                                        attrs: {class: "b-peers btn btn-default"},
	                                                                        children: "Add",
	                                                                        events: {click: ctr.add_host.map(function (ev) {
	                                                                            return new_host.value;
	                                                                        }).handle_event},
	                                                                    },
	                                                                },
	                                                            ]},
	                                                            events: {"$destroyed": ((new_host.owner_destroyed)?(new_host.owner_destroyed.handle_event):(function () {
	                                                            }))},
	                                                        };
	                                                },
	                                                ((ctr.last_add)?(((ctr.last_add.result === "success")?({
	                                                    tag: "p",
	                                                    children: "Successfully added",
	                                                }):({
	                                                    tag: "p",
	                                                    attrs: {class: "b-peers text-warning"},
	                                                    children: "Error adding host: " + String(ctr.last_add.error),
	                                                }))):("")),
	                                            ]}):("")))),
	                                        },
	                                    },
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-peers row"},
	                                        children: {
	                                            tag: "p",
	                                            attrs: {class: "b-peers col-xs-12"},
	                                            children: ((peers)?(((peers.length > 0)?({
	                                                tag: "table",
	                                                attrs: {class: "b-peers table table-bordered table-hover"},
	                                                children: {children: [
	                                                    {
	                                                        tag: "thead",
	                                                        children: {
	                                                            tag: "tr",
	                                                            children: {children: [
	                                                                {
	                                                                    tag: "th",
	                                                                    children: "ID",
	                                                                },
	                                                                {
	                                                                    tag: "th",
	                                                                    children: "IP",
	                                                                },
	                                                                {
	                                                                    tag: "th",
	                                                                    children: "Name",
	                                                                },
	                                                                {
	                                                                    tag: "th",
	                                                                    children: "Last Probe",
	                                                                },
	                                                                {
	                                                                    tag: "th",
	                                                                    children: "Last Report",
	                                                                },
	                                                                {
	                                                                    tag: "th",
	                                                                    children: "Roundtrip",
	                                                                },
	                                                                {
	                                                                    tag: "th",
	                                                                    children: "Peers",
	                                                                },
	                                                                {
	                                                                    tag: "th",
	                                                                    children: "Remote?",
	                                                                },
	                                                            ]},
	                                                        },
	                                                    },
	                                                    {
	                                                        tag: "tbody",
	                                                        children: peers.map(function (p) {
	                                                            return {
	                                                                    tag: "tr",
	                                                                    children: {children: [
	                                                                        {
	                                                                            tag: "td",
	                                                                            children: {children: [
	                                                                                {
	                                                                                    tag: "div",
	                                                                                    attrs: {class: "b-peers id"},
	                                                                                    children: String(p.id),
	                                                                                },
	                                                                                {
	                                                                                    tag: "div",
	                                                                                    attrs: {class: "b-peers tiny"},
	                                                                                    children: String(p.id),
	                                                                                },
	                                                                            ]},
	                                                                        },
	                                                                        {
	                                                                            tag: "td",
	                                                                            attrs: {class: "b-peers" + " " + ((p.name === null)?("text-muted"):(""))},
	                                                                            children: {children: [
	                                                                                ((p.primary_addr)?(String(p.primary_addr)):("to-be-determined")),
	                                                                                {
	                                                                                    tag: "div",
	                                                                                    attrs: {class: "b-peers tinyline"},
	                                                                                    children: "total: " + String(p.addresses.length),
	                                                                                },
	                                                                            ]},
	                                                                        },
	                                                                        function (old_node) {
	                                                                            var paddr = p.primary_addr || p.addresses[0] || ":22682";
	                                                                            var port = paddr.split(":")[1];
	                                                                            return {
	                                                                                    tag: "td",
	                                                                                    attrs: {class: "b-peers" + " " + ((p.name === null)?("text-muted"):(""))},
	                                                                                    children: {children: [
	                                                                                        ((p.name)?(String(p.name)):("unknown")),
	                                                                                        " ",
	                                                                                        {
	                                                                                            tag: "a",
	                                                                                            attrs: {
	                                                                                                href: "http://" + String(p.name) + ":" + String(port),
	                                                                                                class: "b-peers btn btn-default btn-xs",
	                                                                                            },
	                                                                                            children: {
	                                                                                                tag: "span",
	                                                                                                attrs: {class: "b-peers glyphicon glyphicon-play"},
	                                                                                            },
	                                                                                        },
	                                                                                        {
	                                                                                            tag: "div",
	                                                                                            attrs: {class: "b-peers tiny"},
	                                                                                            children: ((p.hostname)?(String(p.hostname)):("unknown")),
	                                                                                        },
	                                                                                    ]},
	                                                                                };
	                                                                        },
	                                                                        {
	                                                                            tag: "td",
	                                                                            attrs: {class: "b-peers" + " " + ((p.probe_time === null)?("text-muted"):(""))},
	                                                                            children: {children: [
	                                                                                millis_time(p.probe_time),
	                                                                                {
	                                                                                    tag: "div",
	                                                                                    attrs: {class: "b-peers tiny"},
	                                                                                    children: String(p.probes_sent),
	                                                                                },
	                                                                            ]},
	                                                                        },
	                                                                        {
	                                                                            tag: "td",
	                                                                            attrs: {class: "b-peers" + " " + ((p.last_report_direct === null)?("text-muted"):(""))},
	                                                                            children: {children: [
	                                                                                millis_time(p.last_report_direct),
	                                                                                {
	                                                                                    tag: "div",
	                                                                                    attrs: {class: "b-peers tiny"},
	                                                                                    children: String(p.pings_received) + "/" + String(p.pongs_received),
	                                                                                },
	                                                                            ]},
	                                                                        },
	                                                                        {
	                                                                            tag: "td",
	                                                                            attrs: {class: "b-peers" + " " + ((p.roundtrip === null)?("text-muted"):(""))},
	                                                                            children: ((p.roundtrip !== null)?({children: [
	                                                                                String(p.roundtrip) + "ms",
	                                                                                {
	                                                                                    tag: "div",
	                                                                                    attrs: {class: "b-peers tiny"},
	                                                                                    children: millis_time(p.random_peer_roundtrip),
	                                                                                },
	                                                                            ]}):("∅")),
	                                                                        },
	                                                                        {
	                                                                            tag: "td",
	                                                                            attrs: {class: "b-peers" + " " + ((p.peers === null)?("text-muted"):(""))},
	                                                                            children: ((p.peers !== null)?(String(p.peers)):("∅")),
	                                                                        },
	                                                                        {
	                                                                            tag: "td",
	                                                                            attrs: {class: "b-peers" + " " + ((p.num_remote === null)?("text-muted"):(""))},
	                                                                            children: ((p.has_remote === null)?("∅"):(((p.has_remote)?({
	                                                                                tag: "span",
	                                                                                attrs: {class: "b-peers glyphicon glyphicon-ok"},
	                                                                            }):({
	                                                                                tag: "span",
	                                                                                attrs: {class: "b-peers glyphicon glyphicon-remove"},
	                                                                            })))),
	                                                                        },
	                                                                    ]},
	                                                                    key: p.id,
	                                                                };
	                                                        }),
	                                                    },
	                                                ]},
	                                            }):({
	                                                tag: "div",
	                                                attrs: {class: "b-peers panel panel-warning"},
	                                                children: {children: [
	                                                    {
	                                                        tag: "div",
	                                                        attrs: {class: "b-peers panel-heading"},
	                                                        children: "No known peers ☹",
	                                                    },
	                                                    {
	                                                        tag: "div",
	                                                        attrs: {class: "b-peers panel-body"},
	                                                        children: "\n                You must add first peer by yourself\n                (or some other node might find you too)\n                ",
	                                                    },
	                                                ]},
	                                            }))):("")),
	                                        },
	                                    },
	                                ]},
	                                events: {"$destroyed": ((toggle.owner_destroyed)?(toggle.owner_destroyed.handle_event):(function () {
	                                }))},
	                            };
	                    },
	                };
	        }
	        exports.millis_time = millis_time
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 30 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	var _get = function get(_x, _x2, _x3) { var _again = true; _function: while (_again) { var object = _x, property = _x2, receiver = _x3; desc = parent = getter = undefined; _again = false; if (object === null) object = Function.prototype; var desc = Object.getOwnPropertyDescriptor(object, property); if (desc === undefined) { var parent = Object.getPrototypeOf(object); if (parent === null) { return undefined; } else { _x = parent; _x2 = property; _x3 = receiver; _again = true; continue _function; } } else if ('value' in desc) { return desc.value; } else { var getter = desc.get; if (getter === undefined) { return undefined; } return getter.call(receiver); } } };

	exports.get_port = get_port;

	function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { 'default': obj }; }

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	function _inherits(subClass, superClass) { if (typeof superClass !== 'function' && superClass !== null) { throw new TypeError('Super expression must either be null or a function, not ' + typeof superClass); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, enumerable: false, writable: true, configurable: true } }); if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass; }

	var _utilBase = __webpack_require__(1);

	var _utilStreams = __webpack_require__(16);

	var _utilRequest = __webpack_require__(8);

	var _templatesRemoteMft = __webpack_require__(31);

	var _templatesRemoteMft2 = _interopRequireDefault(_templatesRemoteMft);

	var _utilWebsock = __webpack_require__(3);

	var _utilWebsock2 = _interopRequireDefault(_utilWebsock);

	function get_port(p) {
	    var paddr = p.primary_addr || p.addresses && p.addresses[0] || ':22682';
	    var port = paddr.split(':')[1];
	    return port;
	}

	var Remote = (function (_Component) {
	    _inherits(Remote, _Component);

	    function Remote() {
	        _classCallCheck(this, Remote);

	        _get(Object.getPrototypeOf(Remote.prototype), 'constructor', this).call(this);
	        this.enable_remote = new _utilStreams.Stream("enable_remote");
	        this.enable_remote.handle(this.call_enable_remote.bind(this));
	        this.peer_map = {};
	    }

	    _createClass(Remote, [{
	        key: 'call_enable_remote',
	        value: function call_enable_remote(value) {
	            this.guard('add_host', new _utilRequest.Submit('/start_remote.json', '')).process(function (data, latency) {});
	        }
	    }, {
	        key: 'init',
	        value: function init() {
	            var _this = this;

	            this.guard('json', new _utilRequest.RefreshJson('/all_peers.json', { interval: 60000 })).process(function (data, latency) {
	                var error = null;
	                var peer_map = {};
	                if (data instanceof Error) {
	                    error = data;
	                    peer_map = _this.peer_map;
	                } else {
	                    var _iteratorNormalCompletion = true;
	                    var _didIteratorError = false;
	                    var _iteratorError = undefined;

	                    try {
	                        for (var _iterator = data.peers[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
	                            var peer = _step.value;

	                            var _name = peer.name || peer.hostname || peer.addr.split(':')[0];
	                            peer_map[peer.id] = _name;
	                        }
	                    } catch (err) {
	                        _didIteratorError = true;
	                        _iteratorError = err;
	                    } finally {
	                        try {
	                            if (!_iteratorNormalCompletion && _iterator['return']) {
	                                _iterator['return']();
	                            }
	                        } finally {
	                            if (_didIteratorError) {
	                                throw _iteratorError;
	                            }
	                        }
	                    }
	                }
	                return { error: error, enabled: data.enabled, peer_map: peer_map, latency: latency };
	            });
	        }
	    }, {
	        key: 'render',
	        value: function render() {
	            return _templatesRemoteMft2['default'].render(_utilWebsock2['default'].remote_enabled(), this.peer_map, this);
	        }
	    }]);

	    return Remote;
	})(_utilBase.Component);

	exports.Remote = Remote;

/***/ },
/* 31 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(32),
	        __webpack_require__(33),
	        __webpack_require__(34),
	        __webpack_require__(36),
	        __webpack_require__(22),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, disabled, list, grid, _mod_util_routing, _mod_util_query) {
	        var Hier = _mod_util_routing.Hier;
	        var RemoteStats = _mod_util_query.RemoteStats;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(""))
	        document.head.appendChild(_style)
	        function render(enabled, peer_map, ctr) {
	            return {
	                    key: "b-remote:render",
	                    children: function (old_node) {
	                        var page = old_node && old_node.store_page || new Hier(1, "list");
	                        return {
	                                tag: "div",
	                                store_page: page,
	                                attrs: {class: "b-remote container"},
	                                children: ((enabled)?({children: [
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-remote row"},
	                                        children: {
	                                            tag: "div",
	                                            attrs: {class: "b-remote col-xs-12"},
	                                            children: {
	                                                tag: "ul",
	                                                attrs: {class: "b-remote nav nav-pills"},
	                                                children: {children: [
	                                                    {
	                                                        tag: "li",
	                                                        attrs: {class: "b-remote" + " " + ((page.value === "list")?("active"):(""))},
	                                                        children: {
	                                                            tag: "a",
	                                                            attrs: {href: "#/remote/list"},
	                                                            children: "Peer List",
	                                                        },
	                                                    },
	                                                    {
	                                                        tag: "li",
	                                                        attrs: {class: "b-remote" + " " + ((page.value === "grid")?("active"):(""))},
	                                                        children: {
	                                                            tag: "a",
	                                                            attrs: {href: "#/remote/grid"},
	                                                            children: "Grid",
	                                                        },
	                                                    },
	                                                ]},
	                                            },
	                                        },
	                                    },
	                                    ((page.value === "list")?({
	                                        tag: "div",
	                                        attrs: {
	                                            style: {"margin-top": "1em"},
	                                            class: "b-remote row",
	                                        },
	                                        children: function (old_node) {
	                                            var stats_q = old_node && old_node.store_stats_q || new RemoteStats();
	                                            return {
	                                                    tag: "p",
	                                                    store_stats_q: stats_q,
	                                                    attrs: {class: "b-remote col-xs-12"},
	                                                    children: ((stats_q.response)?(((stats_q.response.peers.length > 0)?(list.render(stats_q.response.peers, peer_map)):({
	                                                        tag: "div",
	                                                        attrs: {class: "b-remote panel panel-warning"},
	                                                        children: {children: [
	                                                            {
	                                                                tag: "div",
	                                                                attrs: {class: "b-remote panel-heading"},
	                                                                children: "No known peers ☹",
	                                                            },
	                                                            {
	                                                                tag: "div",
	                                                                attrs: {class: "b-remote panel-body"},
	                                                                children: "\n                    You must add first peer by yourself in the Peers tab\n                    (or some other node might find you too)\n                    ",
	                                                            },
	                                                        ]},
	                                                    }))):("")),
	                                                    events: {"$destroyed": ((stats_q.owner_destroyed)?(stats_q.owner_destroyed.handle_event):(function () {
	                                                    }))},
	                                                };
	                                        },
	                                    }):(((page.value === "grid")?({
	                                        tag: "div",
	                                        attrs: {
	                                            style: {"margin-top": "1em"},
	                                            class: "b-remote row",
	                                        },
	                                        children: {
	                                            tag: "p",
	                                            attrs: {class: "b-remote col-xs-12"},
	                                            children: grid.render(peer_map),
	                                        },
	                                    }):("")))),
	                                ]}):(((enabled === undefined)?({
	                                    tag: "div",
	                                    attrs: {class: "b-remote row"},
	                                    children: {
	                                        tag: "div",
	                                        attrs: {class: "b-remote col-xs-12"},
	                                        children: "Loading ...",
	                                    },
	                                }):({
	                                    tag: "div",
	                                    attrs: {class: "b-remote row"},
	                                    children: {
	                                        tag: "div",
	                                        attrs: {class: "b-remote col-xs-12"},
	                                        children: disabled.render(ctr.enable_remote),
	                                    },
	                                })))),
	                                events: {"$destroyed": ((page.owner_destroyed)?(page.owner_destroyed.handle_event):(function () {
	                                }))},
	                            };
	                    },
	                };
	        }
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 32 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(22),
	        __webpack_require__(30),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_query, _mod_pages_remote) {
	        var PeersRequest = _mod_util_query.PeersRequest;
	        var get_port = _mod_pages_remote.get_port;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(""))
	        document.head.appendChild(_style)
	        function render(enable_remote) {
	            return {
	                    key: "b-disabled:render",
	                    children: [
	                        {
	                            tag: "h1",
	                            children: "Remote Stats Are Disabled",
	                        },
	                        {
	                            tag: "div",
	                            attrs: {class: "b-disabled panel panel-warning"},
	                            children: {children: [
	                                {
	                                    tag: "div",
	                                    attrs: {class: "b-disabled panel-heading"},
	                                    children: "Enable remote metrics",
	                                },
	                                function (old_node) {
	                                    var peers_request = old_node && old_node.store_peers_request || new PeersRequest(true, 20000);
	                                    return {
	                                            tag: "div",
	                                            store_peers_request: peers_request,
	                                            attrs: {class: "b-disabled panel-body"},
	                                            children: {children: [
	                                                {
	                                                    tag: "p",
	                                                    children: "\n        You may enable remote metrics. But be aware that it means this\n        node will use a little bit more resources.\n        ",
	                                                },
	                                                {
	                                                    tag: "p",
	                                                    children: {children: [
	                                                        " But more importantly, if you enable remote metrics on all (or\n          too many nodes) you will get ",
	                                                        {
	                                                            tag: "b",
	                                                            children: "full mesh",
	                                                        },
	                                                        " of connections and a lot of traffic. So chose chose nodes\n          wisely.",
	                                                    ]},
	                                                },
	                                                {
	                                                    tag: "p",
	                                                    children: {children: [
	                                                        " You might want to ",
	                                                        {
	                                                            tag: "b",
	                                                            children: "find a node",
	                                                        },
	                                                        " which has remote stats enabled\n          instead of enabling them here.\n        ",
	                                                    ]},
	                                                },
	                                                ((!peers_request.peers)?({
	                                                    tag: "p",
	                                                    children: "Loading node list...",
	                                                }):(((peers_request.peers.length === 0)?({
	                                                    tag: "p",
	                                                    children: {
	                                                        tag: "i",
	                                                        children: "Unfortunately no nodes are known to have remote tab enabled",
	                                                    },
	                                                }):({
	                                                    tag: "ul",
	                                                    children: peers_request.peers.map(function (peer) {
	                                                        return ((peer.name)?({
	                                                                key: peer + ":0",
	                                                                children: function (old_node) {
	                                                                    var port = get_port(peer);
	                                                                    return {
	                                                                            tag: "li",
	                                                                            children: {children: [
	                                                                                {
	                                                                                    tag: "a",
	                                                                                    attrs: {href: "http://" + String(peer.name) + ":" + String(port) + "/"},
	                                                                                    children: "http://" + String(peer.name) + ":" + String(port) + "/",
	                                                                                },
	                                                                                " (" + String(peer.primary_addr) + ")",
	                                                                            ]},
	                                                                        };
	                                                                },
	                                                            }):(""));
	                                                    }),
	                                                })))),
	                                                {
	                                                    tag: "p",
	                                                    attrs: {class: "b-disabled" + " " + ((!peers_request.peers || peers_request.peers.length !== 0)?("text-right"):(""))},
	                                                    children: {
	                                                        tag: "button",
	                                                        attrs: {class: "b-disabled btn btn-danger"},
	                                                        children: "Enable",
	                                                        events: {click: enable_remote.handle_event},
	                                                    },
	                                                },
	                                            ]},
	                                            events: {"$destroyed": ((peers_request.owner_destroyed)?(peers_request.owner_destroyed.handle_event):(function () {
	                                            }))},
	                                        };
	                                },
	                            ]},
	                        },
	                    ],
	                };
	        }
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 33 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(15),
	        __webpack_require__(7),
	        __webpack_require__(30),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_stores, _mod_util_time, _mod_pages_remote) {
	        var Toggle = _mod_util_stores.Toggle;
	        var Store = _mod_util_stores.Store;
	        var Value = _mod_util_stores.Value;
	        var format_diff = _mod_util_time.format_diff;
	        var till_now_ms = _mod_util_time.till_now_ms;
	        var from_ms = _mod_util_time.from_ms;
	        var get_port = _mod_pages_remote.get_port;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode("pre.short.b-list {\n    height: 1em;\n    overflow: hide;\n}\n\n.b-list.corner {\n    position: absolute;\n    right: 8px;\n    top: 8px;\n}\n\n.b-list.beacon {\n    display: inline-block;\n    margin-right: 8px;\n}\n\n.b-list.relative {\n    position: relative;\n}\n\n"))
	        document.head.appendChild(_style)
	        function render(peers, peer_map) {
	            return {
	                    tag: "table",
	                    attrs: {class: "b-list table table-bordered table-hover"},
	                    children: {children: [
	                        {
	                            tag: "thead",
	                            children: {
	                                tag: "tr",
	                                children: {children: [
	                                    {
	                                        tag: "th",
	                                        children: "ID",
	                                    },
	                                    {
	                                        tag: "th",
	                                        children: "IP",
	                                    },
	                                    {
	                                        tag: "th",
	                                        children: "Name",
	                                    },
	                                    {
	                                        tag: "th",
	                                        children: "Connected",
	                                    },
	                                    {
	                                        tag: "th",
	                                        children: "Last Beacon",
	                                    },
	                                    {
	                                        tag: "th",
	                                        children: "Beacon",
	                                    },
	                                    {
	                                        tag: "th",
	                                        children: "Latency",
	                                    },
	                                ]},
	                            },
	                        },
	                        {
	                            tag: "tbody",
	                            children: peers.map(function (p) {
	                                return {
	                                        tag: "tr",
	                                        children: {children: [
	                                            {
	                                                tag: "td",
	                                                children: String(p.id),
	                                            },
	                                            {
	                                                tag: "td",
	                                                attrs: {class: "b-list" + " " + ((p.current_addr)?("text-muted"):(""))},
	                                                children: ((p.current_addr)?(String(p.current_addr)):("∅")),
	                                            },
	                                            ((peer_map[p.id])?(function (old_node) {
	                                                var port = get_port(p);
	                                                return {
	                                                        tag: "td",
	                                                        children: {children: [
	                                                            String(peer_map[p.id]) + " ",
	                                                            {
	                                                                tag: "a",
	                                                                attrs: {
	                                                                    href: "http://" + String(peer_map[p.id]) + ":" + String(port),
	                                                                    class: "b-list btn btn-default btn-xs",
	                                                                },
	                                                                children: {
	                                                                    tag: "span",
	                                                                    attrs: {class: "b-list glyphicon glyphicon-play"},
	                                                                },
	                                                            },
	                                                        ]},
	                                                    };
	                                            }):({
	                                                tag: "td",
	                                                attrs: {class: "b-list text-muted"},
	                                                children: "∅",
	                                            })),
	                                            {
	                                                tag: "td",
	                                                children: String(p.connected),
	                                            },
	                                            {
	                                                tag: "td",
	                                                attrs: {class: "b-list" + " " + ((p.last_beacon_time === null)?("text-muted"):(""))},
	                                                children: ((p.last_beacon_time)?(function (old_node) {
	                                                    var val = p.last_beacon_time;
	                                                    return {
	                                                            tag: "span",
	                                                            children: ((val < 10000)?(String(val) + " ms"):(format_diff(till_now_ms(from_ms(p.last_beacon_time))))),
	                                                        };
	                                                }):("never")),
	                                            },
	                                            function (old_node) {
	                                                var toggle = old_node && old_node.store_toggle || new Toggle();
	                                                return {
	                                                        tag: "td",
	                                                        store_toggle: toggle,
	                                                        attrs: {class: "b-list relative" + " " + ((p.last_beacon === null)?("text-muted"):(""))},
	                                                        children: {children: [
	                                                            ((p.last_beacon)?(((toggle.visible)?({
	                                                                tag: "pre",
	                                                                children: JSON.stringify(p.last_beacon, null, "  "),
	                                                            }):({
	                                                                tag: "div",
	                                                                attrs: {class: "b-list beacon"},
	                                                                children: {children: [
	                                                                    "Values: " + String(p.last_beacon.values) + ", ",
	                                                                    "peers: " + String(p.last_beacon.peers),
	                                                                ]},
	                                                            }))):("∅")),
	                                                            {
	                                                                tag: "button",
	                                                                attrs: {class: "b-list float-right btn btn-default btn btn-xs" + " " + ((toggle.visible)?("corner"):(""))},
	                                                                children: ((toggle.visible)?({
	                                                                    tag: "span",
	                                                                    attrs: {class: "b-list glyphicon glyphicon-chevron-up"},
	                                                                }):({
	                                                                    tag: "span",
	                                                                    attrs: {class: "b-list glyphicon glyphicon-chevron-down"},
	                                                                })),
	                                                                events: {click: toggle.toggle.handle_event},
	                                                            },
	                                                        ]},
	                                                        events: {"$destroyed": ((toggle.owner_destroyed)?(toggle.owner_destroyed.handle_event):(function () {
	                                                        }))},
	                                                    };
	                                            },
	                                            {
	                                                tag: "td",
	                                                attrs: {class: "b-list" + " " + ((p.last_beacon === null)?("text-muted"):(""))},
	                                                children: ((p.last_beacon)?({children: [
	                                                    String(p.last_beacon_time - p.last_beacon.current_time),
	                                                    "ms",
	                                                ]}):("∅")),
	                                            },
	                                        ]},
	                                        key: p.id,
	                                    };
	                            }),
	                        },
	                    ]},
	                    key: "b-list:render",
	                };
	        }
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 34 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(22),
	        __webpack_require__(23),
	        __webpack_require__(17),
	        __webpack_require__(35),
	        __webpack_require__(21),
	        __webpack_require__(30),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_query, _mod_util_compute, donut, sparkline, _mod_util_format, _mod_pages_remote) {
	        var QueryRemote = _mod_util_query.QueryRemote;
	        var cpu_usage = _mod_util_compute.cpu_usage;
	        var mem_chart = _mod_util_compute.mem_chart;
	        var number_formatter = _mod_util_format.number_formatter;
	        var already_percent_formatter = _mod_util_format.already_percent_formatter;
	        var get_port = _mod_pages_remote.get_port;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(".b-grid.peer {\n    float: left;\n    display: inline-block;\n    width: 165px;\n    min-height: 160px;\n    text-align: center;\n    border: solid rgba(44 , 162 , 95 , 0.5) 4px;\n    border-radius: 8px;\n    margin: 2px;\n    position: relative;\n}\n\n.b-grid.latency {\n    float: right;\n}\n\n.b-grid.question {\n    font-size: 120px;\n}\n\n.b-grid.addr-row {\n    display: flex;\n    flex-direction: row;\n    padding: 2px;\n}\n\n.b-grid.addr {\n    font-family: Consolas , monospace;\n    overflow: hidden;\n    text-overflow: ellipsis;\n}\n\n.b-grid.addr-btn {\n    min-width: 24px;\n}\n\n.b-grid.donut-container, .b-grid.sparkline-container {\n    margin: 4px;\n}\n\n"))
	        document.head.appendChild(_style)
	        function render_peer(stats, name, port, cpu_yaxis) {
	            return ((stats && stats.get("cpu_idle").chunks.length)?({
	                    key: "b-grid:render_peer" + ":0",
	                    children: function (old_node) {
	                        var mem = mem_chart(stats.get("memory"));
	                        var usage = cpu_usage(stats.get("cpu_total").chunk, stats.get("cpu_idle").chunks[0][1]);
	                        return {
	                                tag: "div",
	                                attrs: {class: "b-grid peer"},
	                                children: {children: [
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-grid donut-container"},
	                                        children: donut.render(mem.items, 120, 120, mem.total),
	                                    },
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-grid sparkline-container"},
	                                        children: sparkline.render(150, stats.get("cpu_total").timestamps, [{
	                                            title: "Cpu",
	                                            values: usage,
	                                            yaxis: cpu_yaxis,
	                                        }]),
	                                    },
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-grid addr-row"},
	                                        children: {children: [
	                                            {
	                                                tag: "div",
	                                                attrs: {class: "b-grid addr"},
	                                                children: String(name) + " ",
	                                            },
	                                            ((name !== "myself")?({
	                                                tag: "a",
	                                                attrs: {
	                                                    href: "http://" + String(name) + ":" + String(port),
	                                                    class: "b-grid addr-btn btn btn-default btn-xs",
	                                                },
	                                                children: {
	                                                    tag: "span",
	                                                    attrs: {class: "b-grid glyphicon glyphicon-play"},
	                                                },
	                                            }):("")),
	                                        ]},
	                                    },
	                                ]},
	                            };
	                    },
	                }):({
	                    tag: "div",
	                    attrs: {class: "b-grid peer"},
	                    children: {children: [
	                        {
	                            tag: "div",
	                            attrs: {class: "b-grid question"},
	                            children: "?",
	                        },
	                        {
	                            tag: "div",
	                            attrs: {class: "b-grid addr-row"},
	                            children: {children: [
	                                {
	                                    tag: "div",
	                                    attrs: {class: "b-grid addr"},
	                                    children: String(name) + " ",
	                                },
	                                ((name !== "myself")?({
	                                    tag: "a",
	                                    attrs: {
	                                        href: "http://" + String(name) + ":" + String(port),
	                                        class: "b-grid btn btn-default btn-xs",
	                                    },
	                                    children: {
	                                        tag: "span",
	                                        attrs: {class: "b-grid glyphicon glyphicon-play"},
	                                    },
	                                }):("")),
	                            ]},
	                        },
	                    ]},
	                    key: "b-grid:render_peer" + ":1",
	                }));
	        }
	        function render(peer_map) {
	            return {
	                    key: "b-grid:render",
	                    children: function (old_node) {
	                        var grid_query = old_node && old_node.store_grid_query || new QueryRemote({
	                                memory: {
	                                    series: {
	                                        source: "Fine",
	                                        condition: [
	                                            "RegexLike",
	                                            "metric",
	                                            "^memory\.",
	                                        ],
	                                    },
	                                    extract: ["Tip"],
	                                    functions: [],
	                                },
	                                cpu_total: {
	                                    series: {
	                                        source: "Fine",
	                                        condition: [
	                                            "RegexLike",
	                                            "metric",
	                                            "^cpu\.",
	                                        ],
	                                    },
	                                    extract: [
	                                        "HistoryByNum",
	                                        150,
	                                    ],
	                                    functions: [
	                                        ["NonNegativeDerivative"],
	                                        [
	                                            "Sum",
	                                            "Ignore",
	                                        ],
	                                    ],
	                                },
	                                cpu_idle: {
	                                    series: {
	                                        source: "Fine",
	                                        condition: [
	                                            "Eq",
	                                            "metric",
	                                            "cpu.idle",
	                                        ],
	                                    },
	                                    extract: [
	                                        "HistoryByNum",
	                                        150,
	                                    ],
	                                    functions: [["NonNegativeDerivative"]],
	                                },
	                            });
	                        var cpu_yaxis = {
	                                height: 40,
	                                bg_color: "rgb(237,248,233)",
	                                skip_color: "white",
	                                format: already_percent_formatter(),
	                                colors: [
	                                    [
	                                        100,
	                                        "rgb(186,228,179)",
	                                    ],
	                                    [
	                                        200,
	                                        "rgb(116,196,118)",
	                                    ],
	                                    [
	                                        800,
	                                        "rgb(49,163,84)",
	                                    ],
	                                    [
	                                        1600,
	                                        "rgb(0,109,44)",
	                                    ],
	                                    [
	                                        6400,
	                                        "black",
	                                    ],
	                                ],
	                            };
	                        return {
	                                tag: "div",
	                                store_grid_query: grid_query,
	                                children: {children: [
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-grid latency"},
	                                        children: String(grid_query.latency) + "ms",
	                                    },
	                                    {
	                                        tag: "div",
	                                        children: Object.keys(peer_map).concat("myself").map(function (peer) {
	                                            return {
	                                                    key: peer,
	                                                    children: function (old_node) {
	                                                        var name = peer_map[peer] || peer.split(":")[0];
	                                                        var port = get_port(peer);
	                                                        return {
	                                                                tag: "div",
	                                                                children: render_peer(grid_query.response && grid_query.response.get(peer), name, port, cpu_yaxis),
	                                                            };
	                                                    },
	                                                };
	                                        }),
	                                    },
	                                ]},
	                                events: {"$destroyed": ((grid_query.owner_destroyed)?(grid_query.owner_destroyed.handle_event):(function () {
	                                }))},
	                            };
	                    },
	                };
	        }
	        exports.render_peer = render_peer
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 35 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(7),
	        __webpack_require__(14),
	        __webpack_require__(15),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_time, compact, _mod_util_stores) {
	        var format_datetime = _mod_util_time.format_datetime;
	        var Follow = _mod_util_stores.Follow;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(".b-sparkline.bar {\n    height: 41px;\n    position: relative;\n    display: inline-block;\n    vertical-align: middle;\n}\n\n.b-sparkline.title {\n    font-family: Verdana , Tahoma , sans-serif;\n    text-shadow: 0 1px 0 rgba(255 , 255 , 255 , 0.5);\n    position: absolute;\n    left: 12px;\n    top: 8px;\n    font-size: 18px;\n}\n\n.b-sparkline.value {\n    font-family: Verdana , Tahoma , sans-serif;\n    text-shadow: 0 1px 0 rgba(255 , 255 , 255 , 0.5);\n    font-size: 18px;\n    position: absolute;\n    right: 0px;\n    top: 0px;\n    padding-top: 8px;\n    padding-right: 8px;\n    height: 41px;\n}\n\n.b-sparkline.value.follow {\n    border-right: solid black 1px;\n}\n\n.b-sparkline.footer {\n    position: relative;\n}\n\n.b-sparkline.footer-time {\n    position: absolute;\n    right: 0px;\n    top: 0px;\n    padding-top: 8px;\n    padding-right: 8px;\n    height: 41px;\n}\n\nline.tick.b-sparkline {\n    stroke: black;\n}\n\ntext.tick.b-sparkline {\n    font-family: Verdana , Tahoma , sans-serif;\n    text-anchor: middle;\n    font-size: 12px;\n}\n\n"))
	        document.head.appendChild(_style)
	        function render(width, timestamps, items) {
	            return {
	                    key: "b-sparkline:render",
	                    children: function (old_node) {
	                        var mouse_position = old_node && old_node.store_mouse_position || new Follow();
	                        var _stream_1 = mouse_position;
	                        var xaxis = compact.xaxis(timestamps, width);
	                        return {
	                                tag: "span",
	                                store_mouse_position: mouse_position,
	                                children: {
	                                    tag: "span",
	                                    children: items.map(function (item) {
	                                        return ((item.values)?({
	                                                tag: "div",
	                                                attrs: {
	                                                    style: {width: "{width}px}"},
	                                                    class: "b-sparkline bar",
	                                                },
	                                                children: {children: [
	                                                    compact.draw(xaxis, item.yaxis, item.values),
	                                                    {
	                                                        tag: "div",
	                                                        attrs: {class: "b-sparkline title"},
	                                                        children: String(item.title),
	                                                    },
	                                                    ((mouse_position.x !== null && mouse_position.x < width)?(function (old_node) {
	                                                        var px = xaxis.pixels[mouse_position.x];
	                                                        return {
	                                                                tag: "div",
	                                                                attrs: {
	                                                                    style: {right: width - mouse_position.x + "px"},
	                                                                    class: "b-sparkline value follow",
	                                                                },
	                                                                children: ((px)?(((!isNaN(item.values[px.index]))?(((item.yaxis.format)?(String(item.yaxis.format(item.values[px.index]))):(String(item.values[px.index].toFixed(2))))):(""))):("--")),
	                                                            };
	                                                    }):({
	                                                        tag: "div",
	                                                        attrs: {class: "b-sparkline value"},
	                                                        children: ((!isNaN(item.values[0]))?(((item.yaxis.format)?(String(item.yaxis.format(item.values[0]))):(String(item.values[0].toFixed(2))))):("")),
	                                                    })),
	                                                ]},
	                                                key: item + ":0",
	                                            }):({
	                                                tag: "div",
	                                                attrs: {
	                                                    style: {width: "{width}px}"},
	                                                    class: "b-sparkline bar nodata",
	                                                },
	                                                children: "-- no data --",
	                                                key: item + ":1",
	                                            }));
	                                    }),
	                                },
	                                events: {
	                                    mouseleave: _stream_1.mouseleave.handle_event,
	                                    mouseenter: _stream_1.mouseenter.handle_event,
	                                    mousemove: _stream_1.mousemove.handle_event,
	                                    "$destroyed": ((mouse_position.owner_destroyed)?(mouse_position.owner_destroyed.handle_event):(function () {
	                                    })),
	                                },
	                            };
	                    },
	                };
	        }
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ },
/* 36 */
/***/ function(module, exports, __webpack_require__) {

	'use strict';

	Object.defineProperty(exports, '__esModule', {
	    value: true
	});

	var _slicedToArray = (function () { function sliceIterator(arr, i) { var _arr = []; var _n = true; var _d = false; var _e = undefined; try { for (var _i = arr[Symbol.iterator](), _s; !(_n = (_s = _i.next()).done); _n = true) { _arr.push(_s.value); if (i && _arr.length === i) break; } } catch (err) { _d = true; _e = err; } finally { try { if (!_n && _i['return']) _i['return'](); } finally { if (_d) throw _e; } } return _arr; } return function (arr, i) { if (Array.isArray(arr)) { return arr; } else if (Symbol.iterator in Object(arr)) { return sliceIterator(arr, i); } else { throw new TypeError('Invalid attempt to destructure non-iterable instance'); } }; })();

	var _createClass = (function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ('value' in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; })();

	exports.start = start;

	function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError('Cannot call a class as a function'); } }

	var _utilStreams = __webpack_require__(16);

	var router;

	var Router = (function () {
	    function Router(hash) {
	        _classCallCheck(this, Router);

	        this.page_stream = new _utilStreams.Stream('page_change');
	        this.chunk_streams = {};
	        var parsed = this.parse_hash(hash);
	        this.chunks = parsed.chunks;
	    }

	    _createClass(Router, [{
	        key: 'parse_hash',
	        value: function parse_hash(hash) {
	            var url = hash.substr(1);

	            var _url$split = url.split('?', 1);

	            var _url$split2 = _slicedToArray(_url$split, 2);

	            var path = _url$split2[0];
	            var query = _url$split2[1];

	            var chunks = path.split('/');
	            if (chunks[0] == '') {
	                chunks.shift();
	            }
	            return { chunks: chunks }; // TODO(tailhook) parse query
	        }
	    }, {
	        key: 'hash_change',
	        value: function hash_change(nhash) {
	            var nparams = this.parse_hash(nhash);
	            if (nparams.chunks[0] != this.page) {
	                this.page_stream.handle_event(nparams.chunks[0]);
	            }
	            for (var i = 1; i < nparams.chunks.length; ++i) {
	                if (nparams.chunks[i] != this.chunks[i]) {
	                    var stream = this.chunk_streams[i];
	                    if (stream) {
	                        stream.handle_event(nparams.chunks[i]);
	                    }
	                }
	            }
	            this.chunks = nparams.chunks;
	        }
	    }, {
	        key: 'set_chunk',
	        value: function set_chunk(level, value) {
	            // TODO(tailhook) add query
	            var chunks = this.chunks.splice(level, 0, value);
	            window.location.hash = '#/' + chunks.join('/');
	        }
	    }, {
	        key: 'page',
	        get: function get() {
	            return this.chunks[0];
	        }
	    }]);

	    return Router;
	})();

	var Hier = (function () {
	    function Hier(level, defvalue) {
	        _classCallCheck(this, Hier);

	        this.defvalue = defvalue;
	        console.assert(router, "Router must be initialized");
	        // TODO(tailhook) check for conflicts and remove the stream
	        //console.assert(!router.chunk_streams[level],
	        //               "Conflicting hierarchy router",
	        //               router.chunk_streams[level])
	        router.chunk_streams[level] = new _utilStreams.Stream("routing_hier_" + level);
	        router.chunk_streams[level].handle(this.new_value.bind(this));
	        this.value = router.chunks[level] || defvalue;
	    }

	    _createClass(Hier, [{
	        key: 'new_value',
	        value: function new_value(value) {
	            this.value = value || this.defvalue;
	        }
	    }, {
	        key: 'apply',
	        value: function apply(value) {
	            console.assert(router, "Router must be initialized");
	            router.set_chunk(level, value);
	        }
	    }]);

	    return Hier;
	})();

	exports.Hier = Hier;

	function start() {
	    router = new Router(window.location.hash);
	    window.onhashchange = function () {
	        return router.hash_change(window.location.hash);
	    };
	    return router;
	}

	exports['default'] = exports;

/***/ },
/* 37 */
/***/ function(module, exports, __webpack_require__) {

	var __WEBPACK_AMD_DEFINE_ARRAY__, __WEBPACK_AMD_DEFINE_RESULT__;!(__WEBPACK_AMD_DEFINE_ARRAY__ = [
	        __webpack_require__,
	        exports,
	        __webpack_require__(7),
	        __webpack_require__(1),
	        __webpack_require__(3),
	        __webpack_require__(22),
	        __webpack_require__(21),
	        __webpack_require__(35),
	        __webpack_require__(17),
	        __webpack_require__(23),
	    ], __WEBPACK_AMD_DEFINE_RESULT__ = function (require, exports, _mod_util_time, _mod_util_base, websock, _mod_util_query, _mod_util_format, sparkline, donut, _mod_util_compute) {
	        var format_uptime = _mod_util_time.format_uptime;
	        var till_now_ms = _mod_util_time.till_now_ms;
	        var from_ms = _mod_util_time.from_ms;
	        var component = _mod_util_base.component;
	        var Query = _mod_util_query.Query;
	        var already_percent_formatter = _mod_util_format.already_percent_formatter;
	        var bytes_formatter = _mod_util_format.bytes_formatter;
	        var percent_formatter = _mod_util_format.percent_formatter;
	        var cpu_usage = _mod_util_compute.cpu_usage;
	        var mem_chart = _mod_util_compute.mem_chart;
	        var _style = document.createElement("style");
	        _style.appendChild(document.createTextNode(".b-navbar.pointer {\n    cursor: pointer;\n}\n\n.b-navbar.donut, .b-navbar.sparkline {\n    display: inline-block;\n    margin-left: 2px;\n    margin-right: 2px;\n}\n\n.b-navbar.sparkline {\n    border: solid lightblue 2px;\n    border-radius: 3px;\n}\n\n.b-navbar.navbar-right {\n    margin-top: 2px;\n}\n\n.b-navbar.version_string {\n    font-size: x-small;\n    position: relative;\n    top: -4px;\n}\n\n.b-navbar.version {\n    display: inline;\n}\n\n"))
	        document.head.appendChild(_style)
	        function icon(name) {
	            return {
	                    tag: "span",
	                    attrs: {class: "b-navbar glyphicon" + " " + "glyphicon-" + String(name)},
	                    key: "b-navbar:icon",
	                };
	        }
	        function render_self(beacon, connected, cpu, mem) {
	            return {
	                    key: "b-navbar:render_self",
	                    children: [
	                        {
	                            tag: "span",
	                            attrs: {
	                                title: "Uptime of the cantal agent itself",
	                                class: "b-navbar pointer",
	                            },
	                            children: {children: [
	                                " up ",
	                                format_uptime(till_now_ms(from_ms(beacon.startup_time))),
	                            ]},
	                        },
	                        " ",
	                        {
	                            tag: "span",
	                            attrs: {
	                                title: "Latency of requests to the cantal",
	                                class: "b-navbar pointer",
	                            },
	                            children: String(beacon.latency) + "ms",
	                        },
	                        " ",
	                        {
	                            tag: "span",
	                            attrs: {
	                                title: "Time it takes for cantal to read all stats once",
	                                class: "b-navbar pointer",
	                            },
	                            children: String(beacon.scan_duration) + "ms",
	                        },
	                        ((cpu && mem)?({children: [
	                            " ",
	                            bytes_formatter()(mem.rss),
	                            "/",
	                            already_percent_formatter()(cpu.value.value / cpu.delta_sec()),
	                        ]}):("")),
	                    ],
	                };
	        }
	        function render_machine(beacon, stats) {
	            return {
	                    key: "b-navbar:render_machine",
	                    children: [
	                        {
	                            tag: "span",
	                            attrs: {
	                                title: "Uptime of the box running cantal",
	                                class: "b-navbar pointer",
	                            },
	                            children: {children: [
	                                "up ",
	                                format_uptime(till_now_ms(from_ms(beacon.boot_time))),
	                            ]},
	                        },
	                        " ",
	                        ((stats)?(function (old_node) {
	                            var cpu_yaxis = {
	                                    height: 40,
	                                    bg_color: "rgb(237,248,233)",
	                                    skip_color: "white",
	                                    format: already_percent_formatter(),
	                                    colors: [
	                                        [
	                                            100,
	                                            "rgb(186,228,179)",
	                                        ],
	                                        [
	                                            200,
	                                            "rgb(116,196,118)",
	                                        ],
	                                        [
	                                            800,
	                                            "rgb(49,163,84)",
	                                        ],
	                                        [
	                                            1600,
	                                            "rgb(0,109,44)",
	                                        ],
	                                        [
	                                            6400,
	                                            "black",
	                                        ],
	                                    ],
	                                };
	                            var mem = mem_chart(stats.get("memory"));
	                            var usage = cpu_usage(stats.get("cpu_total").chunk, stats.get("cpu_idle").chunks[0][1]);
	                            return {
	                                    tag: "span",
	                                    children: {children: [
	                                        {
	                                            tag: "div",
	                                            attrs: {class: "b-navbar donut"},
	                                            children: donut.render(mem.items, 40, 40, mem.total),
	                                        },
	                                        {
	                                            tag: "div",
	                                            attrs: {class: "b-navbar sparkline"},
	                                            children: sparkline.render(120, stats.get("cpu_total").timestamps, [{
	                                                title: "Cpu",
	                                                values: usage,
	                                                yaxis: cpu_yaxis,
	                                            }]),
	                                        },
	                                    ]},
	                                };
	                        }):("")),
	                    ],
	                };
	        }
	        function render(current_page) {
	            return {
	                    tag: "div",
	                    attrs: {class: "b-navbar navbar navbar-default"},
	                    children: function (old_node) {
	                        var beacon = websock.last_beacon;
	                        return {
	                                tag: "div",
	                                attrs: {class: "b-navbar container-fluid"},
	                                children: {children: [
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-navbar navbar-header"},
	                                        children: {
	                                            tag: "a",
	                                            attrs: {
	                                                href: "#/",
	                                                class: "b-navbar navbar-brand",
	                                            },
	                                            children: {children: [
	                                                "Cantal",
	                                                {
	                                                    tag: "div",
	                                                    attrs: {class: "b-navbar version_string"},
	                                                    children: {children: [
	                                                        {
	                                                            tag: "div",
	                                                            attrs: {class: "b-navbar version"},
	                                                            children: "v0.2.5",
	                                                        },
	                                                        ((beacon)?({children: [
	                                                            " / ",
	                                                            String(beacon.version),
	                                                        ]}):("")),
	                                                    ]},
	                                                },
	                                            ]},
	                                        },
	                                    },
	                                    {
	                                        tag: "div",
	                                        attrs: {class: "b-navbar collapse navbar-collapse"},
	                                        children: {children: [
	                                            {
	                                                tag: "ul",
	                                                attrs: {class: "b-navbar nav navbar-nav"},
	                                                children: {children: [
	                                                    {
	                                                        tag: "li",
	                                                        attrs: {class: "b-navbar" + " " + ((current_page === "status")?("active"):(""))},
	                                                        children: {
	                                                            tag: "a",
	                                                            attrs: {href: "#/status"},
	                                                            children: "Status",
	                                                        },
	                                                    },
	                                                    {
	                                                        tag: "li",
	                                                        attrs: {class: "b-navbar" + " " + ((current_page === "processes")?("active"):(""))},
	                                                        children: {
	                                                            tag: "a",
	                                                            attrs: {href: "#/processes"},
	                                                            children: {children: [
	                                                                "Processes",
	                                                                ((beacon)?(" [" + String(beacon.processes) + "]"):("")),
	                                                            ]},
	                                                        },
	                                                    },
	                                                    {
	                                                        tag: "li",
	                                                        attrs: {class: "b-navbar" + " " + ((current_page === "metrics")?("active"):(""))},
	                                                        children: {
	                                                            tag: "a",
	                                                            attrs: {href: "#/metrics"},
	                                                            children: {children: [
	                                                                "Metrics",
	                                                                ((beacon)?(" [" + String(beacon.values) + "]"):("")),
	                                                            ]},
	                                                        },
	                                                    },
	                                                    {
	                                                        tag: "li",
	                                                        attrs: {class: "b-navbar" + " " + ((current_page === "peers")?("active"):(""))},
	                                                        children: {
	                                                            tag: "a",
	                                                            attrs: {href: "#/peers"},
	                                                            children: {children: [
	                                                                "Peers",
	                                                                ((beacon)?(" [" + String(beacon.peers_with_remote) + "/" + String(beacon.peers) + "]"):("")),
	                                                            ]},
	                                                        },
	                                                    },
	                                                    {
	                                                        tag: "li",
	                                                        attrs: {class: "b-navbar" + " " + ((current_page === "remote")?("active"):(""))},
	                                                        children: {
	                                                            tag: "a",
	                                                            attrs: {href: "#/remote"},
	                                                            children: {children: [
	                                                                "Remote",
	                                                                ((beacon)?(((beacon.remote_total === null)?(" [off]"):(" [" + String(beacon.remote_connected) + "/" + String(beacon.remote_total) + "]"))):("")),
	                                                            ]},
	                                                        },
	                                                    },
	                                                ]},
	                                            },
	                                            ((beacon)?(function (old_node) {
	                                                var query = old_node && old_node.store_query || new Query(2000, {
	                                                        self_cpu: {
	                                                            series: {
	                                                                source: "Fine",
	                                                                condition: [
	                                                                    "And",
	                                                                    [
	                                                                        "Eq",
	                                                                        "pid",
	                                                                        String(beacon.pid),
	                                                                    ],
	                                                                    [
	                                                                        "RegexLike",
	                                                                        "metric",
	                                                                        "_time$",
	                                                                    ],
	                                                                ],
	                                                            },
	                                                            extract: [
	                                                                "DiffToAtMost",
	                                                                1,
	                                                            ],
	                                                            functions: [[
	                                                                "Sum",
	                                                                "Ignore",
	                                                            ]],
	                                                        },
	                                                        self_mem: {
	                                                            series: {
	                                                                source: "Fine",
	                                                                condition: [
	                                                                    "And",
	                                                                    [
	                                                                        "Eq",
	                                                                        "pid",
	                                                                        String(beacon.pid),
	                                                                    ],
	                                                                    [
	                                                                        "Eq",
	                                                                        "metric",
	                                                                        "rss",
	                                                                    ],
	                                                                ],
	                                                            },
	                                                            extract: ["Tip"],
	                                                            functions: [],
	                                                        },
	                                                        memory: {
	                                                            series: {
	                                                                source: "Fine",
	                                                                condition: [
	                                                                    "RegexLike",
	                                                                    "metric",
	                                                                    "^memory\.",
	                                                                ],
	                                                            },
	                                                            extract: ["Tip"],
	                                                            functions: [],
	                                                        },
	                                                        cpu_total: {
	                                                            series: {
	                                                                source: "Fine",
	                                                                condition: [
	                                                                    "RegexLike",
	                                                                    "metric",
	                                                                    "^cpu\.",
	                                                                ],
	                                                            },
	                                                            extract: [
	                                                                "HistoryByNum",
	                                                                120,
	                                                            ],
	                                                            functions: [
	                                                                ["NonNegativeDerivative"],
	                                                                [
	                                                                    "Sum",
	                                                                    "Ignore",
	                                                                ],
	                                                            ],
	                                                        },
	                                                        cpu_idle: {
	                                                            series: {
	                                                                source: "Fine",
	                                                                condition: [
	                                                                    "Eq",
	                                                                    "metric",
	                                                                    "cpu.idle",
	                                                                ],
	                                                            },
	                                                            extract: [
	                                                                "HistoryByNum",
	                                                                120,
	                                                            ],
	                                                            functions: [["NonNegativeDerivative"]],
	                                                        },
	                                                    });
	                                                return {
	                                                        tag: "div",
	                                                        store_query: query,
	                                                        attrs: {class: "b-navbar navbar-right" + " " + ((!websock.connected)?("bg-danger"):(""))},
	                                                        children: {children: [
	                                                            "(",
	                                                            icon("hdd"),
	                                                            " ",
	                                                            ((beacon)?(render_machine(beacon, query.values)):("")),
	                                                            ") (",
	                                                            icon("scale"),
	                                                            ((beacon)?(render_self(beacon, websock.connected, query.values && query.values.get("self_cpu"), query.values && query.values.get("self_mem").to_dict("metric"))):("")),
	                                                            ") ",
	                                                        ]},
	                                                        events: {"$destroyed": ((query.owner_destroyed)?(query.owner_destroyed.handle_event):(function () {
	                                                        }))},
	                                                    };
	                                            }):("")),
	                                        ]},
	                                    },
	                                ]},
	                            };
	                    },
	                    key: "b-navbar:render",
	                };
	        }
	        exports.icon = icon
	        exports.render_self = render_self
	        exports.render_machine = render_machine
	        exports.render = render
	    }.apply(exports, __WEBPACK_AMD_DEFINE_ARRAY__), __WEBPACK_AMD_DEFINE_RESULT__ !== undefined && (module.exports = __WEBPACK_AMD_DEFINE_RESULT__))


/***/ }
/******/ ]);
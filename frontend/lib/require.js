(function(global) {
    var modules = {};

    function require(name) {
        var mod = modules[name];
        if(!mod) {
            console.error("Module", name, "not defined");
            throw Error("ImportError");
        }
        return mod;
    }

    function define(name, requirements, fun) {
        var exp = {}
        modules[name] = exp;
        var args = [];
        for(var i = 0; i < requirements.length; ++i) {
            var v = requirements[i];
            if(v == 'exports') {
                args.push(exp);
            } else {
                args.push(require(v))
            }
        }
        fun.apply(global, args)
    }

    global.require = require;
    global.define = define;
})(this)

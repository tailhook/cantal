import render from 'util/render'


class GuardProxy {
    constructor(guard, component) {
        this._guard = guard
        this._component = component
    }
    process(fun) {
        this._guard.set_handler((...args) => {
            const obj = fun(...args)
            for(var k in obj) {
                this._component[k] = obj[k]
            }
            render.update()
        })
    }
}

export class Component {
    constructor() {
        this._guards = {}
    }
    init() {
    }
    guard(name, value) {
        const old_guard = this._guards[name]
        if(old_guard) {
            value = old_guard.replace_with(value)
            this._guards[name] = value
        } else {
            this._guards[name] = value
            value.start()
        }
        return new GuardProxy(value, this)
    }
    clear_guard(k) {
        var g = this._guards[k]
        if(g) {
            g.stop()
        }
        delete this._guards[k]
    }
    destroy() {
        for(var k in this._guards) {
            this._guards[k].stop()
        }
        delete this._guards
    }
}

export function component(cls, ...args) {
    return function(old_item) {
        if(old_item && (old_item.component instanceof cls)) {
            var cmp = old_item.component
            if(cmp.init) {
                // TODO(tailhook) optimize init
                cmp.init(...args)
            }
        } else {
            var cmp = new cls()
            if(cmp.init) {
                cmp.init(...args)
            }
        }
        try {
            var el = cmp.render()
        } catch(e) {
            console.error("Rendering error", e, e.stack)
            return {
                tag: 'span',
                attrs: {class: 'error'},
                children: e.toString(),
                }
        }
        el.component = cmp
        // Todo use add_events from util/events
        var ev = el.events || (el.events = {})
        ev['$destroyed'] = function() {
            cmp.destroy()
        }
        return el;
    }
}

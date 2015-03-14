
export class Context {
    constructor(component) {
        this.component = component
    }
    // Lifecycle methods
    mount(elem) {
        this._node = cito.vdom.append(elem, {});
        this.update(true);
    }
    update(first_time) {
        if(window.__RENDER_CONTEXT__) {
            console.error("Uncleaned context", window.__RENDER_CONTEXT__)
        }
        window.__RENDER_CONTEXT__ = this
        try {
            if(first_time) {
                this.component.init()
            }
            return cito.vdom.update(this._node, this.component.render())
        } finally {
            if(window.__RENDER_CONTEXT__ !== this) {
                console.error("Replaced context", this,
                    "with", window.__RENDER_CONTEXT__, "during render")
            } else {
                delete window.__RENDER_CONTEXT__
            }
        }
    }
    remove() {
        cito.vdom.remove(this._node)
        this.component.destroy()
    }
    // Client methods
    refresh(element) {
        // TODO(tailhook) optimize rendering
        this.update()
    }
}

Context.current = function() {
    var ctx = window.__RENDER_CONTEXT__
    if(!ctx) {
        throw Error("No active render context")
    }
    return ctx
}

class GuardProxy {
    constructor(guard, component, context) {
        this._guard = guard
        this._component = component
        this._context = context
    }
    process(fun) {
        this._guard.set_handler((...args) => {
            const obj = fun(...args)
            for(var k in obj) {
                this._component[k] = obj[k]
            }
            this._context.refresh(this.component)
        })
    }
}

export class Component {
    constructor() {
        this._guards = {}
    }
    guard(name, value) {
        const old_guard = this._guards[name]
        if(old_guard) {
            old_guard.stop()
        }
        this._guards[name] = value
        value.start()
        return new GuardProxy(value, this, Context.current())
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
        el['component'] = cmp
        var ev = el.events || (el.events = {})
        ev['$destroy'] = function() {
            cmp.destroy()
        }
        return el;
    }
}

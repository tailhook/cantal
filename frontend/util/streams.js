import render from 'util/render'

export class Stream {
    constructor(name) {
        this.name = name
        this.handle_event = this.handle_event.bind(this)
        this._handlers = []
    }

    handle_event(ev) {
        console.log("EVENT", this.name, ev, this._handlers)
        var h = this._handlers;
        for(var i = 0, li = h.length; i < li; ++i) {
            try {
                h[i](ev)
            } catch(e) {
                console.error("Error handing event", ev,
                              "in stream", this.name, e)
            }
        }
        render.update();
    }
    handle(fun) {
        this._handlers.push(fun);
    }
}


import {Stream} from 'util/streams'

export class Tooltip {
    constructor() {
        this.mouseenter = new Stream('tooltip_hover')
        this.enter = new Stream('tooltip_hover')
        this.mouseleave = new Stream('tooltip_leave')
        this.mouseenter.handle(this.show.bind(this))
        this.mouseleave.handle(this.hide.bind(this))
        this.enter.handle(this.show_with_data.bind(this))
        this.visible = false
    }

    show(ev) {
        this.x = ev.pageX
        this.y = ev.pageY
        this.visible = true
    }
    show_with_data(data) {
        this.data = data
        this.visible = true
    }

    hide(ev) {
        this.visible = false
    }

    style() {
        return {
            position: 'fixed',
            left: this.x + 'px',
            top: this.y + 'px',
            }
    }
}

export class Toggle {
    constructor() {
        this.toggle = new Stream('toggle_event')
        this.toggle.handle(this.do_toggle.bind(this))
        this.visible = false
    }

    do_toggle() {
        this.visible = !this.visible;
    }

}


export class Value  {
    constructor() {
        this.keydown = new Stream('set_value')
        this.keydown.handle(this.store.bind(this))
        this.change = this.keydown;
        this.keyup = this.keydown;
        this.value = null;
    }

    store(ev) {
        this.value = ev.target.value;
    }
}

export class Follow  {
    constructor() {
        this.mousemove = new Stream('mousemove')
        this.mouseenter = new Stream('mouseenter')
        this.mouseleave = new Stream('mouseleave')
        this.owner_destroyed = new Stream('owner_destroyed')
        this.mousemove.handle(this.set_coords.bind(this))
        this.mouseenter.handle(this.set_coords.bind(this))
        this.mouseleave.handle(this.do_mouseleave.bind(this))
        this.owner_destroyed.handle(this.cleanup.bind(this))
        this.x = null
        this.y = null
        this._timer = null
    }

    set_coords(ev) {
        this._reset_timer()
        const rect = ev.currentTarget.getBoundingClientRect()
        this.x = Math.floor(ev.clientX - rect.left)
        this.y = Math.floor(ev.clientY - rect.top)
    }
    do_mouseleave() {
        this._timer = setTimeout(this.reset_coords.bind(this), 500)
    }
    reset_coords() {
        this.x = null
        this.y = null
    }
    _reset_timer() {
        if(this._timer) {
            clearInterval(this._timer)
            this._timer = null
        }
    }
    cleanup() {
        this.reset_coords()
        this._reset_timer()
    }
}

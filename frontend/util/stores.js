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

import {Stream} from 'util/streams'

export class Tooltip {
    constructor() {
        this.mouseenter = new Stream('tooltip_hover')
        this.mouseleave = new Stream('tooltip_leave')
        this.mouseenter.handle(this.show.bind(this))
        this.mouseleave.handle(this.hide.bind(this))
        this.visible = false
    }

    show(ev) {
        this.x = ev.pageX
        this.y = ev.pageY
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


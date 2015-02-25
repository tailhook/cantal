import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map} from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'


export class Status {
    mount(elem) {
        this._node = cito.vdom.append(elem, () => this.render());
    }
    render() {
        return hc("div", "container", [
            h("h1", "System Status"),
            { tag: "svg", attrs: { style: {
                width: '256px',
                height: '256px',
                }}, children: [
                    
                ]}
            ]);
    }
    update() {
        cito.vdom.update(this._node, this.render())
    }
    remove() {
        cito.vdom.remove(this._node);
    }
}

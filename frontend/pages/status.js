import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map} from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {DonutChart} from 'util/donut'


export class Status {
    mount(elem) {
        this._mem_chart = new DonutChart()
        this._mem_chart.set_data(8051484, [
            {name: "MemFree", value: 694568, color: '#e5f5f9'},
            {name: "Buffers", value: 221520, color: '#99d8c9'},
            {name: "Cached", value: 3475876, color: '#2ca25f'},
            ])
        this._node = cito.vdom.append(elem, () => this.render());
    }
    render() {
        return hc("div", "container", [
            h("h1", "System Status"),
            this._mem_chart.render(),
        ]);
    }
    update() {
        cito.vdom.update(this._node, this.render())
    }
    remove() {
        cito.vdom.remove(this._node);
    }
}

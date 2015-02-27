import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map,
        td_left, td_right, th_left, th_right,
        } from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {DonutChart} from 'util/donut'
import {RefreshJson} from 'util/request'


const TYPE_TO_ICON = {
    'Float': icon('equalizer'),
    'Integer': icon('stats'),
    'Counter': icon('cd'),
    'State': icon('dashboard'),
}


export class Values {
    mount(elem) {
        this._node = cito.vdom.append(elem, () => this.render());
        this._refresher = new RefreshJson("/values.json", (data, latency) => {
            this.latency = latency;
            if(data instanceof Error) {
                this.error = data;
            } else {
                this.data = data;
                this.error = null;
            }
            this.update()
        });
        this._refresher.start()
    }
    render() {
        return hc("div", "container", [
            h("h1", ["All Values"]),
            hc("div", "text-right",
                this.error ? 'Error: ' + this.error.message
                           : `Fetched in ${this.latency}ms`),
        ].concat(
            this.data
                ? this.data.items.map(this.render_process.bind(this))
                : []
        ));
    }
    render_value(pair) {
        var [name, value] = pair;
        if(value.variant == 'State') {
            return {children: [
                h('tr', [
                    td_left(JSON.stringify(name)),
                    td_left(TYPE_TO_ICON[value.variant] || value.variant),
                    td_right(format_uptime(
                        till_now_ms(from_ms(value.fields[0])))),
                    ]),
                h('tr', {tag: 'td', attrs: { 'colspan': 100, class: 'bg-info' }, children: [
                    icon('arrow-up'), ' ', value.fields[1]
                ]})
            ]}
        } else {
            return h('tr', [
                td_left(JSON.stringify(name)),
                td_left(TYPE_TO_ICON[value.variant] || value.variant),
                td_right(value.fields[0].toString()),
                ])
        }
    }
    render_process(item) {
        return hc("div", "col-xs-12", [
            h("h2", `${item.pid} ${item.process.name}`),
            hc("table", "table table-hover", [
                h("thead", h("tr", [
                    th_left('name'),
                    th_left(icon('asterisk')),
                    th_right('value'),
                    ])),
                h("tbody", item.values.map(this.render_value.bind(this))),
            ])])
    }
    update() {
        cito.vdom.update(this._node, this.render())
    }
    remove() {
        cito.vdom.remove(this._node);
    }
}

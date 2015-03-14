import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map} from 'util/html'
import {toggle} from 'util/events'

export class Chart {
    constructor() {
        this.collapsed = true;
    }
    init(chart, options={}) {
        this.title = options.title || 'The Chart'
        this.unit = options.unit || ''
        this.items = options.items || []
        this.chart = chart
        this._has_collapse = options.items.filter(x => x.collapsed).length;
    }
    render() {
        const items = this.collapsed
            ? this.items.filter(x => !x.collapsed)
            : this.items;
        return {children: [
            h("h2", this.title),
            hc("div", "row", [
                hc("div", "col-xs-4", this.chart),
                hc("div", "col-xs-4", [
                    hc("table", "table table-condensed table-hover", [
                        h("thead", h("tr", [
                            h('th', ''),
                            h('th', 'Title'),
                            hc('th', 'text-right', this.unit),
                            ])),
                        h("tbody", items.map((item) => h('tr', [
                            h('td', item.color
                                ? {tag: 'span', attrs: {
                                    class: 'sample',
                                    style: "background-color: " + item.color,
                                    }}
                                : ""),
                            h('td', item.title),
                            hc('td', 'text-right',
                                item.text || item.value.toString()),
                        ]))),
                        h("tfoot", h("tr", [
                            h('td', ''),
                            hc('td', 'text-center', {
                                tag: 'button',
                                attrs: {class: 'btn btn-default btn-xs'},
                                events: {click: toggle(this, 'collapsed')},
                                children: this.collapsed
                                    ? icon('chevron-down')
                                    : icon('chevron-up'),
                            }),
                            hc('td', ''),
                            ])),
                    ]),
                ]),
            ]),
        ]}
    }
}

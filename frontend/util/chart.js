import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map} from 'util/html'

export class Chart {
    constructor(chart, options) {
        this.title = options.title || 'The Chart'
        this.unit = options.unit || ''
        this.items = []
        this.chart = chart
    }
    set_data(info) {
        this.items = info.items
        this.chart.set_data({
            total: info.total,
            items: info.items.filter((item) => item.color),
        })
    }
    render(mem) {
        return {children: [
            h("h2", this.title),
            hc("div", "row", [
                hc("div", "col-xs-4", this.chart.render()),
                hc("div", "col-xs-4", [
                    hc("table", "table table-condensed table-hover", [
                        h("thead", h("tr", [
                            h('th', ''),
                            h('th', 'Title'),
                            hc('th', 'text-right', this.unit),
                            ])),
                        h("tbody", this.items.map((item) => h('tr', [
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
                    ]),
                ]),
            ]),
        ]}
    }
}

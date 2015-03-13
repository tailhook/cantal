import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map} from 'util/html'

export class Chart {
    init(chart, options={}) {
        this.title = options.title || 'The Chart'
        this.unit = options.unit || ''
        this.items = options.items || []
        this.chart = chart
    }
    render(mem) {
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

import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map} from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {DonutChart} from 'util/donut'
import {RefreshJson} from 'util/request'


function memchart(memory) {
    var items = [
        { title: "Free", value: memory.mem_free, color: "#e5f5f9" },
        { title: "Buffers", value: memory.buffers, color: "#99d8c9" },
        { title: "Cache", value: memory.cached, color: "#2ca25f" },
        { title: "Used", value: memory.used, color: "#A0A0A0" },
        ];
    return {
        total: memory.mem_total,
        items: items,
    }
}

function memtd(val) {
    return
}

function memtable(lst) {
    return lst.map(function (triple) {
        var [color, title, value] = triple;
        return h('tr', [
            h('td', color
                ? {tag: 'span', attrs: {
                    class: 'sample',
                    style: "background-color: " + color,
                    }}
                : ""),
            h('td', title),
            hc('td', 'text-right', (value / 1048576).toFixed(2))
            ])
    })
}


export class Status {
    mount(elem) {
        this._mem_chart = new DonutChart()
        this._node = cito.vdom.append(elem, () => this.render());
        this._refresher = new RefreshJson("/details.json", (data, latency) => {
            this.latency = latency;
            if(data instanceof Error) {
                this.error = data;
            } else {
                this.data = data;
                var mem = this.data.machine.memory;
                mem.used = mem.mem_total
                           - mem.mem_free
                           - mem.buffers
                           - mem.cached
                this._mem_chart.set_data(memchart(mem))
                this.error = null;
            }
            this.update()
        });
        this._refresher.start()
    }
    render_memory(mem) {
        return {children: [
            h("h2", "Memory Usage"),
            hc("div", "row", [
                hc("div", "col-xs-4", [ this._mem_chart.render() ]),
                hc("div", "col-xs-4", [
                    hc("table", "table table-condensed table-hover", [
                        h("thead", h("tr", [
                            h('th', ''),
                            h('th', 'Title'),
                            hc('th', 'text-right', 'MiB'),
                            ])),
                        h("tbody", memtable([
                            ['', 'Total', mem.mem_total],
                            ['#e5f5f9', 'Free', mem.mem_free],
                            ['#99d8c9', 'Buffers', mem.buffers],
                            ['#2ca25f', 'Cached', mem.cached],
                            ['#a0a0a0', 'Used', mem.used],
                            ['', 'Available', mem.mem_available],
                            ['', 'Swap Cached', mem.swap_cached],
                            ['', 'Active', mem.active],
                            ['', 'Inactive', mem.inactive],
                            ['', 'Unevictable', mem.unevictable],
                            ['', 'Memory Locked', mem.mlocked],
                            ['', 'Swap Total', mem.swap_total],
                            ['', 'Swap Free', mem.swap_free],
                            ['', 'Dirty', mem.dirty],
                            ['', 'Writeback', mem.writeback],
                            ['', 'Commit Limit', mem.commit_limit],
                            ['', 'Committed Address Space', mem.committed_as],
                        ])),
                    ]),
                ]),
            ]),
        ]}
    }
    render() {
        return hc("div", "container", [
            h("h1", "System Status"),
            this.error ? h("div", "Error: " + this.error) : "",
            this.data ? this.render_memory(this.data.machine.memory) : "",
        ]);
    }
    update() {
        cito.vdom.update(this._node, this.render())
    }
    remove() {
        cito.vdom.remove(this._node);
    }
}

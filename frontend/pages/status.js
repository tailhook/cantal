import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map} from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {DonutChart} from 'util/donut'
import {Chart} from 'util/chart'
import {RefreshJson} from 'util/request'

const MEM_ITEMS = [
    ['', 'Total', 'mem_total'],
    ['#e5f5f9', 'Free', 'mem_free'],
    ['#99d8c9', 'Buffers', 'buffers'],
    ['#2ca25f', 'Cached', 'cached'],
    ['#a0a0a0', 'Used', 'used'],
    ['', 'Available', 'mem_available'],
    ['', 'Swap Cached', 'swap_cached'],
    ['', 'Active', 'active'],
    ['', 'Inactive', 'inactive'],
    ['', 'Unevictable', 'unevictable'],
    ['', 'Memory Locked', 'mlocked'],
    ['', 'Swap Total', 'swap_total'],
    ['', 'Swap Free', 'swap_free'],
    ['', 'Dirty', 'dirty'],
    ['', 'Writeback', 'writeback'],
    ['', 'Commit Limit', 'commit_limit'],
    ['', 'Committed Address Space', 'committed_as'],
];

function memchart(mem) {
    return {
        total: mem.mem_total,
        items: MEM_ITEMS.map(([color, title, property]) => {
            return {
                color: color,
                title: title,
                value: mem[property],
                text: (mem[property]/1048576).toFixed(1),
            }
        })
    }
}



export class Status {
    mount(elem) {
        this._mem_chart = new Chart(new DonutChart(), {
            title: 'Memory',
            unit: 'MiB',
        })
        this._node = cito.vdom.append(elem, () => this.render());
        this._refresher = new RefreshJson("/details.json", (data, latency) => {
            this.latency = latency;
            if(data instanceof Error) {
                this.error = data;
            } else {
                this._preprocess(data)
                this.error = null;
            }
            this.update()
        });
        this._refresher.start()
    }
    _preprocess(data) {
        var mem = data.machine.memory;
        mem.used = mem.mem_total
                   - mem.mem_free
                   - mem.buffers
                   - mem.cached
        this._mem_chart.set_data(memchart(mem))
    }
    render() {
        return hc("div", "container", [
            h("h1", "System Status"),
            this.error ? h("div", "Error: " + this.error) : "",
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

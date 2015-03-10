import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map} from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {DonutChart} from 'util/donut'
import {Chart} from 'util/chart'
import {RefreshJson} from 'util/request'

const MEM_COLORS = {
    MemFree: '#e5f5f9',
    Buffers: '#99d8c9',
    Cached: '#2ca25f',
    Used: '#a0a0a0',
};

const MEM_ORDER = {
    MemTotal: 1,
    Used: 2,
    Cached: 3,
    Buffers: 4,
    MemFree: 5,
    Dirty: 6,
    Writeback: 7,
    SwapTotal: 8,
    Committed_AS: 9,
    CommitLimit: 10,
}

function memchart(mem) {
    return {
        total: mem.MemTotal,
        items: Object.keys(mem).map(key => {
            var item = mem[key];
            return {
                color: MEM_COLORS[key],
                title: key,
                value: mem[key],
                text: (mem[key]/1048576).toFixed(1),
            }
        }).sort((a, b) => (MEM_ORDER[a.title] || 10000) - (MEM_ORDER[b.title] || 10000))
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
        var mem = {}
        for(var item of data.metrics) {
            if(item[0].metric.substr(0, 7) == 'memory.') {
                mem[item[0].metric.substr(7)] = item[1]
            }
        }
        mem.Used = mem.MemTotal
                   - mem.MemFree
                   - mem.Buffers
                   - mem.Cached
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

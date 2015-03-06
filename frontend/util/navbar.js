import {tag_class as hc, tag as h, link, icon,
        title_span as title} from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {RefreshJson} from 'util/request'
import {Sparkline} from 'util/sparkline'
import {DonutChart} from 'util/donut'


function nav(classname, href, ...args) {
    if(href == window.location.hash) {
        classname += ' active'
    }
    return link(classname, href, ...args)
}


export class Navbar {
    constructor() {
    }
    mount(elem) {
        this._memory_donut = new DonutChart(32, 32)
        this._node = cito.vdom.append(elem, () => this.render())
        this._page = ''
        this._refresher = new RefreshJson("/status.json", (data, latency) => {
            this.latency = latency
            if(data instanceof Error) {
                this.error = data
            } else {
                this._preprocess(data)
                this.error = null
            }
            this.update()
        });
        this._refresher.start()
    }
    update() {
        cito.vdom.update(this._node, this.render())
    }
    _preprocess(data) {
        this.data = data
        this._cpu_graph(data)
        this._memory_graph(data)
    }
    _memory_graph(d) {
        this._memory_donut.set_data({total: d.mem_total, items: [
            {color: '#e5f5f9', title: 'Free', value: d.mem_free},
            {color: '#99d8c9', title: 'Free', value: d.mem_buffers},
            {color: '#2ca25f', title: 'Free', value: d.mem_cached},
            {color: '#a0a0a0', title: 'Free', value:
                d.mem_total - d.mem_free - d.mem_buffers - d.mem_cached},
        ]})
    }
    _cpu_graph(data) {
        const user = data.cpu_user.fine
        const nice = data.cpu_nice.fine
        const idle = data.cpu_idle.fine
        const system = data.cpu_system.fine
        const cpu_graph = []
        let prev_use
        let prev_total
        for(var i = data.history_timestamps.length-1; i >= 0; --i) {
            const use = user[i] + nice[i] + system[i]
            const total = use + idle[i]
            if(prev_total) {
                cpu_graph.push([
                    data.history_timestamps[i][0],
                    (use - prev_use) / (total - prev_total),
                    ])
            }
            prev_use = use
            prev_total = total
        }
        this._cpu_sparkline = new Sparkline(cpu_graph)
    }
    render_self() {
        var stats = this.data;
        return hc('span', '', [
            title("Uptime of the cantal agent itself", ['up ',
                format_uptime(till_now_ms(from_ms(stats.startup_time))) ]),
            ' / ',
            title("Latency of requests to the cantal",
                  [ this.latency.toFixed(0), 'ms']),
            ' / ',
            title("Time it takes for cantal to read all stats once",
                  [ stats.scan_time.toString(), 'ms']),
        ]);
    }
    render_machine() {
        var data = this.data;
        return hc('span', '', [
            title("Minute Load Average", [ data.load_avg_1min.toFixed(2) ]),
            ' / ',
            title("5 Minutes Load Average",
                [ data.load_avg_5min.toFixed(2) ]),
            ' / ',
            title("15 Minutes Load Average",
                [ data.load_avg_15min.toFixed(2) ]),
            ' / ',
            title("Uptime of the box running cantal", [
                'up ', format_uptime(till_now_ms(from_ms(data.boot_time*1000)))
            ]),
            ' ',
            this._cpu_sparkline ? this._cpu_sparkline.render() : '',
            ' ',
            this._memory_donut ? this._memory_donut.render() : '',
        ]);
    }
    render() {
        var hash = window.location.hash;
        return hc("div", "navbar navbar-default", [
            hc('div', 'container-fluid', [
                hc('div', 'navbar-header', [
                    link('navbar-brand', "#/", "Cantal"),
                ]),
                hc('div', 'collapse navbar-collapse', [
                    hc('ul', 'nav navbar-nav', [
                        hc('li', hash == "#/processes" ? 'active' : '',
                            [ link("", "#/processes", "Processes") ]),
                        hc('li', hash == "#/values" ? 'active' : '',
                            [ link("", "#/values", "Values") ]),
                        hc('li', hash == "#/totals" ? 'active' : '',
                            [ link("", "#/totals", "Totals") ]),
                    ]),
                    hc('form',
                        'navbar-form navbar-right' +
                            (this.error ? ' bg-danger': ''), [
                        '( ',
                            icon('hdd'), ' ',
                            this.data && this.render_machine() || "",
                        ' ) ( ',
                            icon('scale'), ' ',
                            this.data && this.render_self() || "",
                        ' ) ',
                        this.error && this.error.message || "",
                        nav('btn btn-default', '#/status', 'Status'),
                    ]),
                ]),
            ]),
        ]);
    }
}

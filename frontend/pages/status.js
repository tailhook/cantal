import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {Component, component} from 'util/base'
import {toggle} from 'util/events'
import {DonutChart} from 'util/donut'
import {Plot} from 'util/plot'
import {RefreshJson} from 'util/request'
import template from 'templates/status.mft'

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

function memchart(data) {
    var mem = {}
    for(var item of data.latest) {
        if(item[0].metric.substr(0, 7) == 'memory.') {
            mem[item[0].metric.substr(7)] = item[1]
        }
    }
    mem.Used = mem.MemTotal
               - mem.MemFree
               - mem.Buffers
               - mem.Cached
    return {
        title: 'Memory',
        unit: 'MiB',
        total: mem.MemTotal,
        items: Object.keys(mem).map(key => {
            var item = mem[key];
            return {
                color: MEM_COLORS[key],
                title: key,
                value: mem[key],
                text: (mem[key]/1048576).toFixed(1),
                collapsed: MEM_ORDER[key] === undefined,
            }
        }).sort((a, b) => (MEM_ORDER[a.title] || 10000) - (MEM_ORDER[b.title] || 10000))
    }
}

function network(data) {
    const raw_bytes_rx = []
    const raw_bytes_tx = []
    for(var _ of data.history_timestamps) {
        raw_bytes_rx.push(0)
        raw_bytes_tx.push(0)
    }
    for(var item of data.history) {
        let dest
        let iface = item[0].interface
        // Ignore tun interfaces as they double the traffic if it's VPN
        if(!iface || iface == 'lo' || iface.substr(3) == 'tun') continue;
        if(item[0].metric == 'net.interface.tx.bytes') {
            dest = raw_bytes_tx
        } else if(item[0].metric == 'net.interface.rx.bytes') {
            dest = raw_bytes_rx
        } else {
            continue
        }
        const ar = item[1].fine
        if(ar) {
            for(var i in ar) {
                dest[i] += ar[i]
            }
        }
    }
    const bytes_rx = []
    const bytes_tx = []
    raw_bytes_rx.reduce((n, o) => { bytes_rx.push(n - o); return o })
    raw_bytes_tx.reduce((n, o) => { bytes_tx.push(n - o); return o })
    return {
        bytes_rx,
        bytes_tx,
        }
}

function disk(data) {
    return {}
}

export class Status extends Component {
    constructor() {
        super()
        this.mem_chart = {items:[]}
    }
    init(elem) {
        this.guard('json', new RefreshJson("/details.json"))
        .process((data, latency) => {
            if(data instanceof Error) {
                return {error: data, latency}
            } else {
                return {
                    error: null,
                    timestamps: data.history_timestamps,
                    mem_chart: memchart(data),
                    network: network(data),
                    disk: disk(data),
                }
            }
        })
        this.guard('new_query', new RefreshJson("/query.json", {
            post_body: JSON.stringify({'rules': {
                'memory': {
                    'source': 'Tip',
                    'condition': {'variant': "RegexLike", fields: [
                        "metric",
                        "^memory\.",
                        ]},
                    'key': ['metric'],
                    'aggregation': 'None',
                    'load': 'Raw',
                    'limit': 1,
                    },
                'network': {
                    'source':'Fine',
                    'condition': {'variant': "RegexLike", fields: [
                        "metric",
                        "^net.interface.[rt]x.bytes$",
                        ]},
                    'key': ['metric'],
                    'aggregation': 'None',
                    'load': 'Raw',
                    'limit': 1,
                    },
            }})}))
        .process((data, latency) => {
            console.log("GOT DATA", data)
        })
    }
    render() {
        const ts = this.timestamps && this.timestamps.slice(1)
        return template.render(this.error, ts, this.mem_chart, this.network)
    }
}

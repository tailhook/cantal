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

function memchart(metrics) {
    metrics['memory.Used'] = [metrics['memory.MemTotal'][0]
                    - metrics['memory.MemFree'][0]
                    - metrics['memory.Buffers'][0]
                    - metrics['memory.Cached'][0]]
    return {
        title: 'Memory',
        unit: 'MiB',
        total: metrics['memory.MemTotal'][0],
        items: Object.keys(metrics).map(metricname => {
            var value = metrics[metricname][0];
            let key = metricname.substr('memory.'.length);
            return {
                color: MEM_COLORS[key],
                title: key,
                value: value,
                text: (value/1048576).toFixed(1),
                collapsed: MEM_ORDER[key] === undefined,
            }
        }).sort((a, b) => (MEM_ORDER[a.title] || 10000) -
                          (MEM_ORDER[b.title] || 10000))
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
        this.guard('new_query', new RefreshJson("/query.json", {
            post_body: JSON.stringify({'rules': {
                'memory': {
                    'source': 'Fine',
                    'condition': ['regex-like', 'metric', '^memory\\.'],
                    'key': ['metric'],
                    'aggregation': 'None',
                    'load': 'Tip',
                    'limit': 15,  // about 30 seconds
                    },
                'network': {
                    'source':'Fine',
                    'condition': ['and',
                        ['regex-like', 'metric',
                         "^net.interface.[rt]x.bytes$"],
                        ['not', ['or',
                            ['eq', 'interface', 'lo'],
                            ['regex-like', 'interface', '^tun|^vboxnet']]]],
                    'key': ['metric'],
                    'aggregation': 'CasualSum',
                    'load': 'Rate',
                    'limit': 1100,
                    },
                'disk': {
                    'source':'Fine',
                    'condition': ['and',
                        ['regex-like', 'metric',
                         "^disk\.(?:read|write)\.ops$"],
                        ['regex-like', 'device',
                         "^sd[a-z]$"]],
                    'key': ['metric'],
                    'aggregation': 'CasualSum',
                    'load': 'Rate',
                    'limit': 1100,
                    },
            }})}))
        .process((data, latency) => {
            if(data instanceof Error) {
                return {error: data, latency}
            } else {
                return {
                    mem_chart: memchart(data.dataset.memory),
                    network: data.dataset.network,
                    disk: data.dataset.disk,
                    fine_timestamps: data.fine_timestamps
                                     .map(([v, d]) => from_ms(v + d/2)),
                }
            }
        })
    }
    render() {
        const ts = this.fine_timestamps
        return template.render(this.error, ts, this.mem_chart,
            this.network, this.disk)
    }
}

import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {Component, component} from 'util/base'
import {toggle} from 'util/events'
import {Plot} from 'util/plot'
import {RefreshJson} from 'util/request'
import template from 'templates/status.mft'
import {cpu_chart, mem_chart} from 'util/compute'


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
                    'limit': 15,  // about 30 seconds
                    },
                'cpu_sum': {
                    'source': 'Fine',
                    'condition': ['regex-like', 'metric', '^cpu\\.'],
                    'key': [],
                    'aggregation': 'CasualSum',
                    'limit': 1100,
                    },
                'cpu': {
                    'source': 'Fine',
                    'condition': ['regex-like', 'metric', '^cpu\\.'],
                    'key': ['metric'],
                    'aggregation': 'None',
                    'limit': 1100,
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
                    'limit': 1100,
                    },
                'disk': {
                    'source':'Fine',
                    'condition': ['and',
                        ['regex-like', 'metric',
                         "^disk\.(?:read|write)\.(:?ops|bytes)$"],
                        ['regex-like', 'device',
                         "^sd[a-z]$"]],
                    'key': ['metric'],
                    'aggregation': 'CasualSum',
                    'limit': 1100,
                    },
                'disk_in_progress': {
                    'source':'Fine',
                    'condition': ['and',
                        ['regex-like', 'metric',
                         "^disk\.in_progress$"],
                        ['regex-like', 'device',
                         "^sd[a-z]$"]],
                    'key': ['metric'],
                    'aggregation': 'CasualSum',
                    'limit': 1100,
                    },
            }})}))
        .process((data, latency) => {
            if(data instanceof Error) {
                return {error: data, latency}
            } else {
                return {
                    mem_chart: mem_chart(data.dataset.memory),
                    cpu_chart: cpu_chart(data.dataset.cpu_sum,
                                         data.dataset.cpu),
                    dataset: data.dataset,
                    fine_timestamps: data.fine_timestamps
                                     .map(([v, d]) => from_ms(v + d/2)),
                }
            }
        })
    }
    render() {
        const ts = this.fine_timestamps
        return template.render(this.error, ts, this.dataset || {},
            this.mem_chart, this.cpu_chart)
    }
}

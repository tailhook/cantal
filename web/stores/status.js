import {METRICS} from '../middleware/local-query.js'

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
    SwapUsed: 8,
    Committed_AS: 9,
    CommitLimit: 10,
}

export function mem_chart(metrics) {
    metrics = metrics.to_dict('metric', 'memory.')
    metrics.Used = (metrics.MemTotal - metrics.MemFree - metrics.Buffers
                    - metrics.Cached)
    metrics.SwapUsed = metrics.SwapTotal - metrics.SwapFree
    return {
        title: 'Memory',
        unit: 'MiB',
        total: metrics.MemTotal,
        items: Object.keys(metrics).map(metricname => {
            var value = metrics[metricname]
            return {
                color: MEM_COLORS[metricname],
                title: metricname,
                value: value,
                text: (value/1048576).toFixed(1),
                collapsed: MEM_ORDER[metricname] === undefined,
            }
        }).sort((a, b) => (MEM_ORDER[a.title] || 10000) -
                          (MEM_ORDER[b.title] || 10000))
    }
}



export function cpu_chart(metrics) {
    let dic = metrics.to_dict('metric', 'cpu.')
    return {
        total: dic.idle.values.map((x, i) => dic.TOTAL.values[i] - x),
        timestamps: metrics.chunks[0][2],
        ...dic,
    }
}

export var prefix_chart = prefix => map_metrics(metrics => {
    return {
        ...metrics.to_dict('metric', prefix),
        timestamps: metrics.chunks[0][2],
    }
})

var map_metrics = fun => (state, action) => {
    if(action.type == METRICS) {
        return fun(action.metrics)
    }
    return state
}

export var memory = map_metrics(mem_chart)
export var cpu = map_metrics(cpu_chart)
export var self_cpu = map_metrics(
    x => (x.value.value * 1000 /
          (x.timestamps[0].getTime() - x.timestamps[1].getTime())))
export var self_mem = map_metrics(x => x.values[0][1].value)

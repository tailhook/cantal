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
    metrics['memory.Used'] = [[metrics['memory.MemTotal'][0][0]
                    - metrics['memory.MemFree'][0][0]
                    - metrics['memory.Buffers'][0][0]
                    - metrics['memory.Cached'][0][0]]]
    metrics['memory.SwapUsed'] = [[metrics['memory.SwapTotal'][0][0]
                    - metrics['memory.SwapFree'][0][0]]]
    return {
        title: 'Memory',
        unit: 'MiB',
        total: metrics['memory.MemTotal'][0][0],
        items: Object.keys(metrics).map(metricname => {
            var value = metrics[metricname][0][0];
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



export function cpu_chart(cpu_total, parts) {
    parts['cpu.usage'] = parts['cpu.idle'][0].map((x, i) => cpu_total[i] - x)
    return parts
}

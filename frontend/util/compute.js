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



export function cpu_usage(cpu_total, parts) {
    let dict = parts.to_dict('metric', 'cpu.')
    return dict['idle'].values.map((x, i) => cpu_total.chunk.values[i] - x)
}

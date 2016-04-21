import {DATA, ERROR} from '../middleware/request'
import {METRICS} from '../middleware/remote-query.js'
import {format_uptime, till_now_ms, from_ms} from '../util/time'
import {last_beacon} from '../websock'


function machine(mach_data) {
    let res = mach_data.chunks.map(([key, values, timestamps]) => { return {
        title: key.cgroup,
        values: values.values.map(x => x*1),
        timestamps: timestamps,
    }});
    res.sort((x, y) => x.title.localeCompare(y.title))
    return res
}

function _cpu(data) {
    return new Map(Array.from(data.entries())
                   .map(([host, values]) => [host, machine(values)]))
}

var map_metrics = fun => (state, action) => {
    if(action.type == METRICS) {
        return fun(action.metrics)
    }
    return state
}

export var cpu = map_metrics(_cpu)

import {DATA, ERROR} from '../middleware/request'
import {METRICS} from '../middleware/remote-query.js'
import {format_uptime, till_now_ms, from_ms} from '../util/time'
import {last_beacon} from '../websock'


function machine(mach_data) {
    let res = new Map();
    mach_data.chunks.sort((x, y) => x[0].cgroup.localeCompare(y[0].cgroup))
    mach_data.chunks.map(([key, values, timestamps]) => {
        if(!res.get(key.cgroup)) {
            res.set(key.cgroup, {
                timestamps: timestamps
            })
        }
        res.get(key.cgroup)[key.metric] = values.values.map(x => x+0)
    });
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

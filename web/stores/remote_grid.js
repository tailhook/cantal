import {DATA, ERROR} from '../middleware/request'
import {METRICS} from '../middleware/remote-query.js'
import {format_uptime, till_now_ms, from_ms} from '../util/time'
import {last_beacon} from '../websock'
import {cpu_chart, mem_chart} from './status'

function sortname(peer) {
    let parts = peer.name.split('.');
    parts.reverse();
    return parts.join('.');
}

export function peer_list(state={}, action) {
    switch(action.type) {
        case DATA:
            let list = [{
                ... last_beacon,
                id: last_beacon.id,
                sortname: sortname(last_beacon),
            }]
            for(let peer of action.data.peers) {
                peer.sortname = sortname(peer)
                list.push(peer)
            }
            list.sort((x, y) => x.sortname.localeCompare(y.sortname))
            state = {list: list}
            break;
        case ERROR:
            state = {error: action.error, ...state}
            break;
    }
    return state
}

function _memory(data) {
    return new Map(Array.from(data.entries())
                   .map(([host, values]) => [host, mem_chart(values)]))
}

function _cpu(data) {
    return new Map(Array.from(data.entries())
                   .map(([host, values]) => [host, cpu_chart(values)]))
}

var map_metrics = fun => (state, action) => {
    if(action.type == METRICS) {
        return fun(action.metrics)
    }
    return state
}

export var memory = map_metrics(_memory)
export var cpu = map_metrics(_cpu)

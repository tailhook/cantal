import {DATA, ERROR} from '../middleware/request'
import {METRICS} from '../middleware/local-query.js'
import {decode_cmdline} from '../util/process.js'

function detect_supervisor(item) {
  // Usually supervisors are non-interesting, so we hide them by default
  // Currently we only detect 'lithos_knot' as unuseful supervisor
  return item.cmdline.indexOf('lithos_knot ') >= 0
}

export function processes(state=null, action) {
    if(action.type == DATA) {
        let map = {}
        for(let item of action.data.all) {
            item.cmdline = decode_cmdline(item.cmdline)
            item.is_supervisor = detect_supervisor(item)
            let group = map[item.uid];
            if(!group) {
                group = {
                    processes: [],
                }
                map[item.uid] = group
            }
            group.processes.push(item);
        }
        let res = new Map()
        let keys = Object.keys(map);
        keys.sort((a, b) => (a - b));
        for(var k of keys) {
            res.set(parseInt(k), map[k])
        }
        return res;
    }
    return state;
}

export function sockets(state={}, action) {
    switch(action.type) {
        case DATA:
            state = {
                latency: action.latency,
                data: action.data,
            }
            break;
        case ERROR:
            state = {error: action.error, ...state}
            break;
    }
    return state
}

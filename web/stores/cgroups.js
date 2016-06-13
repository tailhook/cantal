import {DATA} from '../middleware/request'
import {METRICS} from '../middleware/local-query.js'
import {decode_cmdline} from '../util/process.js'


export function groups(state=null, action) {
    if(action.type == METRICS) {
        let map = new Map()
        for(let tuple of action.metrics.values) {
            let [k] = tuple;
            let supergroup = k.cgroup.split('.')[0]
            let group = k.cgroup.split('.').splice(1).join('.')
            let sub = map.get(supergroup);
            if(!sub) {
                sub = new Map();
                map.set(supergroup, sub);
            }
            let glist = sub.get(group);
            if(!glist) {
                glist = []
                sub.set(group, glist)
            }
            glist.push({pid: parseInt(k.pid)})
        }
        return map
    }
    return state;
}

function detect_supervisor(item) {
  // Usually supervisors are non-interesting, so we hide them by default
  // Currently we only detect 'lithos_knot' as unuseful supervisor
  return item.cmdline.indexOf('lithos_knot ') >= 0
}

export function processes(state=null, action) {
    if(action.type == DATA) {
        let map = new Map()
        for(let item of action.data.all) {
            item.cmdline = decode_cmdline(item.cmdline)
            item.is_supervisor = detect_supervisor(item)
            map.set(item.pid, item);
        }
        return map;
    }
    return state;
}

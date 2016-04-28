import {DATA} from '../middleware/request'
import {METRICS} from '../middleware/local-query.js'


export function metrics(state=null, action) {
    if(action.type == METRICS) {
        let map = new Map()
        for(let tuple of action.metrics.values) {
            let [k] = tuple;
            k.pid = parseInt(k.pid);
            let lst = map.get(k.pid);
            if(lst == null) {
                lst = []
                map.set(k.pid, lst)
            }
            lst.push(tuple);
        }
        return map
    }
    return state;
}

export function processes(state=null, action) {
    if(action.type == DATA) {
        let map = new Map()
        for(let item of action.data.all) {
            item.cmdline.replace('\u0000', ' ')
            map.set(item.pid, item);
        }
        return map;
    }
    return state;
}

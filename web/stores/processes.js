import {DATA, ERROR} from '../middleware/request'
import {format_uptime, till_now_ms, from_ms} from '../util/time'

function build_tree(data) {
    var toplevel = [];
    var by_id = {};
    var tree = {};
    for(var p of data.all) {

        p.uptime = format_uptime(till_now_ms(from_ms(
                    p.start_time + data.uptime_base*1000)))

        by_id[p.pid] = p;
        var lst = tree[p.ppid];
        if(lst === undefined) {
            lst = tree[p.ppid] = [];
        }
        lst.push(p);
        if(p.ppid == 1) {
            toplevel.push(p);
        }
    }
    return {
        all: data.all,
        uptime_base: data.boot_time,
        toplevel,
        tree,
    }
}

export function processes(state={}, action) {
    switch(action.type) {
        case DATA:
            state = {
                latency: action.latency,
                ...build_tree(action.data)
            }
            break;
        case ERROR:
            state = {error: action.error, ...state}
            break;
    }
    return state
}

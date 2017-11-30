import {uptime} from '../util/process.js'
import {DATA, ERROR} from '../middleware/request'

function build_tree(data) {
    var toplevel = [];
    var by_id = {};
    var tree = {};
    for(var p of data.all) {

        p.uptime = uptime(p)

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

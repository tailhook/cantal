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
            let group = map[k]
            let kind = 'unknown'
            if(k == '0') {
                kind = 'root'
            } else {
                let cmds = {}
                let cgroups = {}
                for(var p of group.processes) {
                    if(p.cgroup) {
                        let grp = p.cgroup
                        if(grp.substr(0, 7) == 'system.' ||
                            grp.substr(0, 7) == 'lithos.')
                        {
                            grp = grp.substr(7);
                        }
                        let digits = grp.match(/\.\d+$/);
                        if(digits) {
                            grp = grp.substr(0, digits.index);
                        }
                        cgroups[grp] = 1
                        kind = 'container' // account mixed groups?
                    } else if(p.cmdline.match(/^\-\w+sh\b/)) {
                        kind = 'interactive'
                        break
                    } else {
                        let command = p.cmdline
                            .match(/^[^: ]*?([^\/: ]+)(?:[\s:]|$)/)
                        if(command && command[1]) {
                            cmds[command[1]] = 1;
                        }
                    }
                }
                if(kind == 'unknown') {
                    let lst = Object.keys(cmds)
                    lst.sort()
                    group.commands = lst
                } else if(kind == 'container') {
                    let lst = Object.keys(cgroups)
                    lst.sort()
                    if(lst.length > 2) {
                        let shortnames = {}
                        for(let name of lst) {
                            // Only pick up a major lithos group name,
                            // if possible
                            let idx = name.indexOf(':')
                            if(idx >= 0) {
                                shortnames[name.substr(0, idx)] = true
                            } else {
                                shortnames[name] = true
                            }
                        }
                        lst = Object.keys(shortnames)
                    }
                    group.cgroups = lst
                }
            }
            group.kind = kind
            res.set(parseInt(k), group)
        }
        return res;
    }
    return state;
}

export function sockets(state={}, action) {
    switch(action.type) {
        case DATA:
            let ports = []
            for(let uid in action.data.passive) {
                let usersocks = action.data.passive[uid]
                for(let port in usersocks) {
                    ports.push([port, parseInt(uid)])
                }
            }
            ports.sort(function(a, b) { a[0] - b[0] })
            state = {
                latency: action.latency,
                ports: ports,
                by_user: action.data.by_user,
                active: action.data.active,
                passive: action.data.passive,
            }
            break;
        case ERROR:
            state = {error: action.error, ...state}
            break;
    }
    return state
}

import {DATA, ERROR} from '../middleware/request'
import {format_uptime, till_now_ms, from_ms} from '../util/time'

export function peers(state={}, action) {
    switch(action.type) {
        case DATA:
            state = {
                latency: action.latency,
                peers: action.data.peers,
            }
            break;
        case ERROR:
            state = {error: action.error, ...state}
            break;
    }
    return state
}

export function peers_with_remote(state={}, action) {
    switch(action.type) {
        case DATA:
            state = {
                latency: action.latency,
                peers: action.data.peers.filter(x => x.has_remote),
            }
            break;
        case ERROR:
            state = {error: action.error, ...state}
            break;
    }
    return state
}


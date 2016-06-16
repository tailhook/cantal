import {DATA, ERROR} from '../middleware/request'

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

import {DATA, ERROR} from '../middleware/request'


export function metrics(state={}, action) {
    switch(action.type) {
        case DATA:
            state = {
                latency: action.latency,
                metrics: action.data.metrics,
            }
            break;
        case ERROR:
            state = {error: action.error, ...state}
            break;
    }
    return state
}

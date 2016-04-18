import {DATA} from '../middleware/request'
import {METRICS} from '../middleware/local-query.js'


export function metrics(state=null, action) {
    if(action.type == METRICS) {
        return action.metrics
    }
    return state;
}

export function processes(state=null, action) {
    if(action.type == DATA) {
        return action.data;
    }
    return state;
}

import {METRICS} from '../middleware/local-query.js'


export function memory(state={}, action) {
    switch(action.type) {
        case METRICS:
            console.log("GOT MEMORY", action.metrics)
            break;
    }
}

export function cpu(state={}, action) {
    switch(action.type) {
        case METRICS:
            console.log("GOT CPU", action.metrics)
            break;
    }
}

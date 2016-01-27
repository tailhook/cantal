export function metrics(state={}, action) {
    switch(action.type) {
        case 'data':
            state = {
                latency: action.latency,
                metrics: action.payload.metrics,
            }
            break;
        case 'error':
            state = {error: action.error, ...state}
            break;
    }
    return state
}

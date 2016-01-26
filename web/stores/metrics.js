export function metrics(state={}, action) {
    switch(action.type) {
        case 'data':
            state = {
                latency: action.latency,
                metrics: action.payload.metrics,
            }
            console.log('MET', state.metrics)
            break;
        case 'error':
            state = {error: action.error, ...state}
            break;
    }
    return state
}

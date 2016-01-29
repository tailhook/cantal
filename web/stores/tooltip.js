export function tooltip(state={}, action) {
    switch(action.type) {
        case 'set':
            return {
                visible: true,
                x: action.x,
                y: action.y,
                ...action.payload
            }
        case 'hide':
            return {visible: false}
    }
    return state
}

export function show(event, item) {
    return {
        type: 'set',
        x: event.pageX,
        y: event.pageY,
        payload: item,
    }
}

export function hide() {
    return {type: 'hide'}
}

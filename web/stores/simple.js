export function bool(state=false, action) {
    switch(action.type) {
        case 'enable': return true;
        case 'disable': return false;
        default: return state;
    }
}

export function enable() {
    return { type: 'enable' }
}
export function disable() {
    return { type: 'disable' }
}
export function toggle(value) {
    if(value) {
        return disable()
    } else {
        return enable()
    }
}

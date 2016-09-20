export function entries(x) {
    return Object.keys(x).map(k => [k, x[k]])
}

export function repr(x) {
    return JSON.stringify(x)
}

export function pretty(x) {
    return JSON.stringify(x, null, 2)
}

export function is_string(x) {
    return typeof x == 'string'
}

export function reversed(x) {
    let r = x.concat();
    r.reverse();
    return r
}

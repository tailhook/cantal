export function repr(v) {
    return JSON.stringify(v)
}

export function pprint(v) {
    return JSON.stringify(v, null, 2)
}

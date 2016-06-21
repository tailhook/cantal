export function repr(v) {
    return JSON.stringify(v)
}

export function pprint(v) {
    return JSON.stringify(v, null, 2)
}

export function sum_values(obj) {
    let val = 0
    for(var i in obj) {
        val += obj[i]
    }
    return val
}

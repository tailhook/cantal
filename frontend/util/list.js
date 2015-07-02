export function last(lst) {
    return lst[lst.length-1]
}

export function from_obj(obj) {
    const res = []
    for(var k of Object.keys(obj)) {
        res.push({
            })
    }
    return res
}

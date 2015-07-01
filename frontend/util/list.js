export function from_obj(obj) {
    const res = []
    for(var k of Object.keys(obj)) {
        res.push({
            'title': k,
            'values': obj[k],
            'last': obj[k][obj[k].length-1],
            })
    }
    return res
}

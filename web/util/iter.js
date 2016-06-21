// should be generator, but requires regenerator runtime
// probably we should stop babel'ifying generators
export function pairs(obj) {
    let val = []
    for(var k in obj) {
        val.push([k, obj[k]]);
    }
    return val
}

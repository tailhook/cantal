export function add_key(key, val) {
    if(typeof val == 'object') {
        if(val.key) {
            val.key = key + ':' + val.key
        } else {
            val.key = key
        }
    }
    return val;
}

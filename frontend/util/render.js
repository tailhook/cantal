var registrations = []
var req_id = 0

export function append(el, fun) {
    var node = cito.vdom.append(el, fun);
    registrations.push({
        node: node,
        renderer: fun,
        });
}

function real_update() {
    cancelAnimationFrame(req_id)
    req_id = 0
    for(var i = 0, il = registrations.length; i < il; ++i) {
        var ob = registrations[i]
        cito.vdom.update(ob.node, ob.renderer)
    }
}

export function update() {
    if(!req_id) {
        req_id = requestAnimationFrame(real_update)
    }
}

export default exports

var registrations = [];

export function append(el, fun) {
    var node = cito.vdom.append(el, fun);
    registrations.push({
        node: node,
        renderer: fun,
        });
}

export function update() {
    for(var i = 0, il = registrations.length; i < il; ++i) {
        var ob = registrations[i];
        cito.vdom.update(ob.node, ob.renderer);
    }
}

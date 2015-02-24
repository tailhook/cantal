export function tag(tag) {
    var args = [];
    for (var _i = 1; _i < arguments.length; _i++) {
        args[_i - 1] = arguments[_i];
    }
    return { tag: tag, children: args };
}

export function tag_class(tag, classname, children) {
    return { tag: tag, className: classname, children: children };
}

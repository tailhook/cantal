export function tag(tag) {
    var args = [];
    for (var _i = 1; _i < arguments.length; _i++) {
        args[_i - 1] = arguments[_i];
    }
    return { tag: tag, children: args };
}

export function tag_class(tag, classname, children) {
    return { tag: tag, attrs: { class: classname }, children: children };
}

export function link(classname, href) {
    var args = [];
    for (var _i = 2; _i < arguments.length; _i++) {
        args[_i - 2] = arguments[_i];
    }
    return { tag: 'a', attrs: {
        class: classname,
        href: href,
        }, children: args };
}
export function icon(icon) {
    return { tag: 'span', attrs: {class: 'glyphicon glyphicon-' + icon}};
}
export function title_span(title, children) {
    return { tag: 'span', attrs: {
        title: title,
        class: "title",
        }, children: children };
}

export function tag(tag, children) {
    return { tag: tag, children: children };
}

export function tag_class(tag, classname, children) {
    return { tag: tag, attrs: { class: classname }, children: children };
}

export function link(classname, href, ...args) {
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

export function tag_key(tag, key, children) {
    return { tag: tag, key: key, children: children };
}

export function tag_map(tagname) {
    return function(list) {
        return list.map(tag.bind(null, tagname))
    }
}

export function button_xs(kind, children, handler) {
    return { tag: 'button',
        attrs: {class: 'btn btn-xs btn-'+kind},
        events: { click: handler },
        children: children };
}

export function td_left(value) {
    return tag_class('td', 'text-left', value)
}
export function td_right(value) {
    return tag_class('td', 'text-right', value)
}
export function th_left(value) {
    return tag_class('th', 'text-left', value)
}
export function th_right(value) {
    return tag_class('th', 'text-right', value)
}

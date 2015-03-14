import {Context} from 'util/base'


export function toggle(object, property) {
    const ctx = Context.current()
    return function(ev) {
        object[property] = !object[property]
        ev.preventDefault()
        ctx.refresh(object)
    }
}

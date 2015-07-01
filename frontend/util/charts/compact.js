import {format_time, from_ms} from 'util/time'

export function xaxis(timestamps, width, step=2000) {
    const ticks = []
    const tick_pixels = 60
    const now = new Date().getTime()
    const tick_step = step*tick_pixels
    let tick = Math.floor(now / tick_step) * tick_step
    let px = width - Math.floor((now - tick) / step)
    while(px > 0) {
        ticks.push({
            x: px,
            text: format_time(from_ms(tick)),
        })
        px -= 60
        tick -= tick_step
    }
    return {ticks}
}

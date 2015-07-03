import {format_time, from_ms} from 'util/time'

export function xaxis(timestamps, width, step=2000) {
    const ticks = []
    const tick_pixels = 60
    const now = new Date().getTime()
    const tick_step = step*tick_pixels
    const pixels = new Array(width)
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
    const start = Math.floor(now / step) * step
    for(var i = timestamps.length-1; i >= 0; --i) {
        let tx = timestamps[i]
        let px = width - Math.round((start - tx) / step)
        if(px < 0 || px >= width) {
            continue
        }
        if(pixels[px]) {
            //console.warn("Duplicate pixel", px, tx, start)
        }
        pixels[px] = {
            index: i,
            exact_time: tx,
        }
    }
    return {ticks, pixels, width}
}

function draw_on(canvas, xaxis, yaxis, data) {
    canvas.width = xaxis.width
    canvas.height = yaxis.height
    const ctx = canvas.getContext("2d")
    for(var i = 0, il = xaxis.pixels.length; i < il; ++i) {
        const px = xaxis.pixels[i]
        const val = px ? data[px.index] : null
        if(px == null || val == null) {
            ctx.fillStyle = yaxis.skip_color
            ctx.fillRect(i, 0, 1, yaxis.height)
            continue
        }
        let prev_thresh = 0
        let prev_color = yaxis.bg_color
        let idx = 0
        for(var [thresh, color] of yaxis.colors) {
            if(val < thresh) {
                break;
            }
            prev_thresh = thresh
            prev_color = color
            idx += 1
        }
        let h = Math.ceil((val - prev_thresh)/(thresh - prev_thresh)
                          * yaxis.height)
        ctx.fillStyle = color
        ctx.fillRect(i, yaxis.height - h, 1, h)
        ctx.fillStyle = prev_color
        ctx.fillRect(i, 0, 1, yaxis.height - h)
    }
}

export function draw(xaxis, yaxis, data) {
    return function drawer(old_elem) {
        if(old_elem) {
            draw_on(old_elem.dom, xaxis, yaxis, data)
        } else {
            return {'tag': 'canvas', 'attr': {
                'width': String(xaxis.width),
                'height': String(yaxis.height),
                }, 'events': {
                    '$created': function(ev) {
                        draw_on(ev.target, xaxis, yaxis, data)
                    }
                }}
        }
    }
}

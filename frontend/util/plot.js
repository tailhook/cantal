function linear_axis(min, max, nticks) {
    console.assert(min == 0)
    let diff = max - min
    let tick_delta = diff / nticks
    let num_decimals = - Math.floor(Math.log10(tick_delta))
    let magnitude = Math.pow(10, -num_decimals)

    let norm = tick_delta / magnitude
    let k
    if(norm < 1.5) {
        k = 1
    } else if (norm < 3) {
        k = 2
        if (norm > 2.25) {
            k = 2.5
            num_decimals += 1
        }
    } else if (norm < 7.5) {
        k = 5
    } else {
        k = 10
    }
    let tick_size = k * magnitude
    let tick_start = tick_size * Math.floor(min / tick_size)
    let num_ticks = Math.ceil((max - tick_start) / tick_size)
    let height = tick_size * num_ticks

    let ticks = []
    for(let i = 0; i < num_ticks; ++i) {
        let value = tick_start + tick_size * i
        let label
        if(num_decimals > 0) {
            label = value.toFixed(num_decimals)
        } else if(num_decimals <= -6) {
            label = (value / 1000000).toFixed(0) + "M"
        } else if(num_decimals <= -3) {
            label = (value / 1000).toFixed(0) + "k"
        } else {
            label = value.toFixed(0)
        }
        ticks[i] = {value, label}
    }

    return {
        min, tick_size, height, tick_start, num_ticks, ticks,
        max: min + height,
    }
}

function time_axis(ts) {
    let min = ts[0]
    let max = ts[ts.length-1]
    let diff = min - max
}

export class Plot {
    constructor(ts, data, width, height) {
        var xoff = ts[0].getTime()
        var max = this.max = Math.max.apply(null, data);
        var min = this.min = Math.min.apply(null, data);
        var yaxis = this.yaxis = linear_axis(0, max, 0.3 * Math.sqrt(height))
        var xaxis = this.xaxis = time_axis(ts)
        var xscale = width / (xoff - ts[data.length-1].getTime())
        var yscale = height / yaxis.height;
        var path = `M ${width}, ${height - data[0]*yscale} L`
        for(var i = 1, il = data.length; i < il; ++i) {
            path += ` ${width - (xoff - ts[i].getTime())*xscale}
                      ${height - data[i]*yscale}`
        }
        this.xscale = xscale
        this.yscale = yscale
        this.path = path
    }
}

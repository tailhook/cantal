export class Plot {
    init(timestamps, values, options={}) {
        this.width = options.width || 256
        this.height = options.height || 100
        this.timestamps = timestamps
        this.values = values
    }
    render() {
        const ts = this.timestamps
        const val = this.values
        if(!val || val.length < 2) {
            return {tag: 'div', attrs: { class: 'nodata' },
                    children: '-- no data --' }
        }
        var xoff = ts[0][0]
        var xscale = this.width / (xoff - ts[ts.length-1][0])
        var yscale = this.height / Math.max.apply(null, val)
        var path = `M ${this.width}, ${val[0]*yscale} L`
        console.log(val)
        for(var i = 1, il = val.length; i < il; ++i) {
            path += ` ${this.width - (xoff - ts[i][0])*xscale}
                      ${this.height - val[i]*yscale}`
        }

        return { tag: "svg", attrs: { style: {
                width: `${this.width}px`,
                height: `${this.height}px`,
            }}, children: [
                { tag: 'path', attrs: {
                    d: path,
                    fill: 'none',
                    style: {
                        stroke: 'black',
                        strokeWidth: '2px',
                    },
                }},
            ],
        };
    }
}

export class Sparkline {
    init(points, options={}) {
        this.width = points.length*2 + 2
        const h = this.height = options.height || 32
        const path = points
                     .map(([x, y], idx) => `${idx*2+1} ${h - y*h}`)
                     .join(' ')
        this.path = `M 1 ${h}
                     L ${path}
                     L ${this.width-2} ${h}
                     z`
    }
    render() {
        return { tag: "svg", attrs: {
            style: {
                'vertical-align': 'middle',
                width: `${this.width}px`,
                height: `${this.height}px`,
            }}, children: [
                {tag: 'path', attrs: {
                    'stroke': 'blue',
                    'fill': 'silver',
                    'd': this.path,
                }},
                {tag: 'rect', attrs: {
                    'width': this.width-2,
                    'height': this.height,
                    'stroke': 'gray',
                    'fill': 'none',
                    }},
            ],
        };
    }
}

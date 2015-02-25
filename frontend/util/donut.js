const RAD = Math.PI / 180;

// D65 standard referent
const LAB_X = 0.950470;
const LAB_Z = 1.088830;

function _lab_xyz(v) {
    return v > 0.206893034 ? v * v * v : (v - 4 / 29) / 7.787037;
}

function _xyz_rgb(v) {
    return Math.round(255 * (v <= 0.00304
        ? 12.92 * v
        : 1.055 * Math.pow(v, 1 / 2.4) - 0.055))
}

function hcl_color(h, c, l) {
    // HCL -> LAB
    h *= RAD;
    var a = Math.cos(h) * c;
    var b = Math.sin(h) * c;
    // LAB -> XYZ
    var y = (l + 16) / 116;
    var x = y + a / 500;
    var z = y - b / 200;
    x = lab_xyz(x) * LAB_X;
    y = lab_xyz(y); // * one
    z = lab_xyz(z) * LAB_Z;
    // XYZ -> RGB
    var r = _xyz_rgb( 3.2404542 * x - 1.5371385 * y - 0.4985314 * z)
    var g = _xyz_rgb(-0.9692660 * x + 1.8760108 * y + 0.0415560 * z)
    var b = _xyz_rgb( 0.0556434 * x - 0.2040259 * y + 1.0572252 * z)
    return `rgb(${r},${g},${b})`
}

function sector(cx, cy, r, sa, ea) {
    var cos = Math.cos;
    var sin = Math.sin;
    var x1 = cx + r * cos(-sa * RAD);
    var x2 = cx + r * cos(-ea * RAD);
    var xm = cx + r / 2 * cos(-(sa + (ea - sa) / 2) * RAD);
    var y1 = cy + r * sin(-sa * RAD);
    var y2 = cy + r * sin(-ea * RAD);
    var ym = cy + r / 2 * sin(-(sa + (ea - sa) / 2) * RAD);
    var large = +(Math.abs(ea - sa) > 180);
    return `M ${cx}, ${cy} L ${x1}, ${y1}
            A ${r}, ${r}, 0, ${large}, 1, ${x2}, ${y2}
            z`;
}

export class DonutChart {
    constructor(width=256, height=256) {
        this.width = 256
        this.height = 256
    }
    set_data(total, items) {
        this.total_value = total
        this.items = items
    }
    render() {
        var items = this.items;
        var total = this.total_value;
        var paths = [];
        var angle = 0;
        var cx = this.width >> 1;
        var cy = this.width >> 1;
        var r = Math.min(cx, cy) - 10;
        for(var i = 0, il = items.length; i < il; ++i) {
            var it = items[i];
            var sangle = angle;
            angle -= 360 * it.value / total;
            var path = sector(cx, cy, r, sangle, angle);
            paths.push({tag: 'path', attrs: {
                fill: it.color,
                d: path,
                }})
        }

        return { tag: "svg", attrs: { style: {
            width: '256px',
            height: '256px',
        }}, children: paths,
        };
    }
}

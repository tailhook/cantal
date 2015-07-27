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

function sector(cx, cy, r1, r2, sa, ea) {
    var c1 = Math.cos(-sa * RAD)
    var c2 = Math.cos(-ea * RAD)
    var s1 = Math.sin(-sa * RAD)
    var s2 = Math.sin(-ea * RAD)

    var x1 = cx + r2 * c1;
    var y1 = cy + r2 * s1;
    var large = +(Math.abs(ea - sa) > 180);
    return `M ${cx + r2*c1}, ${cy + r2*s1}
            A ${r2}, ${r2}, 0, ${large}, 1, ${cx + r2*c2}, ${cy + r2*s2}
            L ${cx + r1*c2}, ${cy + r1*s2}
            A ${r1}, ${r1}, 0, ${large}, 0, ${cx + r1*c1}, ${cy + r1*s1}
            z`;
}

export function with_paths(items, total, size) {
    var result = []
    var angle = 0
    var cx = size >> 1
    var cy = size >> 1
    var r = Math.min(cx, cy)
    for(var i = 0, il = items.length; i < il; ++i) {
        var it = items[i]
        if(it.value == 0 || !it.color) {
            continue;
        }
        var sangle = angle
        if(total == 0) {
            angle = sangle + 360
        } else if (it.value == total) {
            angle -= 360 * it.value / total - 0.01;
        } else {
            angle -= 360 * it.value / total;
        }
        var path = sector(cx, cy,
            // TODO(tailhook) use some interpolation
            r > 150 ? r*0.50 :
            r > 50 ? r*0.4 : r*0.2,
            r, sangle, angle)
        result.push({path: path, ...it})
    }
    return result
}

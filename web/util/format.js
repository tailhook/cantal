/*
const RE_PATTERN = /\{([a-zA-Z_0-9]+)([^:}]+)(?::([^}]+)\}/

export function format(pattern, ...replacements) {
    RE_PATTERN.sub(pattern, function(match) {
    })
}
*/

export function number_formatter(decimals=0) {
    return function(x) {
        return x.toFixed(decimals)
    }
}

export function percent_formatter(decimals=0) {
    return function(x) {
        return (x*100).toFixed(decimals) + '%'
    }
}

export function already_percent_formatter(decimals=0) {
    return function(x) {
        return x.toFixed(decimals) + '%'
    }
}

export function bytes_formatter() {
    return function(x) {
        if(x >= 10737418240) {
            return (x / 10737418240).toFixed(0) + 'Gi'
        } else if(x >= 5368709120) {
            return (x / 10737418240).toFixed(1) + 'Gi'
        } else if(x >= (10 << 19)) {
            return (x >> 20) + 'Mi'
        } else if(x >= (1 << 19)) {
            return (x / (1 << 20)).toFixed(1) + 'Mi'
        } else if(x >= (10 << 9)) {
            return (x >> 10) + 'ki'
        } else if(x >= (1 << 9)) {
            return (x / (1 << 10)).toFixed(1) + 'ki'
        } else {
            return (x | 0) + 'b'
        }
    }
}

export function integral_formatter() {
    return function(x) {
        let res = (x | 0) % 1000;
        let nlen = 3
        x = (x / 1000) | 0
        while(x > 0) {
            switch(nlen - res.length) {
                case 0: break
                case 1: res = ",0" + res; break
                case 2: res = ",00" + res; break
            }
            res = (x % 1000) + "," + res
            x = (x / 1000) | 0
            nlen += 4;
        }
        return res
    }
}

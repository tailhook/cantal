
export class Enum {
    constructor(options) {
        this.numeric_options = {}
        this.named_options = {}
        for(let k in Object.keys(options)) {
            let typ = options[k]
            let n = parseInt(k)
            if(isNaN(n)) {
                this.named_options[k] = typ
            } else {
                this.numeric_options[n] = typ
                this.named_options[typ.constructor.name] = typ
            }
        }
    }
}

export class Reflect {
}

class DecodeError extends Error {}

export function decode(typ, buf) {
    const proto = typ.probor_protocol
    const val = CBOR.decode(buf)
    return decode_value(typ, proto, val)
}

function decode_value(typ, proto, val) {
    if(proto.constructor == Enum) {
        return decode_enum(proto, val)
    } else if(proto.constructor == Reflect) {
        return new typ(val);
    } else if(typeof proto == "object") {
        return new typ(val);
    } else {
        throw Error("Not implemented type")
    }
}

function decode_enum(proto, value) {
    if(!Array.isArray(value)) throw new DecodeError("Enum decoding error");
    let typ = proto.numeric_options[value[0]] || proto.named_options[value[0]];
    return decode_value(typ, typ.probor_protocol, value[1])
}

export class SimpleStruct {
    constructor(obj) {
        for(var k of Object.keys(obj)) {
            this[k] = obj[k]
        }
    }
}

export class Proto { }

export class Enum extends Proto {
    constructor(options) {
        super()
        this.numeric_options = {}
        this.named_options = {}
        for(let k of Object.keys(options)) {
            let typ = options[k]
            let n = parseInt(k)
            if(isNaN(n)) {
                this.named_options[k] = typ
            } else {
                this.numeric_options[n] = typ
                if(typeof typ == 'string') {
                    this.named_options[typ] = typ
                } else {
                    this.named_options[typ.name] = typ
                }
            }
        }
    }
    decode(val, base_type) {
        if(!Array.isArray(val)) {
            throw new DecodeError(`Enum requires array got ${val} instead`)
        }
        let typ = this.numeric_options[val[0]] || this.named_options[val[0]];
        if(typ === undefined) {
            throw new DecodeError(
                `Unknown enum value ${val[0]} for ${base_type}`)
        } else if(typeof typ == 'string') {
            return typ
        }
        let args = []
        let proto = typ.probor_enum_protocol;
        if(proto === undefined) {
            throw new DecodeError("Enum protocol is undefined for " + typ)
        }
        for(var i = 0, il = proto.length; i < il; ++i) {
            args.push(decode_value(proto[i], val[i+1]))
        }
        return new typ(...args)
    }
}

export class Reflect extends Proto {
    decode(val, typ) {
        if(typ !== undefined) {
            return new typ(val)
        } else {
            return val
        }
    }
}

export class Dict extends Proto {
    constructor(key, value) {
        super()
        this.key = key
        this.value = value
    }
    decode(val) {
        let res = new Map()
        for(let k in val) {
            let dk = decode_value(this.key, k)
            let dv = decode_value(this.value, val[k])
            res.set(dk, dv)
        }
        return res
    }
}

export class List extends Proto {
    constructor(item) {
        super()
        this.item = item
    }
    decode(val) {
        let res = []
        for(let item of val) {
            res.push(decode_value(this.item, item))
        }
        return res
    }
}

export class Str extends Proto {
    decode(val) {
        if(typeof val != 'string')
            throw new DecodeError(`String expected got ${val}`)
        return val
    }
}

export class Int extends Proto {
    decode(val) {
        if(typeof val != 'number')
            throw new DecodeError(`Integer expected got ${val}`)
        // TODO(tailhook) should we check for mantissa
        return val
    }
}

export class Float extends Proto {
    decode(val) {
        if(typeof val != 'number')
            throw new DecodeError(`Float expected got ${val}`)
        return val
    }
}

export class Tuple extends Proto {
    constructor(...items) {
        super()
        this.items = items
    }
    decode(val) {
        if(val.length != this.items.length)
            throw new DecodeError("Tuple of wrong length")
        let res = []
        for(var i = 0; i < val.length; ++i) {
            res.push(decode_value(this.items[i], val[i]))
        }
        return res
    }
}

export class Struct extends Proto {
    constructor(fields) {
        super()
        this.fields = fields
    }
    decode(val, typ) {
        let props = {}
        // TODO(tailhook) check for required fields?
        if(Array.isArray(val)) {
            for(let [k, n, v] of this.fields) {
                let curv = val[n];
                if(curv) {
                    props[k] = decode_value(v, curv)
                }
            }
        } else {
            for(let [k, n, v] of this.fields) {
                if(n in val) {
                    props[k] = decode_value(v, val[n])
                } else if(k in val) {
                    props[k] = decode_value(v, val[k])
                }
            }
        }
        return new typ(props)
    }
}

export class Optional extends Proto {
    constructor(typ) {
        super()
        this.type = typ
    }
    decode(val, typ) {
        if(val == null) {
            return null
        } else {
            return decode_value(this.type, val)
        }
    }
}

export class DecodeError extends Error {
    constructor(...args) {
        super()
        let e = Error(...args)
        e.name = this.constructor.name
        this.message = e.message
        this.stack = e.stack
    }
}

export function decode(typ, buf) {
    const val = CBOR.decode(buf)
    return decode_value(typ, val)
}

function decode_value(typ, val) {
    //console.log("TYPE", typ, val)
    try {
        if(typ instanceof Proto) {
            return typ.decode(val)
        } else {
            return typ.probor_protocol.decode(val, typ)
        }
    } finally {
        //console.log("ENDTYPE", typ, val)
    }
}

import {
    Struct, Enum, Dict, List, Str, Int, Tuple, Simple, SimpleStruct, Optional,
    decode, Proto, Float as FloatProto
    } from '../util/probor'

import CBOR from 'cbor-js'

export const EMPTY_KEY = {}

class Key extends Proto {
    decode(val) {
        if(!val.length) {
            return EMPTY_KEY
        } else {
            return CBOR.decode(val.buffer.slice(val.byteOffset,
                                                val.byteOffset + val.byteLength))
        }
    }
}

class Timestamp extends Proto {
    decode(val) {
        const dt = new Date()
        dt.setTime(val)
        return dt
    }
}


export class BaseQuery {
    constructor(url, post_data=null,
        response_type='json', interval=2000)
    {
        this._timer = null
        this.owner_destroyed = new Stream('query_remote_destroyed');
        this.owner_destroyed.handle(this.stop.bind(this));
        this.url = url
        this.interval = interval
        this.post_data = post_data
        this.response_type = response_type
    }
    start() {
        if(this._timer) {
            clearInterval(this._timer);
        }
        this._timer = setInterval(() => this.refresh_now(), this.interval)
        this.refresh_now()
    }
    stop() {
        if(this._req) {
            this._req.abort();
            this._req = null;
        }
        if(this._timer) {
            clearInterval(this._timer);
            this._timer = 0;
        }
    }
    refresh_now() {
        if(this._req) {
            this._req.onreadystatechange = null;
            this._req.abort();
        }
        var req = this._req = new XMLHttpRequest();
        var time = new Date();
        req.onreadystatechange = (ev) => {
            if(req.readyState < 4) {
                return;
            }
            this.latency = new Date() - time;
            if(req.status != 200) {
                console.error("Error fetching", this.url, req)
                this.error = Error(`Status ${req.status}`)
                return;
            }
            var data = this.decode(req.response)
            console.log("Query", this.url, "returned", data)
            this.apply(data)
            update();
        }
        const post_data = this.post_data
        if(post_data) {
            req.open('POST', this.url, true)
            req.responseType = this.response_type;
            req.send(post_data)
        } else {
            req.open('GET', this.url, true)
            req.responseType = this.response_type;
            req.send();
        }
    }
}

let chunk = new Enum(function() {
    class State {
        constructor([ts, value]) {
            this.ts = ts
            this.value = value
        }
    }
    State.probor_enum_protocol = [new Tuple(new Timestamp(), new Str())]

    class Counter {
        constructor(values) {
            this.values = values
        }
    }
    Counter.probor_enum_protocol = [new List(new Optional(new Int()))]
    class Integer {
        constructor(values) {
            this.values = values
        }
    }
    Integer.probor_enum_protocol = [new List(new Optional(new Int()))]
    class Float {
        constructor(values) {
            this.values = values
        }
    }
    Float.probor_enum_protocol = [new List(new Optional(new FloatProto()))]

    return {
        0: State,
        1: Counter,
        2: Integer,
        3: Float,
    }}())

let tip = new Enum(function() {
    class State {
        constructor([ts, value]) {
            this.ts = ts
            this.value = value
        }
    }
    State.probor_enum_protocol = [new Timestamp(), new Str()]

    class Counter {
        constructor(value) {
            this.value = value
        }
    }
    Counter.probor_enum_protocol = [new Int()]
    class Integer {
        constructor(value) {
            this.value = value
        }
    }
    Integer.probor_enum_protocol = [new Int()]
    class Float {
        constructor(value) {
            this.value = value
        }
    }
    Float.probor_enum_protocol = [new FloatProto()]

    return {
        0: State,
        1: Counter,
        2: Integer,
        3: Float,
    }}())

class SingleSeries {
    constructor(key, chunk, timestamps) {
        this.key = key
        this.chunk = chunk
        this.timestamps = timestamps
    }
}
SingleSeries.probor_enum_protocol = [
    new Key(), chunk, new List(new Timestamp())]

class MultiSeries {
    constructor(chunks) {
        this.chunks = chunks
    }
    to_dict(prop, prefix) {
        let res = {};
        if(prefix) {
            let prefix_len = prefix.length
            for(let [key, value] of this.chunks) {
                let rkey = key[prop]
                if(rkey && rkey.substr(0, prefix.length) == prefix) {
                    res[rkey.substr(prefix.length)] = value
                } else if(key == EMPTY_KEY) {
                    res.TOTAL = value
                }
            }
        } else {
            for(let [key, value] of this.chunks) {
                res[key[prop]] = value
            }
        }
        return res
    }
}
MultiSeries.probor_enum_protocol = [new List(
    new Tuple(new Key(), chunk, new List(new Timestamp())))]

class SingleTip {
    constructor(key, value, timestamps) {
        this.key = key
        this.value = value
        this.timestamps = timestamps
    }
    delta_sec() {
        return (this.timestamps[0] - this.timestamps[1]) / 1000
    }
}
SingleTip.probor_enum_protocol = [new Key(), tip,
                                  new Tuple(new Timestamp(), new Timestamp())]

class MultiTip {
    constructor(values) {
        this.values = values
    }
    to_dict(prop, prefix) {
        let res = {};
        if(prefix) {
            let prefix_len = prefix.length
            for(let [key, value] of this.values) {
                let rkey = key[prop]
                if(rkey.substr(0, prefix.length) == prefix) {
                    res[rkey.substr(prefix.length)] = value.value
                }
            }
        } else {
            for(let [key, value] of this.values) {
                res[key[prop]] = value.value
            }
        }
        return res
    }
}
MultiTip.probor_enum_protocol = [new List(
    new Tuple(new Key(), tip, new Tuple(new Timestamp(), new Timestamp())))]

class Chart {
    constructor(chart) {
        this.chart = chart
    }
}
Chart.probor_enum_protocol = [new Dict(new Str(), new Int())]
class Empty { }
Empty.probor_enum_protocol = []
class Incompatible {
    constructor(reason) {
        this.reason = reason
    }
}
Incompatible.probor_enum_protocol = [new Enum({
    100: "CantSumChart",
    101: "Dissimilar",
    102: "CantSumTimestamps",
    103: "CantSumStates",
    104: "CantDerive",
})]

let dataset = new Enum({
    100: SingleSeries,
    101: MultiSeries,
    200: SingleTip,
    201: MultiTip,
    300: Chart,
    998: Empty,
    999: Incompatible,
})


export class QueryResponse extends SimpleStruct { }
QueryResponse.probor_protocol = new Struct([
    ["values", null, new Dict(new Str(), dataset)],
    ])

export var hosts_response = new Dict(new Str(), new Dict(new Str(), dataset))

let value = new Enum(function() {
    class State {
        constructor([ts, value]) {
            this.ts = ts
            this.value = value
        }
    }
    State.probor_enum_protocol = [new Tuple(new Timestamp(), new Str())]

    class Counter {
        constructor(v) {
            this.value = v
        }
    }
    Counter.probor_enum_protocol = [new Int()]
    class Integer {
        constructor(value) {
            this.value = value
        }
    }
    Integer.probor_enum_protocol = [new Int()]
    class Float {
        constructor(value) {
            this.value = value
        }
    }
    Float.probor_enum_protocol = [new FloatProto()]

    return {
        0: State,
        1: Counter,
        2: Integer,
        3: Float,
    }}())


export class MetricsResponse extends SimpleStruct { }
MetricsResponse.probor_protocol = new Struct([
    ["metrics", null, new List(new Tuple(new Key(), new Timestamp(), value))],
    ])

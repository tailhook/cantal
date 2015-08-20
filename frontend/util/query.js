import {from_ms} from 'util/time'
import {update} from 'util/render'
import {HTTPError} from 'util/request'
import {Stream} from 'util/streams'
import {
    Struct, Enum, Dict, List, Str, Int, Tuple, Simple, SimpleStruct, Optional,
    decode, Proto, Float as FloatProto
    } from 'util/probor'

class Key extends Proto {
    decode(val) {
        return CBOR.decode(val.buffer.slice(val.byteOffset,
                                            val.byteOffset + val.byteLength))
    }
}

class Timestamp extends Proto {
    decode(val) {
        const dt = new Date()
        dt.setTime(val)
        return val
    }
}


export class CborQuery  {
    constructor() {
        this._timer = null
        this.owner_destroyed = new Stream('query_remote_destroyed')
            .handle(this.stop.bind(this));
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
            try {
                var data = decode(this.schema, req.response)
            } catch(e) {
                console.error("Error parsing cbor at", this.url, e.stack)
                this.error = Error("Bad Json")
                return;
            }
            this.apply(data)
            update();
        }
        const post_data = this.post_data
        if(post_data) {
            req.open('POST', this.url, true)
            req.responseType = "arraybuffer";
            req.send(post_data)
        } else {
            req.open('GET', this.url, true)
            req.responseType = "arraybuffer";
            req.send();
        }
    }
}

export class QueryRemote extends CborQuery {
    constructor(rules) {
        super()
        this.rules = rules
        this.url = '/remote/query_by_host.cbor'
        this.interval = 5000
        this.post_data = JSON.stringify({
            'rules': this.rules,
        })
        this.start()
    }
    apply(obj) {
        this.response = obj
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
    constructor(chunk) {
        this.chunk = chunk
    }
}
SingleSeries.probor_enum_protocol = [
    new Key(), chunk, new List(new Timestamp())]

class MultiSeries {
    constructor(chunks) {
        this.chunks = chunks
    }
}
MultiSeries.probor_enum_protocol = [new List(
    new Tuple(new Key(), chunk, new List(new Timestamp())))]

class SingleTip {
    constructor(key, value) {
        this.key = key
        this.value = value
    }
}
SingleTip.probor_enum_protocol = [new Key(), tip]

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
MultiTip.probor_enum_protocol = [new List(new Tuple(new Key(), tip))]

class Chart {
    constructor(chart) {
        this.chart = chart
    }
}
Chart.probor_enum_protocol = [new Dict(new Str(), new Int())]

let dataset = new Enum({
    100: SingleSeries,
    101: MultiSeries,
    200: SingleTip,
    201: MultiTip,
    300: Chart,
})

class QueryResponse extends SimpleStruct { }
QueryResponse.probor_protocol = new Struct([
    ["values", null, new Dict(new Str(), dataset)],
    ])


export class Query extends CborQuery {
    constructor(interval, rules) {
        super()
        this.rules = rules
        this.url = '/query.cbor'
        this.interval = interval || 5000
        this.post_data = JSON.stringify({
            'rules': this.rules,
        })
        this.schema = QueryResponse
        this.start()
    }
    apply(response) {
        this.values = response.values
    }
}

export class RemoteStats extends CborQuery {
    constructor(interval) {
        super()
        this.url = '/remote_stats.json'
        this.interval = interval || 5000
        this.start()
    }
    apply(response) {
        this.values = response.values
    }
}

export class PeersRequest extends CborQuery {
    constructor(only_remote, interval) {
        super()
        this.url = '/peers_with_remote.json'
        this.interval = interval || 5000
        this.start()
    }
    apply(json) {
        this.peers = json.peers
    }
}

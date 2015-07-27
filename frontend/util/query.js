import {from_ms} from 'util/time'
import {update} from 'util/render'
import {HTTPError} from 'util/request'
import {Stream} from 'util/streams'



export class JsonQuery  {
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
                var json = JSON.parse(req.responseText)
            } catch(e) {
                console.error("Error parsing json at", this.url, e)
                this.error = Error("Bad Json")
                return;
            }
            if(!json || typeof(json) != "object") {
                console.error("Returned json is not an object", this.url, req);
                this.error = Error("Bad Json")
                return;
            }
            this.apply(json)
            update();
        }
        req.open('POST', this.url, true)
        req.send(this.post_data)
    }
}

export class QueryRemote extends JsonQuery {
    constructor(rules) {
        super()
        this.rules = rules
        this.url = '/remote/query_by_host.json'
        this.interval = 5000
        this.post_data = JSON.stringify({
            'rules': this.rules,
        })
        this.start()
    }
    apply(json) {
        const obj = {}
        for(let i in json) {
            const old = json[i]
            obj[i] = {
                "fine_timestamps": old.fine_timestamps.map(from_ms),
                "fine_metrics": old.fine_metrics,
                }
        }
        this.response = obj
    }
}


export class RefreshJson {
    constructor(url, options={}) {
        this.url = url;
        this.interval = options.interval || 2000;
        this.post_body = options.post_body || null;
    }
    set_handler(fun) {
        this.handler = fun
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
    replace_with(other) {
        if(other.url != this.url || other.interval != this.interval
            || other.post_body != this.post_body) {
            this.stop()
            other.start()
            return other
        } else {
            return this
        }
    }
    refresh_now() {
        if(this._req) {
            this._req.abort();
        }
        var req = this._req = new XMLHttpRequest();
        var time = new Date();
        req.onreadystatechange = (ev) => {
            if(req.readyState < 4) {
                return;
            }
            var lcy = new Date() - time;
            if(req.status != 200) {
                console.error("Error fetching", this.url, req);
                this.handler(Error(`Status ${req.status}`), lcy);
                return;
            }
            try {
                var json = JSON.parse(req.responseText);
            } catch(e) {
                console.error("Error parsing json at", this.url, e);
                this.handler(Error("Bad Json"), lcy);
                return;
            }
            if(!json || typeof(json) != "object") {
                console.error("Returned json is not an object", this.url, req);
                this.handler(Error("Bad Json"), lcy);
                return;
            }
            this.handler(json, lcy);
        }
        if(this.post_body) {
            req.open('POST', this.url, true);
            req.send(this.post_body)
        } else {
            req.open('GET', this.url, true);
            req.send()
        }
    }
}

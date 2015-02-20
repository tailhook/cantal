export class RequestError extends Error {
    constructor(err) {
        if(err.status) {
            this.short_name = "E" + err.status
            this.description = `${err.status} ${err.status_text}`
        } else {
            this.short_name = err.message;
            this.description = err.toString();
            this.error = err;
        }
        this.time = new Date()
    }
}

export class RefreshJson {
    constructor(url, timeout=2000) {
        this.url = url;
        this.timeout = timeout;
        this._timer = null
        riot.observable(this)
        this._start_request()
    }
    _start_request() {
        fetch(this.url).then((resp) => {
            if(resp.status == 200) {
                return resp.json()
            } else {
                console.error("Error fetching json:",
                    resp.status, resp.statusText)
                throw new RequestError(resp)
            }
        }).then((value) => {
            this.start_timer()
            this.trigger("json_update", value)
        }).catch((err) => {
            this.start_timer()
            this.trigger("json_error", new RequestError(err))
        })
    }
    start_timer() {
        if(this._timer) {
            clearTimeout(this._timer)
        }
        this._timer = setTimeout(()=> this._start_request(), this.timeout)
    }

}


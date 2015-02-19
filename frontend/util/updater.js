export class RequestError extends Error {
    constructor(response) {
        this.status = response.status
        this.status_text = response.statusText
        this.time = new Date()
    }
}

export class RefreshJson {
    constructor(url, timeout=2000) {
        console.log("Instance", this)
        this.url = url;
        this.timeout = timeout;
        this._timer = null
        riot.observable(this)
        this._start_request()
    }
    _start_request() {
        console.log("Started", this)
        fetch(this.url).then((resp) => {
            this.start_timer()
            if(resp.status == 200) {
                return resp.json()
            } else {
                console.error("Error fetching json:",
                    resp.status, resp.statusText)
                throw new RequestError(resp)
            }
        }).then((value) => {
            this.trigger("json_update", value)
        }).catch((err) => {
            this.trigger("json_error", err)
        })
    }
    start_timer() {
        if(this._timer) {
            clearTimeout(this._timer)
        }
        this._timer = setTimeout(()=> this._start_request, this.timeout)
    }

}


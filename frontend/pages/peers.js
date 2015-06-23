import {Component} from 'util/base'
import {Stream} from 'util/streams'
import {RefreshJson} from 'util/request'
import peers from 'templates/peers.mft'

export class Peers extends Component {
    constructor() {
        super()
        this.add_host = new Stream("add_host")
        this.add_host.handle(this.call_add_host.bind(this))
    }
    init() {
        this.guard('json', new RefreshJson('/all_metrics.json',
                                           {interval: 120000}))
        .process((data, latency) => {
            let error = null;
            if(data instanceof Error) {
                error = data
            } else {
            }
            return {error, data: data, latency}
        })
    }
    render() {
        return peers.render(this.data, this)
    }
    call_add_host(value) {
        console.log("VALUE", value)
    }
}

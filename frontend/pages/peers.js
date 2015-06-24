import {Component} from 'util/base'
import {Stream} from 'util/streams'
import {RefreshJson, Submit} from 'util/request'
import peers from 'templates/peers.mft'

export class Peers extends Component {
    constructor() {
        super()
        this.add_host = new Stream("add_host")
        this.add_host.handle(this.call_add_host.bind(this))
    }
    init() {
        this.guard('json', new RefreshJson('/all_peers.json',
                                           {interval: 5000}))
        .process((data, latency) => {
            let error = null;
            let peers = null;
            if(data instanceof Error) {
                error = data
            } else {
                peers = data
            }
            return {error, peers, latency}
        })
    }
    render() {
        return peers.render(this.peers, this)
    }
    call_add_host(value) {
        this.last_add = {progress: true}
        this.guard('add_host', new Submit('/add_host.json', {
            'ip': value,
        })).process((data, latency) => {
            return {last_add: (data instanceof Error)
                        ? {result: 'error', error: data}
                        : {result: 'success'}}
        })
    }
}

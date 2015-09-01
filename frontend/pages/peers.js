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
            let peers = this.peers;
            if(data instanceof Error) {
                error = data
            } else {
                peers = data.peers
            }
            console.log("Peers", peers);
            return {error, peers, latency}
        })
    }
    render() {
        return peers.render(this.peers, this)
    }
    call_add_host(value) {
        this.last_add = {progress: true}
        if(value.indexOf(':') < 0) {
            value = value + ":" + (location.port || "22682");
        }
        this.guard('add_host', new Submit('/add_host.json', {
            'addr': value,
        })).process((data, latency) => {
            return {last_add: (data instanceof Error)
                        ? {result: 'error', error: data}
                        : {result: 'success'}}
        })
    }
}

import {Component} from 'util/base'
import {Stream} from 'util/streams'
import {RefreshJson, Submit} from 'util/request'
import remote from 'templates/remote.mft'
import websock from 'util/websock'

export class Remote extends Component {
    constructor() {
        super()
        this.enable_remote = new Stream("enable_remote");
        this.enable_remote.handle(this.call_enable_remote.bind(this));
        this.peer_map = {}
    }
    call_enable_remote(value) {
        this.guard('add_host', new Submit('/start_remote.json', ''))
        .process((data, latency) => {})
    }
    init() {
        this.guard('json', new RefreshJson('/all_peers.json',
                                           {interval: 60000}))
        .process((data, latency) => {
            let error = null
            let peer_map = {}
            if(data instanceof Error) {
                error = data
                peer_map = this.peer_map
            } else {
                for(var peer of data.peers) {
                    const name = peer.name || peer.hostname ||
                        peer.addr.split(':')[0]
                    peer_map[peer.id] = name
                }
            }
            return {error, enabled: data.enabled, peer_map: peer_map, latency}
        })
    }
    render() {
        return remote.render(websock.remote_enabled(),
            this.peer_map, this)
    }
}

import {Component} from 'util/base'
import {Stream} from 'util/streams'
import {RefreshJson, Submit} from 'util/request'
import remote from 'templates/remote.mft'

export class Remote extends Component {
    constructor() {
        super()
        this.enable_remote = new Stream("enable_remote");
        this.enable_remote.handle(this.call_enable_remote.bind(this));
    }
    call_enable_remote(value) {
        this.guard('add_host', new Submit('/start_remote.json', ''))
        .process((data, latency) => {})
    }
    init() {
        this.guard('json', new RefreshJson('/remote_stats.json',
                                           {interval: 5000}))
        .process((data, latency) => {
            let error = null;
            let peers = this.peers;
            if(data instanceof Error) {
                error = data
            } else {
                peers = data.peers
            }
            return {error, enabled: data.enabled, peers, latency}
        })
    }
    render() {
        return remote.render(this.enabled, this.peers, this.error, this)
    }
}

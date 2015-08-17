import {component, Component} from 'util/base'
import {RefreshJson} from 'util/request'
import navbar from 'templates/navbar.mft'


export class Navbar extends Component {
    init() {
        //this._memory_donut = new DonutChart(32, 32)
        this.guard('status', new RefreshJson("/status.json"))
        .process((data, latency) => {
            let error = null;
            if(data instanceof Error) {
                return {error: data, latency}
            } else {
                return {data, error, latency}
            }
        })
    }
    render() {
        return navbar.render(this.data, this.latency, this.error,
            window.location.hash.substr(2)); // TODO(tailhook) better hash
    }
}

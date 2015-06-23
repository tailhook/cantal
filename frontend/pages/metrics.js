import {Component} from 'util/base'
import {RefreshJson} from 'util/request'
import metrics from 'templates/metrics.mft'

export class Metrics extends Component {
    init() {
        this.guard('json', new RefreshJson('/all_metrics.json',
                                           {interval: 120000}))
        .process((data, latency) => {
            let error = null;
            if(data instanceof Error) {
                error = data
            } else {
                data.sort((a, b) => a.metric > b.metric ? 1 : a.metric < b.metric ? -1 : 0)
            }
            return {error, metrics: data, latency}
        })
    }
    render() {
        console.log("DATA", this.metrics)
        return metrics.render(this.metrics)
    }
}

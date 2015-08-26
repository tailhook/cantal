import {Component} from 'util/base'
import {RefreshJson} from 'util/request'
import metrics from 'templates/metrics.mft'

export class Metrics extends Component {
    init() {
    }
    render() {
        return metrics.render()
    }
}

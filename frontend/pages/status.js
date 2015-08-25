import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {Component, component} from 'util/base'
import {toggle} from 'util/events'
import {Plot} from 'util/plot'
import {RefreshJson} from 'util/request'
import template from 'templates/status.mft'
import {cpu_chart, mem_chart} from 'util/compute'


export class Status extends Component {
    constructor() {
        super()
    }
    init(elem) { }
    render() {
        return template.render()
    }
}

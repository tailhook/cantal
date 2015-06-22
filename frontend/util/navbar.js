import {component, Component} from 'util/base'
import {RefreshJson} from 'util/request'
import navbar from 'templates/navbar.mft'


function nav(classname, href, ...args) {
    if(href == window.location.hash) {
        classname += ' active'
    }
    return link(classname, href, ...args)
}

function cpu_graph_data(data) {
    const user = data.cpu_user.fine
    const nice = data.cpu_nice.fine
    const idle = data.cpu_idle.fine
    const system = data.cpu_system.fine
    const cpu_graph = []
    let prev_use
    let prev_total
    for(var i = data.history_timestamps.length-1; i >= 0; --i) {
        const use = user[i] + nice[i] + system[i]
        const total = use + idle[i]
        if(prev_total) {
            cpu_graph.push([
                data.history_timestamps[i][0],
                (use - prev_use) / (total - prev_total),
                ])
        }
        prev_use = use
        prev_total = total
    }
    return cpu_graph
}

function memory_graph_data(d) {
    return {total: d.mem_total, items: [
        {color: '#e5f5f9', title: 'Free', value: d.mem_free},
        {color: '#99d8c9', title: 'Free', value: d.mem_buffers},
        {color: '#2ca25f', title: 'Free', value: d.mem_cached},
        {color: '#a0a0a0', title: 'Free', value:
            d.mem_total - d.mem_free - d.mem_buffers - d.mem_cached},
    ]}
}


export class Navbar extends Component {
    init() {
        //this._memory_donut = new DonutChart(32, 32)
        this.guard('status', new RefreshJson("/status.json"))
        .process((data, latency) => {
            let error = null;
            if(data instanceof Error) {
                return {error: data, latency}
            } else {
                return {data, error, latency,
                    //cpu_chart: cpu_graph_data(data),
                    //memory_chart: memory_graph_data(data),
                    }
            }
        })
    }
    render() {
        return navbar.render(this.data, this.latency, this.error,
            window.location.hash.substr(2)); // TODO(tailhook) better hash
    }
}

import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map,
        td_left, td_right, th_left, th_right,
        } from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {Component, component} from 'util/base'
import {Chart} from 'util/chart'
import {DonutChart} from 'util/donut'
import {RefreshJson} from 'util/request'

const COLORS = [
    "#4D4D4D",  // (gray)
    "#5DA5DA",  // (blue)
    "#FAA43A",  // (orange)
    "#60BD68",  // (green)
    "#F17CB0",  // (pink)
    "#B2912F",  // (brown)
    "#B276B2",  // (purple)
    "#DECF3F",  // (yellow)
    "#F15854",  // (red)
]


class StateText extends Component {
    init(title, value) {
        this.title = title
        this.value = value
    }
    render() {
        return {children: [
            h('h2', this.title),
            h('p', this.value),
        ]}
    }
}

function aggregate(data) {
    var start = new Date();
    var states = {}
    for(var item of data.latest) {
        var [dim, metric] = item;
        if(dim.state && dim.state.indexOf('.') > 0) {
            var stchunks = dim.state.split('.')
            var sub = stchunks.pop()
            var stname = stchunks.join('.')
            var st = states[stname]
            if(!st) {
                states[stname] = st = {
                    counters: {},
                    durations: {},
                    states: {},
                }
            }
            if(dim.metric == 'count') {
                st.counters[sub] = (st.counters[sub] || 0) + metric
            } else if(dim.metric == 'duration') {
                st.durations[sub] = (st.durations[sub] || 0) + metric
            }
        }
        if(dim.state && metric.length !== undefined && metric[0] != 0) {
            var st = states[dim.state]
            if(!st) {
                states[dim.state] = st = {
                    counters: {},
                    durations: {},
                    states: {},
                }
            }
            var state = metric[1];
            st.states[state] = (st.states[state] || 0) + 1
            st.durations[state] = (st.durations[state] || 0) +
                till_now_ms(from_ms(metric[0]))
        }
    }

    var charts = []
    for(var name in states) {
        var state = states[name];
        var keys = Object.keys(state.durations);
        if(keys.length > 1) {
            var items = [];
            var total = 0;
            var dur = states[name].durations;
            var colors = COLORS.concat();
            for(var k in dur) {
                const val = dur[k]
                items.push({
                    'title': k,
                    value: dur[k],
                    color: colors.pop(),
                    })
                total += val
            }
            const chart = {total, items, title: name, unit: 'ms'};
            charts.push(chart)
        } else {
            charts.push({title: name, text: keys[0]})
        }
    }
    charts.sort((a, b) => a.title.localeCompare(b.title))

    return charts
}


export class Totals extends Component {
    constructor() {
        super()
        this.charts = []
    }
    init() {
        this.guard('json', new RefreshJson("/states.json"))
        .process((data, latency) => {
            var error = null;
            if(data instanceof Error) {
                error = data;
            }
            return {error, latency, charts: aggregate(data)}
        });
    }
    render() {
        return hc("div", "container", [
            h("h1", ["States"]),
            hc("div", "text-right",
                this.error
                   ? 'Error: ' + this.error.message
                   : `Fetched in ${this.latency}ms`),
        ].concat(this.charts.map((item) => {
            if(item.hasOwnProperty('text')) {
                return component(StateText, item.title, item.text)
            } else {
                return component(Chart, component(DonutChart, item), item)
            }
        })));
    }
}

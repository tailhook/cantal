import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map,
        td_left, td_right, th_left, th_right,
        } from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
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


export class Totals {
    mount(elem) {
        this._charts = {}
        this._node = cito.vdom.append(elem, () => this.render());
        this._refresher = new RefreshJson("/values.json", (data, latency) => {
            this.latency = latency;
            if(data instanceof Error) {
                this.error = data;
            } else {
                this.data = this._aggregate(data);
                this.error = null;
            }
            this.update()
        });
        this._refresher.start()
    }
    _aggregate(data) {
        var start = new Date();

        var states = {}
        for(var item of data.items) {
            for(var pair of item.values) {
                var [dim, metric] = pair;
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
                        st.counters[sub] = (st.counters[sub] || 0) +
                            metric.fields[0]
                    } else if(dim.metric == 'duration') {
                        st.durations[sub] = (st.durations[sub] || 0) +
                            metric.fields[0]
                    }
                }
                if(dim.state && metric.variant == 'State' &&
                   metric.fields[0] > 0) {
                    var st = states[dim.state]
                    if(!st) {
                        states[stname] = st = {
                            counters: {},
                            durations: {},
                            states: {},
                        }
                    }
                    var state = metric.fields[1];
                    st.states[state] = (st.states[state] || 0) + 1
                    st.durations[state] = (st.durations[state] || 0) +
                        till_now_ms(from_ms(metric.fields[0]))
                }
            }
        }

        var newcharts = {}
        for(var name in states) {
            var chart = this._charts[name]
            if(!chart) {
                chart = new DonutChart()
            }
            var items = [];
            var total = 0;
            var dur = states[name].durations;
            var colors = COLORS.concat();
            for(var k in dur) {
                const val = dur[k]
                items.push({'title': name, value: dur[k], color: colors.pop()})
                total += val
            }
            if(total != 0) {
            chart.set_data({total, items})
            newcharts[name] = chart;
            }
        }
        this._charts = newcharts

        this._process_time = new Date() - start
    }
    render_chart(name) {
        return {children:[
            h('h2', name),
            this._charts[name].render(),
        ]}
    }
    render() {
        return hc("div", "container", [
            h("h1", ["States"]),
            hc("div", "text-right",
                this.error
                   ? 'Error: ' + this.error.message
                   : `Fetched in ${this.latency}ms / ${this._process_time}ms`),
        ].concat(
            this._charts
                ? Object.keys(this._charts).map(this.render_chart.bind(this))
                : []
        ));
    }
    update() {
        cito.vdom.update(this._node, this.render())
    }
    remove() {
        cito.vdom.remove(this._node);
    }
}

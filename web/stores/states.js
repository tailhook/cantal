import {METRICS} from '../middleware/local-query.js'

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

const FORMAT = {
    duration: x => x + ' ms',
    count: x => x,
}

export function charts(state=new Map(), action) {
    if(action.type == METRICS) {
        let index = {}
        let result = new Map()
        action.metrics.values.sort((x, y) => {
            let xs = x[0].state;
            let ys = y[0].state;
            if(xs < ys) return -1;
            if(xs > ys) return 1;
            return 0;
        })
        for(let met of action.metrics.values) {
            let chunks = met[0].state.split('.');
            let item = chunks.pop();
            let base = chunks.join('.');

            let value = met[1].value
            let metricname = met[0].metric;
            let key = met[0].state + metricname;

            if(key in index) {
                let {metric, itemobj} = index[key];
                metric.total += value
                itemobj.value += value
                itemobj.text = FORMAT[metricname](itemobj.value)
            } else {
                let state = result.get(base)
                if(!state) {
                    state = {
                        duration: { items: [], total: 0 },
                        count: { items: [], total: 0 },
                    }
                    result.set(base, state)
                }
                let metric = state[metricname]
                let itemobj = {
                    title: item,
                    value: value,
                    text: FORMAT[metricname](value),
                    color: COLORS[metric.items.length],
                }
                metric.items.push(itemobj)
                metric.total += value
                index[key] = {metric, itemobj}
            }
        }
        return result;
    }
    return state;
}

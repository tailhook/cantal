import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map,
        td_left, td_right, th_left, th_right,
        } from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {RefreshJson} from 'util/request'
import {Component} from 'util/base'


export class Processes extends Component {
    constructor() {
        super()
        this.open_items = {};
    }
    init() {
        this.guard('json', new RefreshJson("/all_processes.json", 5000))
        .process((data, latency) => {
            if(data instanceof Error) {
                return {error: data, latency}
            } else {
                const res = this.build_tree(data)
                res['error'] = null
                res['latency'] = latency
                return res
            }
        })
    }
    build_tree(data) {
        var toplevel = [];
        var by_id = {};
        var tree = {};
        var old_open = this.open_items;
        var new_open = {};  // only existing pids
        for(var p of data.all) {
            by_id[p.pid] = p;
            var lst = tree[p.ppid];
            if(lst === undefined) {
                lst = tree[p.ppid] = [];
            }
            lst.push(p);
            if(p.ppid == 1) {
                toplevel.push(p);
            }
            if(p.pid in old_open) {
                new_open[p.pid] = true;
            }
        }
        return {
            all: data.all,
            uptime_base: data.boot_time,
            open_items: new_open,
            toplevel,
            tree,
        }
    }
    render_process(level=0, process) {
        var children = this.tree[process.pid];
        var is_open = this.open_items[process.pid];
        var head = hk("tr", process.pid, [
            td_left([
                {tag: 'div', attrs: {
                    style: {display: 'inline-block', width: `${16*level}px`}}},
                (children
                    ? is_open
                        ? button("default", [icon("minus"), ` ${children.length}`],
                            () => {
                                delete this.open_items[process.pid]
                                this.update()
                            })
                        : button("default", [icon("plus"), ` ${children.length}`],
                            () => {
                                this.open_items[process.pid] = true
                                this.update()
                            })
                    : ""),
                ' ' + process.pid.toString(),
            ]),
            td_left(title(process.cmdline.split('\u{0000}').join(' '),
                     [process.name.toString()])),
            td_left(format_uptime(till_now_ms(from_ms(
                    process.start_time + this.uptime_base*1000)))),
            td_right((process.rss / 1048576).toFixed(1)),
        ]);
        if(children && this.open_items[process.pid]) {
            var ch = children.map(this.render_process.bind(this, level+1))
            ch.splice(0, 0, head)
            return {children: ch}
        } else {
            return head;
        }
    }
    render_processes() {
        return hc("table", "table table-hover", [
            h("thead", h("tr", [
                th_left('pid'),
                th_left('name'),
                th_left('uptime'),
                th_right('mem (MiB)'),
                ])),
            h("tbody", this.toplevel.map(this.render_process.bind(this, 0))),
        ]);
    }
    render() {
        return hc("div", "container", [
            h("h1", ["All Processes"]),
            hc("div", "text-right",
                this.error ? 'Error: ' + this.error.message
                           : `Fetched in ${this.latency}ms`),
                this.all ? this.render_processes() : "",
            ]);
    }
}

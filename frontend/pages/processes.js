import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map,
        td_left, td_right, th_left, th_right,
        } from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {RefreshJson} from 'util/request'


export class Processes {
    mount(elem) {
        this._node = cito.vdom.append(elem, () => this.render());
        this._refresher = new RefreshJson("/all_processes.json",
            (data, latency) => {
                this.latency = latency;
                if(data instanceof Error) {
                    this.error = data;
                } else {
                    this.set_data(data);
                    this.error = null;
                }
                this.update()
            }, 5000);
        this._refresher.start();
        this.open_items = {};
    }
    set_data(data) {
        this.uptime_base = data.boot_time;
        this.all = data.all;
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
        this.open_items = new_open;
        this.toplevel = toplevel;
        this.tree = tree;
    }
    update() {
        cito.vdom.update(this._node, this.render())
    }
    remove() {
        cito.vdom.remove(this._node);
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

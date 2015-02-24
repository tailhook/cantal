import {tag_class as hc, tag as h, link, icon, button_xs as button,
        title_span as title, tag_key as hk, tag_map} from 'util/html'
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
    render_process(process) {
        var children = this.tree[process.pid];
        var is_open = this.open_items[process.pid];
        var head = hk("tr", process.pid, tag_map('td')([
            (children
                ? is_open
                    ? button("default", [icon("minus"), ` ${children.length}`],
                        () => delete this.open_items[process.pid])
                    : button("default", [icon("plus"), ` ${children.length}`],
                        () => this.open_items[process.pid] = true)
                : ""),
            process.pid.toString(),
            process.name.toString(),
            process.cmdline.split('\u{0000}').join(' '),
            ]));
        if(children && this.open_items[process.pid]) {
            var ch = children.map(this.render_process.bind(this))
            ch.splice(0, 0, head);
            return {children: ch};
        } else {
            return head;
        }
    }
    render_processes() {
        return hc("table", "table", [
            h("thead", h("tr", tag_map('th')([
                '',
                'pid',
                'name',
                'command-line',
                ]))),
            h("tbody", this.toplevel.map(this.render_process.bind(this))),
        ]);
    }
    render() {
        return hc("div", "container", [
            hc("div", "text-right",
                this.error ? 'Error: ' + this.error.message
                           : `Fetched in ${this.latency}ms`),
                this.all ? this.render_processes() : "",
            ]);
    }
}

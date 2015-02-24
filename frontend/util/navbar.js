import {tag_class as hc, tag as h, link, icon,
        title_span as title} from 'util/html'
import {format_uptime, till_now_ms, from_ms} from 'util/time'
import {RefreshJson} from 'util/request'

export class Navbar {
    constructor() {
    }
    mount(elem) {
        this._node = cito.vdom.append(elem, () => this.render());
        this._refresher = new RefreshJson("/status.json", (data, latency) => {
            this.latency = latency;
            if(data instanceof Error) {
                this.error = data;
            } else {
                this.data = data;
                this.error = null;
            }
            this.update()
        });
        this._refresher.start();
    }
    update() {
        var value = this.render();
        cito.vdom.update(this._node, value)
    }
    render_self() {
        var stats = this.data;
        return hc('span', '', [
            title("Uptime of the cantal agent itself", ['up ',
                format_uptime(till_now_ms(from_ms(stats.startup_time))) ]),
            ' / ',
            title("Latency of requests to the cantal",
                  [ this.latency.toFixed(0), 'ms']),
            ' / ',
            title("Time it takes for cantal to read all stats once",
                  [ stats.scan_time.toString(), 'ms']),
        ]);
    }
    render_machine() {
        var mach = this.data.machine;
        return hc('span', '', [
            title("Minute Load Average", [ mach.load_avg_1min.toFixed(2) ]),
            ' / ',
            title("5 Minutes Load Average",
                [ mach.load_avg_5min.toFixed(2) ]),
            ' / ',
            title("15 Minutes Load Average",
                [ mach.load_avg_15min.toFixed(2) ]),
            ' / ',
            title("Uptime of the box running cantal", [
                'up ', format_uptime(mach.uptime*1000) ]),
        ]);
    }
    render() {
        return hc("div", "navbar navbar-default", [
            hc('div', 'container-fluid', [
                hc('div', 'navbar-header', [
                    hc('a', 'navbar-brand', "Cantal"),
                ]),
                hc('div', 'collapse navbar-collapse', [
                    hc('ul', 'nav navbar-nav', [
                        hc('li', '', [
                            link("", "#/processes", "Processes"),
                        ]),
                    ]),
                    hc('form',
                        'navbar-form navbar-right' +
                            (this.error ? ' bg-danger': ''), [
                        '( ',
                            icon('hdd'), ' ',
                            this.data && this.render_machine() || "",
                        ' ) ( ',
                            icon('scale'), ' ',
                            this.data && this.render_self() || "",
                        ' ) ',
                        link('btn btn-default', '/status', 'Status'),
                    ]),
                ]),
            ]),
        ]);
    }
}

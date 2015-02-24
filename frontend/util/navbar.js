import {tag_class as hc} from 'util/html'

export class Navbar {
    constructor() {
        this.counter = 0;
    }
    init(ctx) {
    }
    render(ctx, me) {
        me.tag = "div";
        me.className = "navbar navbar-default"
        me.children = [
            hc('div', 'container-fluid', [
                hc('div', 'navbar-header', [
                    hc('a', 'navbar-brand', "Cantal"),
                    hc('a', 'navbar-brand', this.counter.toString()),
                ]),
                hc('div', 'collapse navbar-collapse', [
                    hc('ul', 'nav navbar-nav', [
                        hc('li', '', [
                            b.link(hc("a", "", "Processes"), "processes"),
                        ]),
                    ]),
                ]),
            ]),
        ]
    }
}

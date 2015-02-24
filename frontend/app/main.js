import {tag as h} from 'util/html'
import {Navbar} from 'util/navbar'
import {Processes} from 'util/processes'


export class App {
    constructor() {
        this.navbar = new Navbar()
    }
    static start() {
        var app = new App();
        b.routes(b.route({ handler: app }, [
            b.route({ name: "processes", handler: new Processes() }),
        ]));
    }
    render(ctx, me) {
        me.tag = "div";
        me.children = [
            {component: this.navbar},
            me.data.activeRouteHandler()
        ];
    }
}

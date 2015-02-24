import {tag as h} from 'util/html'
import {Navbar} from 'util/navbar'
import {Processes} from 'util/processes'


export class App {
    constructor() {
        this.navbar = new Navbar()
    }
    static start() {
        var app = new App();
        app.navbar.mount(document.body)
    }
}

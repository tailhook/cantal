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
        if(window.location.hash == '#/processes') {
            app.page = new Processes();
            app.page.mount(document.body)
        }
        window.onhashchange = function() {
            if(app.page) {
                app.page.remove();
            }
            if(window.location.hash == '#/processes') {
                app.page = new Processes();
                app.page.mount(document.body)
            }
        }
    }
}

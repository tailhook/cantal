import {tag as h} from 'util/html'
import {Navbar} from 'util/navbar'

import {Processes} from 'pages/processes'
import {Status} from 'pages/status'
import {Values} from 'pages/values'
import {Totals} from 'pages/totals'


export class App {
    constructor() {
        this.navbar = new Navbar()
    }
    static start() {
        var app = new App();
        app.navbar.mount(document.body)
        window.onhashchange = function() {
            if(app.page) {
                app.page.remove()
                app.page = null
            }
            if(window.location.hash == '#/processes') {
                app.page = new Processes()
                app.page.mount(document.body)
            } else if(window.location.hash == '#/status') {
                app.page = new Status()
                app.page.mount(document.body)
            } else if(window.location.hash == '#/values') {
                app.page = new Values()
                app.page.mount(document.body)
            } else if(window.location.hash == '#/totals') {
                app.page = new Totals()
                app.page.mount(document.body)
            }
            app.navbar.update()
        }
        window.onhashchange()
    }
}

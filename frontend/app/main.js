import {tag as h} from 'util/html'
import {Navbar} from 'util/navbar'
import {Context} from 'util/base'

import {Processes} from 'pages/processes'
import {Status} from 'pages/status'
import {Values} from 'pages/values'
import {Totals} from 'pages/totals'


export class App {
    constructor() {
        this.navbar = new Context(new Navbar())
    }
    static start() {
        console.log("BODY", document.body)
        var app = new App();
        app.navbar.mount(document.body)
        window.onhashchange = function() {
            if(app.page) {
                app.page.remove()
                app.page = null
            }
            if(window.location.hash == '#/processes') {
                app.page = new Context(new Processes())
                app.page.mount(document.body)
            } else if(window.location.hash == '#/status') {
                app.page = new Context(new Status())
                app.page.mount(document.body)
            } else if(window.location.hash == '#/values') {
                app.page = new Context(new Values())
                app.page.mount(document.body)
            } else if(window.location.hash == '#/totals') {
                app.page = new Context(new Totals())
                app.page.mount(document.body)
            }
            app.navbar.refresh()
        }
        window.onhashchange()
    }
}

App.start()

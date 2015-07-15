import {component} from 'util/base'
import websock from 'util/websock'
import {Context} from 'util/base'

import {Processes} from 'pages/processes'
import {Status} from 'pages/status'
import {Values} from 'pages/values'
import {Totals} from 'pages/totals'
import {Metrics} from 'pages/metrics'
import {Peers} from 'pages/peers'
import {update, append} from 'util/render'
import navbar from 'templates/navbar.mft'


export class App {
    constructor() {
    }
    render() {
        return {tag: 'div', children: [
            navbar.render(this.page && this.page.constructor.name.toLowerCase()),
            this.page ? component(this.page) : "",
            ]}
    }
    static start() {
        var app = new App();
        window.onhashchange = function() {
            if(app.page) {
                app.page = null
            }
            if(window.location.hash == '#/processes') {
                app.page = Processes;
            } else if(window.location.hash == '#/status') {
                app.page = Status;
            } else if(window.location.hash == '#/values') {
                app.page = Values;
            } else if(window.location.hash == '#/totals') {
                app.page = Totals;
            } else if(window.location.hash == '#/metrics') {
                app.page = Metrics;
            } else if(window.location.hash == '#/peers') {
                app.page = Peers;
            }
            update()
        }
        append(document.body, app.render.bind(app))
        websock.start('ws://' + location.host + '/ws')
        window.onhashchange()
    }
}

App.start()

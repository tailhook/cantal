import {component} from 'util/base'
import websock from 'util/websock'
import {Context} from 'util/base'

import {Processes} from 'pages/processes'
import {Status} from 'pages/status'
import {Values} from 'pages/values'
import {Totals} from 'pages/totals'
import {Metrics} from 'pages/metrics'
import {Peers} from 'pages/peers'
import {Remote} from 'pages/remote'
import {update, append} from 'util/render'
import routing from 'util/routing'
import navbar from 'templates/navbar.mft'


export class App {
    constructor() {
    }
    render() {
        return {tag: 'div', children: [
            navbar.render(this.page && this.page.name.toLowerCase()),
            this.page ? component(this.page) : "",
            ]}
    }
    change_page(page) {
        if(this.page) {
            this.page = null
        }
        if(page == 'processes') {
            this.page = Processes;
        } else if(page == 'status') {
            this.page = Status;
        } else if(page == 'values') {
            this.page = Values;
        } else if(page == 'totals') {
            this.page = Totals;
        } else if(page == 'metrics') {
            this.page = Metrics;
        } else if(page == 'peers') {
            this.page = Peers;
        } else if(page == 'remote') {
            this.page = Remote;
        }
        update()
    }
    static start() {
        var app = new App();

        let router = routing.start()
        router.page_stream.handle(app.change_page.bind(app));
        app.change_page(router.page)

        websock.start('ws://' + location.host + '/ws')

        append(document.body, app.render.bind(app))

    }
}

App.start()

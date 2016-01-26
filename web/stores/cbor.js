import {decode} from '../util/probor'
import {applyMiddleware, createStore} from 'redux'
import middleware from 'redux-saga'
import {guard} from './util'
import {take, put, race} from 'redux-saga'

var sleep = (num) => new Promise((accept) => setTimeout(accept, num))


function* request() {
    while(true) {
        var url = yield take('url');
        let time = Date.now()
        try {
            let response = yield fetch(url.url)
            let buf = yield response.arrayBuffer()
            let data = decode(url.schema, buf)
            yield put({
                type:'data',
                payload: data,
                latency: Date.now() - time,
            })
        } catch(e) {
            console.log("Error fetching", url.url, e)
            yield put({
                type: 'error',
                error: e,
            })
        }
        let {new_url} = yield race({
            new_url: take('url'),
            delay: sleep(url.interval || 5000),
        })
        if(new_url) {
            url = new_url;
        }
    }
}

export function url(url, schema, interval=5000) {
    return {type: 'url', url: url, schema: schema, interval: interval}
}

export var createQuery = applyMiddleware(
    middleware(guard(request))
)(createStore)

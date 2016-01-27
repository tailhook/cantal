import {applyMiddleware, createStore} from 'redux'
import {guard} from '../middleware/util'
import middleware, {put, take, race, fork, runSaga, storeIO} from 'redux-saga'
import {sleep} from '../middleware/util'
import {CANCEL} from 'khufu-runtime'
import {QueryResponse} from './query'
import {decode} from '../util/probor'


var counter = 0


function global_reducer(state, action) {

    if(action.type.substr(0, 7) != 'EFFECT_') {
        console.log("HELLO --", action)
    }
}

function* manager(queries) {
    while(true) {
        // Don't need to refresh on removed query, but must never miss removing
        // of a query to avoid memory leaks
        let {add, del} = yield race({
            'add': take('add_query'),
            'del': take('del_query'),
        })
        console.log("ADD, DEL", add, del)
        if(add) {
            queries[add.id] = add.query
        }
        if(del) {
            delete queries[q.id]
        }
    }
}

function* aggregator() {
    let queries = {}
    yield fork(manager, queries)

    while(true) {
        let textq = JSON.stringify(queries)
        if(textq != '{}') {
            console.log("QUERY", textq)
            let response = yield fetch('/query.cbor', {
                method: 'POST',
                body: `{"rules": ${ textq }}`,
                headers: {'Content-Type': 'application/json'},
            })
            let buf = yield response.arrayBuffer()
            let data = decode(QueryResponse, buf)
            for(let key of Object.keys(queries)) {
                console.log('key', key, data.values.get(key))
            }
        }
        yield sleep(5000)
    }
}


function specific_query(query) {
    return function* do_query() {
        let id = 'q' + (++counter);
        try {
            global_store.dispatch({type: 'add_query', query: query, id: id})
            yield take(CANCEL)
        } finally {
            global_store.dispatch({type: 'del_query', id: id})
        }
    }
}


export var global_store = applyMiddleware(
    middleware(guard(aggregator))
)(createStore)(global_reducer)

export var query = query => applyMiddleware(
    middleware(guard(specific_query(query)))
)(createStore)


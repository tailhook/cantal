import {applyMiddleware, createStore} from 'redux'
import {refresher, probor} from '../middleware/request'
import {UPDATE_REQUEST, DATA, ERROR} from '../middleware/request'
import {CANCEL} from 'khufu-runtime'
import {QueryResponse} from './query'
import {decode} from '../util/probor'

export const METRICS = '@@local-query/metrics'

var counter = 0
var queries = {}
var stores = new Map()


let update = refresher({
    dispatch(action) {
        switch(action.type) {
            case DATA: {
                for(let [key, store] of stores.entries()) {
                    let val = action.data.values.get(key)
                    /// TODO(tailhook) capture errors? middleware?
                    store.dispatch({type: METRICS, metrics: val})
                }
                break;
            }
            case ERROR: {
                for(let store of stores.values()) {
                    /// TODO(tailhook) capture errors? middleware?
                    store.dispatch(action)
                }
            }
        }
    }
})

function add_query(id, query, store) {
    queries[id] = query
    stores.set(id, store)
    update(probor('/query.cbor', QueryResponse, 5000, {
        body: JSON.stringify({"rules": queries}),
        immediate: true,
    }))
}

function del_query(id) {
    delete queries[id]
    stores.delete(id)
    if(stores.size > 0) {
        update(probor('/query.cbor', QueryResponse, 5000, {
            body: JSON.stringify({"rules": queries}),
        }))
    } else {
        update({type: CANCEL})
    }
}

var specific_query = query => store => dispatch => {
    let id = 'q' + (++counter);
    add_query(id, query, store)
    return action => {
        if(action.type == CANCEL) {
            del_query(id)
        }
        dispatch(action)
    }
}


export var query = query => applyMiddleware(
    specific_query(query)
)(createStore)


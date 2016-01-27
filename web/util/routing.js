import {createStore, applyMiddleware} from 'redux'
import {guard} from '../middleware/util'
import {take, race, put} from 'redux-saga'
import middleware from 'redux-saga'

const DEFAULT_PAGES = {
    true: 'grid',
    false: 'status',
}

function serialize(ob) {
    let path = '';
    if(ob.remote) {
        path += '/remote';
    } else {
        path += '/local'
    }
    if(ob.page) {
        path += '/' + ob.page
    } else {
        path += '/' + DEFAULT_PAGES[!!ob.remote];
    }
    return path
}

function deserialize(path) {
    let m = path.match(/^https?:\/\/[^\/]+(\/.*)$/);
    if(m) {
        path = m[1];
    }
    let chunks = path.split('/');
    let res = {
        remote: chunks[1] == 'remote',
        page: chunks[2],
    }
    return res
}

function apply(state, update) {
    return Object.assign({}, state, update)
}

export function path(state={remote: false}, action) {
    switch(action.type) {
        case 'update':
            return apply(state, action.delta)
        case 'reset':
            return action.value
    }
    return state
}

export function go(delta_or_event, event) {
    let delta;
    if(delta_or_event instanceof Event) {
        event = delta_or_event
        delta = deserialize(event.currentTarget.href)
    } else {
        delta = delta_or_event
    }
    if(event) {
        event.preventDefault()
    }
    return {type: 'update', delta: delta}
}

export function toggle_remote(store) {
    return go({remote: !store.remote, page: DEFAULT_PAGES[!store.remote]})
}

function back(path) {
    return {type: 'reset', value: deserialize(path)}
}

function* push_history(getState) {
    while(true) {
        let {a_go, a_back} = yield race({
            a_go: take('update'),
            a_back: take('reset'),
        })
        if(a_go) {
            history.pushState({}, '', serialize(apply(getState(), a_go.delta)))
        }
    }
}

function fetch_history_middleware({dispatch, getState}) {
    dispatch(back(location.pathname))
    window.addEventListener('popstate', function(e) {
        dispatch(back(location.pathname))
    })
    return action => action;
}

export var router = applyMiddleware(
    middleware(guard(push_history)),
    fetch_history_middleware
)(createStore)(path);

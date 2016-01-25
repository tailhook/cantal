import {createStore, applyMiddleware} from 'redux'
import {guard} from '../stores/util'
import {take, race, put} from 'redux-saga'
import middleware from 'redux-saga'

export function path(state={child: '/', segment: ''}, action) {
    switch(action.type) {
        case 'go':
            let chunks = action.path.substr(1).split('/');
            state.segment = chunks[0];
            break;
    }
    return state
}

export function go(uri, event) {
    if(event) {
        event.preventDefault()
    }
    let m = uri.match(/https?:\/\/[^\/]+(\/.*)$/);
    if(m) {
        return { type: 'go', path: m[1]}
    } else {
        return { type: 'go', path: uri}
    }
}

export function back(path) {
    return { type: 'back', path: path}
}

function* push_history() {
    while(true) {
        let {a_go, a_back} = yield race({
            a_go: take('go'),
            a_back: take('back'),
        })
        if(a_go) {
            history.pushState({}, '', a_go.path)
        } else {
            yield put(go(a_back.path))
        }
    }
}

function fetch_history_middleware({dispatch, getState}) {
    dispatch(go(location.pathname))
    window.addEventListener('popstate', function(e) {
        dispatch(back(location.pathname))
    })
    return action => action;
}

export var createRoute = applyMiddleware(
    middleware(() => guard(push_history)),
    fetch_history_middleware
)(createStore)


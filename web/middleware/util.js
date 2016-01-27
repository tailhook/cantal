import {CANCEL} from 'khufu-runtime'
import {fork, take, cancel} from 'redux-saga'

export var sleep = (num) => new Promise((accept) => setTimeout(accept, num))

export function guard(fun) {
    return function* guard(getState) {
        let task = yield fork(fun, getState)
        yield take(CANCEL)
        yield cancel(task)
    }
}



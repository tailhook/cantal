import {CANCEL} from 'khufu-runtime'
import {fork, take, cancel} from 'redux-saga'

export function* guard(generator) {
    let task = yield fork(generator)
    yield take(CANCEL)
    yield cancel(task)
}



import {createStore, applyMiddleware} from 'redux'
import {attach} from 'khufu-runtime'
import {Router} from 'khufu-routing'
import {} from './graphql'

import {main} from './main.khufu'

let router = new Router(window);
let app_el = document.getElementById('app');
let khufu_instance = attach(app_el, main(router, VERSION), {
    store(reducer, middleware, state) {
        if(typeof reducer != 'function') {
            return reducer
        }
        let mid = middleware.filter(x => typeof x === 'function')
        if(DEBUG) {
            let logger = require('redux-logger')
            mid.push(logger.createLogger({
                collapsed: true,
            }))
        }
        let store = createStore(reducer, state, applyMiddleware(...mid))
        for(var m of middleware) {
            if(typeof m !== 'function') {
                if(m.type) {
                    store.dispatch(m)
                } else if(DEBUG) {
                    console.error("Wrong middleware", m)
                    throw Error("Wrong middleware: " + m)
                }
            }
        }
        return store
    }
})

let unsubscribe = router.subscribe(khufu_instance.queue_render)

if(module.hot) {
    module.hot.accept()
    module.hot.dispose(() => {
        unsubscribe()
        websock.stop()
    })
}

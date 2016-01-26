import khufu from 'khufu-runtime'
import * as websock from './websock'
import regeneratorRuntime from 'regenerator/runtime'
window.regeneratorRuntime = regeneratorRuntime

import {main} from './main.khufu'
import {router} from './util/routing'

let khufu_instance = khufu(document.getElementById('app'), main(VERSION))
router.subscribe(khufu_instance.queue_render)

if(process.env.NODE_ENV == 'production') {
    websock.start('ws://' + location.host  + '/ws',
        khufu_instance.queue_render)
} else {
    websock.start('ws://' + location.hostname  + ':22682/ws',
        khufu_instance.queue_render)
}

if(module.hot) {
    module.hot.accept()
}

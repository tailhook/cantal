import {Stream} from 'util/streams'

var router

class Router {
    constructor(hash) {
        this.page_stream = new Stream('page_change')
        this.chunk_streams = {}
        const parsed = this.parse_hash(hash)
        this.chunks = parsed.chunks
    }
    parse_hash(hash) {
        const url = hash.substr(1);
        const [path, query] = url.split('?', 1)
        let chunks = path.split('/')
        if(chunks[0] == '') {
            chunks.shift()
        }
        return {chunks}  // TODO(tailhook) parse query
    }
    get page() {
        return this.chunks[0]
    }
    hash_change(nhash) {
        const nparams = this.parse_hash(nhash)
        if(nparams.chunks[0] != this.page) {
            this.page_stream.handle_event(nparams.chunks[0]);
        }
        for(let i = 1; i < nparams.chunks.length; ++i) {
            if(nparams.chunks[i] != this.chunks[i]) {
                const stream = this.chunk_streams[i]
                if(stream) {
                    stream.handle_event(nparams.chunks[i])
                }
            }
        }
        this.chunks = nparams.chunks
    }
    set_chunk(level, value) {
        // TODO(tailhook) add query
        const chunks = this.chunks.splice(level, 0, value)
        window.location.hash = '#/' + chunks.join('/')
    }
}


export class Hier {
    constructor(level, defvalue) {
        this.defvalue = defvalue
        console.assert(router, "Router must be initialized")
        // TODO(tailhook) check for conflicts and remove the stream
        //console.assert(!router.chunk_streams[level],
        //               "Conflicting hierarchy router",
        //               router.chunk_streams[level])
        router.chunk_streams[level] = new Stream("routing_hier_" + level)
        router.chunk_streams[level].handle(this.new_value.bind(this))
        this.value = router.chunks[level] || defvalue
    }
    new_value(value) {
        this.value = value || this.defvalue
    }
    apply(value) {
        console.assert(router, "Router must be initialized")
        router.set_chunk(level, value)
    }
}

export function start() {
    router = new Router(window.location.hash)
    window.onhashchange = () => router.hash_change(window.location.hash)
    return router
}

export default exports

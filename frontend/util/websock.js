import render from 'util/render'
import {Enum, Reflect, decode} from 'util/probor'

var web_socket
var url = url
export var last_beacon
export var connected

export function start(_url) {
    url = _url
    connect()
}

function connect() {
    web_socket = new WebSocket(url)
    web_socket.binaryType = "arraybuffer";
    web_socket.onmessage = message_received;
    web_socket.onopen = onconnected
    web_socket.onclose = ondisconnected
}

class Message {
}

class Beacon extends Message {
    constructor(props) {
        super()
        for(let k of Object.keys(props)) {
            this[k] = props[k]
        }
    }
}

Beacon.probor_protocol = new Reflect()

// We only need beacon for client websockets for now
Message.probor_protocol = new Enum({0: Beacon})

function message_received(ev) {
    if(ev.data.constructor == ArrayBuffer) {
        let data = decode(Message, ev.data);
        if(data.constructor == Beacon) {
            const tm = new Date().getTime();
            const beacon = data
            beacon.receive_time = tm
            beacon.latency = tm - beacon.current_time
            last_beacon = beacon
            console.log("Beacon", beacon)
            render.update()
        }
    } else {
        console.error("Spontaneous text data", ev.data)
    }
}

function onconnected(ev) {
    connected = true
}

function ondisconnected(ev) {
    connected = false
    setTimeout(connect, 1000)
}

export function send(variant, ...args) {
    web_socket.send(JSON.stringify({"variant": variant, "fields": args}))
}

export function remote_enabled() {
    return last_beacon && last_beacon.remote_total != null
}


export default exports
window.WEBSOCK_DEBUG_INTERFACE = exports


import render from 'util/render'

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
    web_socket.onmessage = message_received;
    web_socket.onopen = connected
    web_socket.onclose = disconnected
}

function message_received(ev) {
    if(typeof(ev.data) == "string") {
        let data = JSON.parse(ev.data);
        switch(data.variant) {
            case "Beacon":
                const tm = new Date().getTime();
                const beacon = data.fields[0]
                beacon.receive_time = tm
                beacon.latency = tm - beacon.current_time
                last_beacon = beacon
                console.log("Beacon", beacon)
                render.update()
                break;
            default:
                console.log("Wrong message", data)
        }
    } else {
        console.error("Wrong websock message", ev.data);
    }
}

function connected(ev) {
    connected = true
}

function disconnected(ev) {
    connected = false
    setTimeout(connect, 1000)
}

export function send(variant, ...args) {
    web_socket.send(JSON.stringify({"variant": variant, "fields": args}))
}


export default exports
window.WEBSOCK_DEBUG_INTERFACE = exports


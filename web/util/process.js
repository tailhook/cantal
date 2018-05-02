import {format_uptime, till_now_ms, from_ms} from '../util/time'

export function decode_cmdline(cmd) {
    return cmd.replace(/\u0000/g, ' ')
}

export function uptime(process) {
    let beacon = false; // TODO
    if(beacon) {
        return format_uptime(till_now_ms(from_ms(
            process.start_time + beacon.boot_time)))
    } else {
        return '?'
    }
}

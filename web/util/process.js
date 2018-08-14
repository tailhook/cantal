import {format_uptime, till_now_ms, from_ms} from '../util/time'
import {beacon} from '../graphql'

export function decode_cmdline(cmd) {
    return cmd.replace(/\u0000/g, ' ')
}

export function uptime(process) {
    if(beacon) {
        return format_uptime(till_now_ms(from_ms(
            process.start_time + beacon.bootTime*1000)))
    } else {
        return '?'
    }
}

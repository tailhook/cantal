export function decode_cmdline(cmd) {
    return cmd.replace(/\u0000/g, ' ')
}

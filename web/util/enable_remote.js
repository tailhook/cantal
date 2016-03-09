export function remote(state=null, action) { }

export function enable_remote() {
    // We assume that SysOp users are smart enough to not to press the
    // "Enable" button too much times (to exhaust browser's memory).
    // It's fairly quick and idempotent query in cantal anyway.
    fetch("/start_remote.json", {method: 'POST'})
    return {type: 'enabling'}
}

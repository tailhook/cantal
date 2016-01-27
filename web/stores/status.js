export function memory(state={}, action) {
    switch(action) {
        case 'data':
            console.log("GOT MEMORY", action.payload)
            break;
    }
}

export function cpu(state={}, action) {
    switch(action) {
        case 'data':
            console.log("GOT MEMORY", action.payload)
            break;
    }
}

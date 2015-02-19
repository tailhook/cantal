<request-error>
    <span class="error bg-danger"
        title={ title(opts.error) }>
        [E{ opts.error.status }]
    </span>


    this.title = function title(err) {
        return `${err.status} ${err.status_text}\nAt ${err.time}`
    }

</request-error>

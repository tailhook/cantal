<request-error>
    <span class="error bg-danger"
        title={ title(opts.error) }>
        [{ opts.error.short_name || error }]
    </span>

    import {format_datetime} from "util/time"

    this.title = function title(err) {
        return `${err.description}\nAt ${
            format_datetime(err.time)}`
    }

</request-error>

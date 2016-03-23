import {already_percent_formatter} from '../util/format'

export const CPU_COLORS = [
    [100, 'rgb(186,228,179)'],
    [200,'rgb(116,196,118)'],
    [800, 'rgb(49,163,84)'],
    [1600, 'rgb(0,109,44)'],
    [6400, "black"]
]
export const CPU_YAXIS = {
    height: 40,
    bg_color: 'rgb(237,248,233)',
    skip_color: "white",
    format: already_percent_formatter(),
    colors: CPU_COLORS,
}

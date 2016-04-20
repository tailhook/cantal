class Query {
    constructor(value) {
        Object.assign(this, value)
    }
    // Generic filter
    filter(cond) {
        return new Query({
            ...this,
            series: {
                ...this.series,
                condition: this.series.condition.length
                    ? ['And', this.series.condition, cond]
                    : cond,
            },
        })
    }
    // Filters
    matching(item, regex_str) {
        return this.filter(["RegexLike", item, regex_str])
    }
    has(...items) {
        let flt = null;
        for(let item of items) {
            if(flt) {
                flt = ['Or', ['Has', item], flt];
            } else {
                flt = ['Has', item];
            }
        }
        return this.filter(flt)
    }
    non_matching(item, regex_str) {
        return this.filter(['Not', ['RegexLike', item, regex_str]])
    }

    // Extractors
    tip() {
        return new Query({
            ...this,
            extract: ['Tip'],
        })
    }
    history(num=1100) {
        return new Query({
            ...this,
            extract: ['HistoryByNum', num],
        })
    }
    diff(num=300) { // ten minutes
        return new Query({
            ...this,
            extract: ['DiffToAtMost', num],
        })
    }
    // Generic function
    func(...item) {
        return new Query({
            ...this,
            functions: this.functions.concat([item]),
        })
    }
    // Functions
    derivative() {
        return this.func('NonNegativeDerivative')
    }
    sumby(item, calc_total=true) {
        return this.func('SumBy', item, 'Ignore', calc_total)
    }
    sum() {
        return this.func('Sum', 'Ignore')
    }
}

export function fine_grained() {
    return new Query({
        series: {source: 'Fine', condition: []},
        extract: null,
        functions: [],
    })

}

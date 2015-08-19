use history::{TimeDelta};
use Condition;

#[derive(RustcDecodable, Debug, Clone, Copy)]
#[derive(PartialEq, Eq, Hash)]
pub enum Source {
    Tip,
    Fine,
}

probor_enum_encoder_decoder!(Source {
    #0 Tip(),
    #1 Fine(),
});

probor_struct!(
#[derive(RustcDecodable, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Filter {
    pub source: Source => (#0),
    pub condition: Condition => (#1),
});


#[derive(RustcDecodable, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetricKind {
    Counter,
    Level,
    State,
}

probor_enum_encoder_decoder!(MetricKind {
    #0 Counter(),
    #1 Level(),
    #2 State(),
});

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expectation {
    SingleSeries(MetricKind),
    MultiSeries(MetricKind),
    SingleTip(MetricKind),
    MultiTip(MetricKind),
    Chart,
    // TODO(tailhook) multi-chart?
}

// Keep in sync with query::dataset::Dataset
probor_enum_encoder_decoder!(Expectation {
    #100 SingleSeries(kind #1),
    #101 MultiSeries(kind #1),
    #200 SingleTip(kind #1),
    #201 MultiTip(kind #1),
    #300 Chart(),
});

json_enum_decoder!(Expectation {
    SingleSeries(kind),
    MultiSeries(kind),
    SingleTip(kind),
    MultiTip(kind),
    Chart(),
});

#[derive(RustcDecodable, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UndefFilter {
    Ignore,
    // TODO(tailhook) add more filter modes
}

probor_enum_encoder_decoder!(UndefFilter {
    #0 Ignore(),
});

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Function {
    Expect(Expectation),
    NonNegativeDerivative,
    ScaleToSeconds,
    Sum(UndefFilter),
    SumBy(Vec<String>, UndefFilter),
    StateChart(/* limit of distinct values */ usize),
}

probor_enum_encoder_decoder!(Function {
    #0 Expect(kind #1),
    #1 NonNegativeDerivative(),
    #2 ScaleToSeconds(),
    #3 Sum(undef_filter #1),
    #4 SumBy(fields #1, undef_filter #2),
    #5 StateChart(distinct_num #1),
});

json_enum_decoder!(Function {
    Expect(kind),
    NonNegativeDerivative(),
    ScaleToSeconds(),
    Sum(undef_filter),
    SumBy(fields, undef_filter),
    StateChart(distinct_num),
});

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Extract {
    Tip,
    DiffToAtMost(usize),
    HistoryByNum(usize),
    HistoryByTime(TimeDelta),
}

probor_enum_encoder_decoder!(Extract {
    #0 Tip(),
    #1 DiffToAtMost(limit #1),
    #2 HistoryByNum(limit #1),
    #3 HistoryByTime(millis #1),
});

json_enum_decoder!(Extract {
    Tip(),
    DiffToAtMost(limit),
    HistoryByNum(limit),
    HistoryByTime(millis),
});


probor_struct!(
#[derive(RustcDecodable, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rule {
    pub series: Filter => (#0),
    pub extract: Extract => (#1),
    pub functions: Vec<Function> => (#2),
});


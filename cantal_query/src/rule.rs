use history::{TimeStamp, TimeDelta};
use Condition;

#[derive(RustcEncodable, RustcDecodable, Debug, Clone, Copy)]
#[derive(PartialEq, Eq, Hash)]
pub enum Source {
    Tip,
    Fine,
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Filter(Source, Condition);

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetricKind {
    Counter,
    Level,
    State,
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expectation {
    MultiSeries(MetricKind),
    SingleSeries(MetricKind),
    MultiTip(MetricKind),
    SingleTip(MetricKind),
    Chart,
    // TODO(tailhook) multi-chart?
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UndefFilter {
    Ignore,
    // TODO(tailhook) add more filter modes
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq, Hash)]
// Is it useful enough?
pub enum ExtractTime {
    First,
    Last,
    Mean,
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Function {
    Expect(Expectation),
    NonNegativeDerivative,
    ScaleToSeconds,
    Sum(UndefFilter),
    SumBy(Vec<String>, UndefFilter),
    StateChart(/* limit of distinct values */ usize),
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Extract {
    Tip,
    DiffToAtMost(usize),
    HistoryByNum(usize),
    HistoryByTime(TimeDelta),
}


#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rule {
    pub series: Filter,
    pub extract: Extract,
    pub functions: Vec<Function>,
}

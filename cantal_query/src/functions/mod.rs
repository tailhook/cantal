mod sum;
mod derive;

use {Function, Dataset, UndefFilter};

impl Function {
    /// Execute a function, the backwards argument is because this function
    /// is going to be used in Iterator::fold()
    pub fn exec(d: Dataset, func: &Function) -> Dataset {
        use Function::*;
        match func {
            &Expect(_) => unimplemented!(),
            &NonNegativeDerivative => derive::non_negative_derivative(d),
            &Sum(UndefFilter::Ignore) => sum::sum(d),
            &SumBy(ref key, UndefFilter::Ignore, total)
            =>  sum::sum_by(&key, total, d),
            &StateChart(_num) => unimplemented!(),
        }
    }
}

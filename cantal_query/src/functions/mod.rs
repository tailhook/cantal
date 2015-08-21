mod sum;

use {Function, Dataset, UndefFilter};

impl Function {
    /// Execute a function, the backwards argument is because this function
    /// is going to be used in Iterator::fold()
    pub fn exec(d: Dataset, func: &Function) -> Dataset {
        use Function::*;
        match func {
            &Expect(_) => unimplemented!(),
            &NonNegativeDerivative => unimplemented!(),
            &ScaleToSeconds => unimplemented!(),
            &Sum(UndefFilter::Ignore) => sum::sum(d),
            &SumBy(_, UndefFilter::Ignore) => unimplemented!(),
            &StateChart(_num) => unimplemented!(),
        }
    }
}

use std::collections::VecDeque;
use std::cmp::min;


/// Returns tuple of
/// ("number of new datapoints", "number of valid data points")
pub fn compare_timestamps(new: &Vec<u64>, old: &VecDeque<(u64, u32)>)
    -> (u64, usize)
{
    let mut iter_new = new.iter().enumerate().peekable();
    let last_ots = old[0].0;
    let mut new_pt;
    loop { // New points
        match iter_new.peek() {
            None => return (new.len() as u64, new.len()),
            Some(&(_, &nts)) if nts > last_ots => {
                iter_new.next().unwrap();
                continue;
            }
            Some(&(nidx, _)) => {
                new_pt = nidx;
                break;
            }
        }
    }
    for ((nidx, &nts), &(ots, _)) in iter_new.zip(old.iter()) {
        if nts != ots {
            return (new_pt as u64, nidx);
        }
    }
    return (new_pt as u64, min(new.len(), new_pt + old.len()));
}

#[cfg(test)]
mod test {
    use super::compare_timestamps;

    #[test]
    fn all_new() {
        assert_eq!(compare_timestamps(
            &vec![130, 120, 110],
            &vec![(30, 0), (20, 0), (10, 0)].into_iter().collect()),
            (3, 3));
    }

    #[test]
    fn touch() {
        assert_eq!(compare_timestamps(
            &vec![50, 40, 30],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (2, 3));
    }
    #[test]
    fn overlap() {
        assert_eq!(compare_timestamps(
            &vec![40, 30, 20],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (1, 3));
    }

    #[test]
    fn old() {
        assert_eq!(compare_timestamps(
            &vec![30, 20, 10],
            &vec![(130, 0), (120, 0), (110, 0)].into_iter().collect()),
            (0, 0));
    }

    #[test]
    fn middle() {
        assert_eq!(compare_timestamps(
            &vec![40, 30, 25],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (1, 2));
    }

    #[test]
    fn middle2() {
        assert_eq!(compare_timestamps(
            &vec![40, 35, 25],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (2, 2));
    }

    #[test]
    fn middle3() {
        assert_eq!(compare_timestamps(
            &vec![50, 40, 35, 20],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (3, 3));
    }

    #[test]
    fn new_big() {
        assert_eq!(compare_timestamps(
            &vec![50, 40, 30, 20, 10],
            &vec![(30, 0), (20, 0)].into_iter().collect()),
            (2, 4));
    }
}

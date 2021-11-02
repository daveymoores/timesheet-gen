use crate::timesheet::GitLogDates;
use std::collections::{HashMap, HashSet};

pub fn get_timesheet_hashmap() -> GitLogDates {
    let date_hashmap: GitLogDates = vec![
        (2020, vec![(8, vec![1])]),
        (2019, vec![(1, vec![3])]),
        (2021, vec![(10, vec![23, 20, 21]), (9, vec![8])]),
    ]
    .into_iter()
    .map(|x| {
        let y: HashMap<u32, HashSet<u32>> =
            x.1.into_iter()
                .map(|k| {
                    let n: HashSet<u32> = k.1.into_iter().collect();
                    (k.0, n)
                })
                .collect();
        (x.0, y)
    })
    .collect();

    date_hashmap
}

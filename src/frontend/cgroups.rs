use std::collections::BTreeMap;

use frontend::graphql::ContextRef;
use scan::processes::MinimalProcess;

pub struct CGroup<'a> {
    name: &'a String,
    processes: Vec<&'a MinimalProcess>,
}

graphql_object!(<'a> CGroup<'a>: ContextRef<'a> as "CGroup" |&self| {
    field name(&executor) -> &String { self.name }
    field processes(&executor) -> &[&MinimalProcess] { &self.processes }
});

pub fn cgroups<'x>(ctx: &ContextRef<'x>) -> Vec<CGroup<'x>> {
    let mut buf = BTreeMap::new();
    for pro in &ctx.stats.processes {
        if let Some(ref gname) = pro.cgroup {
            buf.entry(&*gname)
                .or_insert_with(Vec::new)
                .push(pro);
        }
    }
    return buf.into_iter()
        .map(|(name, processes)| CGroup { name, processes })
        .collect();
}

use std::collections::BTreeMap;

use frontend::graphql::ContextRef;
use scan::processes::MinimalProcess;

pub struct CGroup<'a> {
    name: &'a String,
    processes: Vec<&'a MinimalProcess>,
}

#[derive(GraphQLInputObject)]
#[graphql(name="CGroupFilter", description="Filter for cgroups")]
pub struct Filter {
    name_prefix: Option<String>,
}

graphql_object!(<'a> CGroup<'a>: ContextRef<'a> as "CGroup" |&self| {
    field name(&executor) -> &String { self.name }
    field processes(&executor) -> &[&MinimalProcess] { &self.processes }
});

pub fn cgroups<'x>(ctx: &ContextRef<'x>, filter: Option<Filter>)
    -> Vec<CGroup<'x>>
{
    let mut buf = BTreeMap::new();
    let name_prefix = filter.as_ref().and_then(|x| x.name_prefix.as_ref());
    for pro in &ctx.stats.processes {
        if let Some(ref gname) = pro.cgroup {
            if !name_prefix.map(|x| gname.starts_with(x)).unwrap_or(true) {
                continue;
            }
            buf.entry(&*gname)
                .or_insert_with(Vec::new)
                .push(pro);
        }
    }
    return buf.into_iter()
        .map(|(name, processes)| CGroup { name, processes })
        .collect();
}

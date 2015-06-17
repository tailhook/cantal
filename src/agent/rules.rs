pub enum Source {
    Tip,
    Fine,
    Coarse,
}

pub enum Aggregation {
    None,
    Sum,
}

pub enum Load {
    Raw,
    Rate,
}

pub enum Condition {
    Eq(String, String),
    NotEq(String, String),
    RegexLike(String, String),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}


pub struct Rule {
    pub source: Source,
    pub condition: Condition,
    pub key: Vec<String>,
    pub aggregation: Aggregation,
    pub load: Load,
}

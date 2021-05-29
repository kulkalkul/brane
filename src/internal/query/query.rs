use crate::internal::parser::{Parser, JSONParser};
use crate::internal::parser::query_parser::{QueryParser, LogicalOperation};

pub enum Query {
    All,
    Id(String),
    By(LogicalOperation),
}

impl Query {
    pub fn new(json: String) -> Query {
        if let "{}" = json.as_str() {
            Query::All
        } else {
            let parser = JSONParser::new(json);
            let parser = QueryParser::new(parser.parse());
            Query::By(parser.parse())
        }
    }
}
use super::cache::Cache;

pub struct Search<'a> {
    pub cache: &'a mut Cache,
}

impl<'a> Search<'a> {
    pub fn new(cache: &mut Cache) -> Search {
        Search {
            cache
        }
    }
}
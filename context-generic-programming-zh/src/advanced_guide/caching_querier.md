# Caching Querier

```rust
trait PersonQuerier<Context>
where
    Context: PersonContext + HasError,
{
    fn query_person(context: &Context, person_id: &Context::PersonId)
        -> Result<Context::Person, Context::Error>;
}

trait PersonCacheContext: PersonContext {
    fn person_cache(&self) -> &HashMap<Self::PersonId, Self::Person>;
}

struct CachingPersonQuerier<InQuerier>(InQuerier);

impl<Context, InQuerier> PersonQuerier<Context>
    for CachingPersonQuerier<InQuerier>
where
    InQuerier: PersonQuerier<Context>,
    Context: PersonCacheContext,
    Context: HasError,
    Context::PersonId: Hash + Eq,
    Context::Person: Clone,
{
    fn query_person(context: &Context, person_id: &Context::PersonId)
        -> Result<Context::Person, Context::Error>
    {
        let entry = context.person_cache().get(person_id);

        match entry {
            Some(person) => Ok(person.clone()),
            None => InQuerier::query_person(context, person_id),
        }
    }
}
```

## Caching App Context

```rust
#[derive(Clone)]
struct BasicPerson {
    name: String,
}

struct AppContext {
    kv_store: FsKvStore,
    person_cache: HashMap<String, BasicPerson>,
    // ...
}

impl PersonCacheContext for AppContext {
    fn person_cache(&self) -> &HashMap<String, BasicPerson> {
        &self.person_cache
    }
}

impl HasPersonQuerier for AppContext {
    type PersonQuerier = CachingPersonQuerier<KvStorePersonQuerier>;
}

fn app_greeter() -> impl Greeter<AppContext> {
    SimpleGreeter
}
```
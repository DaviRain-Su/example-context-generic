# Store Implementation

```rust
struct FsKvStore { /* ... */ }
struct KvStoreError { /* ... */ }

struct ParseError { /* ... */ }

impl HasError for FsKvStore {
    type Error = KvStoreError;
}

impl KvStore for FsKvStore {
    fn get(&self, key: &str) -> Result<Vec<u8>, Self::Error> {
        unimplemented!() // stub
    }
}

impl TryFrom<Vec<u8>> for BasicPerson {
    type Error = ParseError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        unimplemented!() // stub
    }
}

enum AppError {
    KvStore(KvStoreError),
    Parse(ParseError),
    // ...
}

impl From<KvStoreError> for AppError {
    fn from(err: KvStoreError) -> Self {
        Self::KvStore(err)
    }
}

impl From<ParseError> for AppError {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

struct AppContext {
    kv_store: FsKvStore,
    // ...
}

impl HasError for AppContext {
    type Error = AppError;
}

impl PersonContext for AppContext {
    type PersonId = String;
    type Person = BasicPerson;
}

impl KvStoreContext for AppContext {
    type Store = FsKvStore;

    fn store(&self) -> &Self::Store {
        &self.kv_store
    }
}

impl HasPersonQuerier for AppContext {
    type PersonQuerier = KvStorePersonQuerier;
}

fn app_greeter() -> impl Greeter<AppContext> {
    SimpleGreeter
}
```

## Multiple Context Implementations

```rust
struct Foo;
struct Bar;

struct FooContext {
    kv_store: FsKvStore,
    foo: Foo,
    // ...
}

struct BarContext {
    kv_store: FsKvStore,
    bar: Bar,
    // ...
}

impl HasError for FooContext {
    type Error = AppError;
}

impl HasError for BarContext {
    type Error = AppError;
}

impl PersonContext for FooContext {
    type PersonId = String;
    type Person = BasicPerson;
}

impl PersonContext for BarContext {
    type PersonId = String;
    type Person = BasicPerson;
}

impl KvStoreContext for FooContext {
    type Store = FsKvStore;

    fn store(&self) -> &Self::Store {
        &self.kv_store
    }
}

impl KvStoreContext for BarContext {
    type Store = FsKvStore;

    fn store(&self) -> &Self::Store {
        &self.kv_store
    }
}

impl HasPersonQuerier for FooContext {
    type PersonQuerier = KvStorePersonQuerier;
}

impl HasPersonQuerier for BarContext {
    type PersonQuerier = KvStorePersonQuerier;
}

fn foo_greeter() -> impl Greeter<FooContext> {
    SimpleGreeter
}

fn bar_greeter() -> impl Greeter<BarContext> {
    SimpleGreeter
}
```
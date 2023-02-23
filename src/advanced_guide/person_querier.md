# Person Querier

We now look at the problem of having multiple context implementations, as well as how to deduplicate them. For this we will focus on just the implementation for `PersonQuerier` that is being used by `SimpleGreeter`.

我们现在来看一下具有多个上下文实现的问题，以及如何去除重复代码。为此，我们将仅关注被 `SimpleGreeter` 使用的 `PersonQuerier` 的实现。

The requirement for querying a person details can be implemented in many ways, such as using a key-value store (KV store) or an SQL database. Now suppose we have the following API for a KV store:

查询人员详细信息的需求可以通过多种方式实现，例如使用键值存储（KV存储）或SQL数据库。现在假设我们有以下 KV 存储的 API：

```rust
struct FsKvStore { /* ... */ }
struct KvStoreError { /* ... */ }

impl FsKvStore {
    fn get(&self, key: &str) -> Result<Vec<u8>, KvStoreError> {
        unimplemented!() // stub
    }
    // ...
}
```

We could implement PersonQuerier for any context type that contains FsKvStore in its field:

我们可以为任何包含`FsKvStore`字段的上下文类型实现`PersonQuerier`：

```rust
struct BasicPerson {
    name: String,
}

struct ParseError { /* ... */ }

impl TryFrom<Vec<u8>> for BasicPerson {
    type Error = ParseError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        unimplemented!() // stub
    }
}

struct AppContext {
    kv_store: FsKvStore,
    // ...
}

enum AppError {
    KvStore(KvStoreError),
    Parse(ParseError),
    // ...
}

impl HasError for AppContext {
    type Error = AppError;
}

impl PersonContext for AppContext {
    type PersonId = String;
    type Person = BasicPerson;
}

impl PersonQuerier for AppContext {
    fn query_person(&self, person_id: &Self::PersonId)
        -> Result<Self::Person, Self::Error>
    {
        let key = format!("persons/{}", person_id);
        let bytes = self.kv_store.get(&key)
            .map_err(AppError::KvStore)?;

        let person = bytes.try_into()
            .map_err(AppError::Parse)?;

        Ok(person)
    }
}
```

Even this simplified version of the implementation for `query_person` involves quite a bit of logic. First, we need to implement serialization logic to parse `BasicPerson` from raw bytes. We also need to implement the logic of mapping the namespaced key from the person ID, as well as mapping the errors in each operation into `AppError`s.

即使是这个简化版本的 `query_person` 实现，也涉及到相当多的逻辑。首先，我们需要实现序列化逻辑以从原始字节解析出 `BasicPerson`。我们还需要实现将 `person ID` 映射为带命名空间的 key 的逻辑，以及将每个操作中的错误映射为 AppError。

Fortunately, with the context traits design pattern, components like `SimpleGreeter` do not need to be aware of how `PersonQuerier` is implemented, or the existence of the key-value store in the context. However, it would still be problematic if we need to re-implement `PersonQuerier` for every new context type that we implement.

幸运的是，使用上下文特征设计模式时，如 `SimpleGreeter` 组件不需要知道 `PersonQuerier` 如何实现，或上下文中键值存储的存在。然而，如果我们需要为每个新的上下文类型重新实现 `PersonQuerier`，这仍将是有问题的。

To avoid copying the body of `query_person` for all context types, we want to have a generic implementation of `PersonQuerier` for any context that has `FsKvStore` in one of its fields. But if we recall from earlier sections, we already came up with the design pattern for implementing context-generic components like `Greeter`. So why not just turn `PersonQuerier` itself into a context-generic component?

为了避免为每个新的上下文类型重新实现`PersonQuerier`，我们希望对于任何具有`FsKvStore`字段之一的上下文都有一个通用的`PersonQuerier`实现。但是，如果我们回顾之前的章节，我们已经提出了实现像`Greeter`这样的上下文通用组件的设计模式。那么，为什么不将`PersonQuerier`本身变成一个上下文通用组件呢？

In fact, with a little re-arrangement, we can redefine `PersonQuerier` as `PersonQuerier` as follows:

实际上，稍作调整，我们可以将`PersonQuerier`重新定义为如下形式:

```rust
trait PersonQuerier<Context>
where
    Context: PersonContext + HasError,
{
     fn query_person(context: &Context, person_id: &Context::PersonId)
         -> Result<Context::Person, Context::Error>;
}
```

We now have a `PersonQuerier` component that is parameterized by a generic `Context` type, and it looks very similar to how we define `Greeter`. With this, we can now define a context-generic implementation of `PersonQuerier` for any context that has an `FsKvStore`:

现在，我们有一个以泛型 `Context` 类型为参数的 `PersonQuerier` 组件，其定义与我们定义 `Greeter` 的方式非常相似。有了这个，我们现在可以为任何具有 `FsKvStore` 的上下文定义一个通用的 `PersonQuerier` 实现：

```rust
trait KvStoreContext {
    fn kv_store(&self) -> &FsKvStore;
}

struct KvStorePersonQuerier;

impl<Context, PersonId, Person, Error, ParseError>
    PersonQuerier<Context> for KvStorePersonQuerier
where
    Context: KvStoreContext,
    Context: PersonContext<Person=Person, PersonId=PersonId>,
    Context: HasError<Error=Error>,
    PersonId: Display,
    Person: TryFrom<Vec<u8>, Error=ParseError>,
    Error: From<KvStoreError>,
    Error: From<ParseError>,
{
    fn query_person(context: &Context, person_id: &PersonId)
        -> Result<Person, Error>
    {
        let key = format!("persons/{}", person_id);

        let bytes = context.kv_store().get(&key)?;

        let person = bytes.try_into()?;

        Ok(person)
    }
}
```

We first define a `KvStoreContext` trait, which allows us to extract a reference to `FsKvStore` out of a context that implements it. Following that, we define `KvStorePersonQuerier` as an empty struct, similar to how we defined `SimpleGreeter`.

我们首先定义一个`KvStoreContext trait`，该trait允许我们从实现它的上下文中提取对`FsKvStore`的引用。然后，我们定义了`KvStorePersonQuerier`作为空结构体，类似于我们如何定义`SimpleGreeter`。


We then implement `PersonQuerier` for `KvStorePersonQuerier` to work with any `Context` type, given that several additional constraints are satisfied. We also use explicit type parameter bindings to simplify the specification of our constraints.

然后，我们为 `KvStorePersonQuerier` 实现 `PersonQuerier`，以便与任何 `Context` 类型一起使用，只要满足几个附加的约束条件。我们还使用显式类型参数绑定来简化约束条件的说明。

We require `Context` to implement `KvStoreContext`, so that we can extract FsKvStore from it. We also require `Context::PersonId` to implement `Display` so that we can format the key as a string. Similarly, we require that `Context::Person` implements `TryFrom<Vec<u8>>` and bind the conversion error to an additional type binding ParseError.

我们要求 `Context` 实现 `KvStoreContext`，以便从中提取出 `FsKvStore`。我们还要求 `Context::PersonId` 实现 `Display`，以便将键格式化为字符串。同样，我们要求 `Context::Person` 实现 `TryFrom<Vec<u8>>`，并将转换错误绑定到额外的类型参数 `ParseError`。

The above bindings essentially make it possible for `KvStorePersonQuerier` to work with not only any context that provides `FsKvStore`, but also any `Context::PersonId` and `Context::Person` types as long as they implement the `Display` and `TryFrom` traits.

上述的绑定本质上使得 `KvStorePersonQuerier` 能够与任何提供 `FsKvStore` 的上下文配合使用，同时还能与任何实现 `Display` 和 `TryFrom` 特性的 `Context::PersonId` 和 `Context::Person` 类型配合使用。

We additionally require `Context::Error` to allow injection of sub-errors from `KvStoreError` and `ParseError`, so that we can propagate the errors inside `query_person`. If an error arises either when fetching bytes from the store via the `context.kv_store().get()` call, or when parsing those bytes via the `bytes.try_into()` call, the `?` operator will implicitly call `into()` appropriately in order to coerce the error into an `Error`.

我们还要求 `Context::Error` 允许从 `KvStoreError` 和 `ParseError` 注入子错误，以便我们可以在 `query_person` 内部传播错误。如果在通过 `context.kv_store().get()` 调用从存储中获取字节或通过 `bytes.try_into()` 调用解析这些字节时出现错误，则 ? 运算符将隐式调用 `into()` 以将错误强制转换为 `Error`。

## Generic Store

We managed to get `KvStorePersonQuerier` we defined earlier to not only work with a generic context containing an `FsKvStore`, but also work with any `PersonId` and `Person` types that satisfy certain constraints.

我们已经成功使之前定义的 `KvStorePersonQuerier` 能够适用于一个包含 `FsKvStore` 的通用 `context`，同时也适用于任何满足特定约束条件的 `PersonId` 和 `Person` 类型。

We can further generalize the implementation of `KvStorePersonQuerier` to work with any key-value store implementation. With that, we can easily swap our store implementation from file-based to memory-based.

我们可以进一步将 `KvStorePersonQuerier` 的实现泛化，以适用于任何键值存储实现。这样，我们可以轻松地将存储实现从基于文件的转换为基于内存的。

```rust
trait KvStore: HasError {
    fn get(&self, key: &str) -> Result<Vec<u8>, Self::Error>;
}

trait KvStoreContext {
    type Store: KvStore;

    fn store(&self) -> &Self::Store;
}

struct KvStorePersonQuerier;

impl<Context, Store, PersonId, Person, Error, ParseError, StoreError>
    PersonQuerier<Context> for KvStorePersonQuerier
where
    Context: KvStoreContext<Store=Store>,
    Context: PersonContext<Person=Person, PersonId=PersonId>,
    Context: HasError<Error=Error>,
    Store: KvStore<Error=StoreError>,
    PersonId: Display,
    Person: TryFrom<Vec<u8>, Error=ParseError>,
    Error: From<StoreError>,
    Error: From<ParseError>,
{
    fn query_person(context: &Context, person_id: &PersonId)
        -> Result<Person, Error>
    {
        let key = format!("persons/{}", person_id);

        let bytes = context.store().get(&key)?;

        let person = bytes.try_into()?;

        Ok(person)
    }
}
```

We first define a `KvStore` trait that provides a `get` method for reading values from the store. It also has `HasError` as its supertrait, so that we can reuse the `Error` associated type.

We then redefine the `KvStoreContext` to contain an associated type Store, which is required to implement the `KvStore` trait. We then make the `store` method return a reference to `Self::Store`.

Inside the PersonQuerier implementation for `KvStorePersonQuerier`, we introduce two new explicit type bindings: `Store` for `Context::Store`, and `StoreError` for `Store::Error`. We also require the main Error type to implement `From<StoreError>` so that any error from the store can be propagated.

首先，我们定义了一个 `KvStore trait`，它提供了一个 `get` 方法用于从 `store` 中读取值。它还有一个超级 `trait HasError`，所以我们可以重用 Error 关联类型。

接下来，我们重新定义了 `KvStoreContex`t，它包含一个关联类型 `Store`，该类型必须实现 `KvStore trait`。然后我们让 store 方法返回对 `Self::Store` 的引用。

在 `KvStorePersonQuerier` 的 `PersonQuerier` 实现中，我们引入了两个新的显式类型绑定：`Store` 表示 `Context::Store`，StoreError 表示 `Store::Error`。我们还要求主要的 `Error` 类型实现 `From<StoreError>`，以便 store 中的任何错误都可以被传播。
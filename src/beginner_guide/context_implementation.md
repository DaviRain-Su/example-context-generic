# 上下文实现（Context Implementation）

With the basic traits implemented, we now look at how we can define a concrete context that satisfies the traits:

> 通过实现基本trait，我们现在来看一下如何定义一个满足这些trait的具体上下文：

```rust
struct BasicPerson {
    name: String,
}

impl NamedPerson for BasicPerson {
    fn name(&self) -> &str {
        &self.name
    }
}

struct AppContext {
    database: Database,
}

// Database stubs
struct Database;
struct DbError;

enum AppError {
    Database(DbError),
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
        unimplemented!() // database stub
    }
}
```

We first define a `BasicPerson` struct with only a name field, since that is the minimal information required for greet to work. We implement `NamedPerson` for `BasicPerson`, by simply returning `&self.name`.

> 首先，我们定义了一个只有名字字段的`BasicPerson`结构体，因为这是`greet`函数所需的最少信息。我们为`BasicPerson`实现了`NamedPerson`trait，只需返回`&self.name`即可。

We also define an `AppContext` struct with a stub database field. For demonstration purposes, we have a dummy `Database` struct, and a `DbError` type to represent database errors. We also define an `AppError` enum to represent all application errors, with one of them being `DbError`.

> 我们还定义了一个带有存根(stub)数据库字段的`AppContext`结构体。为了演示目的，我们有一个虚拟的`Database`结构体和一个`DbError`类型来表示数据库错误。我们还定义了一个`AppError`枚举来表示所有应用程序错误，其中一个错误是`DbError`。

We implement `HasError` for `AppContext`, with `AppError` as the `Error` type. We also implement `PersonContext` for `AppContext`, with the `PersonId` associated type being String and the `Person` associated type being `BasicPerson`. We also implement `PersonQuerier` but leave the `query_person` as a stub for performing database queries in an actual application.

> 我们为`AppContext`实现了`HasError`，以`AppError`作为错误类型。我们还为`AppContext`实现了`PersonContext`，其中`PersonId`相关类型为`String`，`Person`相关类型为`BasicPerson`。我们还实现了`PersonQuerier`，但是将`query_person`留作存根(stub)，以便在实际应用程序中执行数据库查询。

# 泛型上下文（Generic Context）

There are ways that we can make our `greet` function in Rust works more flexibly similar to its dynamic-typed counterparty. First, we define a concrete `Context` type as follows:

> 有一些方法可以使我们的Rust中的`greet`函数更加灵活，类似于其动态类型的对应项。首先，我们定义一个具体的上下文（`Context`）类型，如下所示：

```rust
struct Context {
    database: Database,
    // ...
}


fn greet(context: &Context, person_id: &PersonId) -> Result<(), DbError> {
    let person = context.database.query_person(person_id)?;
    println!("Hello, {}", person.name);
    Ok(())
}
```

At this stage, we have a concrete `Context` struct that contains the database handle as well as other environment parameters that we may need. However, `Context` is still concrete, so it is difficult to reuse the `greet` function in a different context. So let's make the `Context` generic instead:

> 在这个阶段，我们有一个具体的`Context`结构体，它包含了数据库句柄以及我们可能需要的其他环境参数。然而，`Context`仍然是具体的，因此在不同的上下文中重用`greet`函数比较困难。所以我们让`Context`成为泛型类型：

```rust
trait ContextWithDatabase {
    fn database(&self) -> &Database;
}

fn greet<Context>(context: &Context, person_id: &PersonId) -> Result<(), DbError>
where
    Context: ContextWithDatabase,
{
    let person = context.database().query_person(person_id)?;
    println!("Hello, {}", person.name);
    Ok(())
}
```

In our first attempt, we turn `Context` into a generic type parameter. With that, the concrete details of the context are lost, and we no longer know how to access the fields such as the database. But we can recover that by defining a `ContextWithDatabase` trait, which provides read-only access to extract a reference to `Database` from the context.

> 在我们的第一个尝试中，我们将`Context`转换为一个泛型类型参数。通过这样做，上下文的具体细节丢失了，我们不再知道如何访问数据库等字段。但是我们可以通过定义一个`ContextWithDatabase trait` 来恢复它，该trait提供只读访问权限，以从上下文中提取对`Database`的引用。

With that, we are able to make `greet` work with any context type as long as it contains a field for `Database`. For example:

> 这样一来，只要上下文类型包含一个名为`Database`的字段，我们就可以让`greet`函数与任何上下文类型一起使用。例如：

```rust
struct AppContext {
    database: Database
}

impl ContextWithDatabase for AppContext {
    fn database(&self) -> &Database {
        &self.database
    }
}
```

However, since the `Database` type is concrete, it is challenging if we want to run `greet` with an environment without a database, such as an in-memory key-value store, cache store, or a blockchain. What we can do instead is to define methods such that we can query for a person's details directly from the context:

> 然而，由于`Database`类型是具体的，如果我们想在没有数据库的环境中运行`greet`，比如内存键值存储、缓存存储或区块链等，那么这就很具有挑战性。我们可以改变策略，定义一些方法，使我们可以直接从上下文中查询人的详细信息：

```rust
struct Error { /* ... */}

trait PersonQuerier {
    fn query_person(&self, person_id: &PersonId) -> Result<Person, Error>;
}

fn greet<Context>(context: &Context, person_id: &PersonId)
    -> Result<(), Error>
where
    Context: PersonQuerier,
{
    let person = context.query_person(person_id)?;
    println!("Hello, {}", person.name);
    Ok(())
}
```

We define a `PersonQuerier` trait that exposes a method for querying for a person's details directly from the context. With that, we can have our `greet` function work with any context type that knows how to query for person details, regardless of whether it is implemented as a database query.

> 我们定义了一个`PersonQuerier triat`，公开了一种直接从上下文中查询人的详细信息的方法。通过这样做，我们可以使我们的`greet`函数与任何上下文类型一起使用，只要它知道如何查询人的详细信息，无论它是作为数据库查询实现的还是其他实现。

## 带错误的上下文 （Context with Error）

One thing to note however is that the `Error` type in `PersonQuerier` is concrete. With that, it would be problematic if we want to define new contexts that have different query methods but also return different errors. While it is possible to define a dynamic error type such as `Box<dyn Error>`, such type erasure would mean that we lose information about what kinds of errors can happen when we try to query for `Person` details.

> 有一件需要注意的事情是，`PersonQuerier`中的`Error`类型是具体的。这样一来，如果我们想定义具有不同查询方法并返回不同错误的新上下文，那么就会出现问题。虽然可以定义动态错误类型，例如`Box<dyn Error>`，但这种类型擦除意味着我们失去了有关在尝试查询`Person`详细信息时可能发生哪些错误的信息。

We can instead make the error type generic. But instead of using it as a generic parameter for `greet`, we can define it as an associated type for the generic type `Context`:

> 我们可以将错误类型改为泛型类型。但是，我们可以将其定义为泛型类型`Context`的相关类型，而不是将其用作`greet`的泛型参数：

```rust
trait HasError {
    type Error;
}

trait PersonQuerier: HasError {
    fn query_person(&self, person_id: &PersonId) -> Result<Person, Self::Error>;
}

fn greet<Context>(context: &Context, person_id: &PersonId) -> Result<(), Context::Error>
where
    Context: PersonQuerier,
{
    let person = context.query_person(person_id)?;
    println!("Hello, {}", person.name);
    Ok(())
}
```

We define a new `HasError` trait with only one thing, which is the `Error` associated type. Aside from that, there is nothing known about the Error type, but that is ok as we will see later on. The trait `PersonQuerier` then has `HasError` as its supertrait, esentially allowing it to access the associated type as `Self::Error` in the return type of `query_person`.

> 我们定义一个新的`HasError`trait，其中只有一个`Error`相关类型。除此之外，关于错误类型没有任何其他信息，但这没有关系，因为后面我们会看到。`PersonQuerier` trait随后将`HasError`作为其 `supertrait`，从而允许在`query_person`的返回类型中通过`Self::Error`访问相关类型。

We define the `Error` associated type in a separate `HasError` trait, instead of directly in the `PersonQuerier` trait. As we will see later, this is essential to allow multiple context traits to access the same `Error` type.

> 我们将`Error`相关类型定义在一个单独的`HasError`trait 中，而不是直接在`PersonQuerier` trait 中定义。正如我们将要看到的那样，这对于允许多个上下文trait 访问相同的`Error`类型是至关重要的。

In the `greet` function, we require the generic `Context` type to implement `PersonQuerier`. But since `HasError` is a supertrait of `PersonQuerier`, we would also able to access the error type as `Context::Error`.

> 在`greet`函数中，我们要求泛型`Context`类型实现`PersonQuerier`。但由于`HasError`是`PersonQuerier`的supertrait，我们也可以通过`Context::Error`访问错误类型。

## 显式关联类型绑定 （Explicit Associated Type Binding）

As we can see, by having generic type parameters as associated types in the traits that `Context` implements, we are able to keep just one generic type parameter in the `greet` function.

> 正如我们所看到的，通过将泛型类型参数定义为上下文实现的trait中的相关类型，我们能够在`greet`函数中只保留一个泛型类型参数。

However, it is still possible for us to explicitly pull out `Error` as a generic type parameter and bind to the `Error` associated type as follows:

> 然而，我们仍然可以显式地将`Error`作为泛型类型参数提取出来，并将其绑定到`Error`相关类型，如下所示：

```rust
fn greet<Context, Error>(context: &Context, person_id: &PersonId)
    -> Result<(), Error>
where
    Context: PersonQuerier<Error=Error>,
{
    let person = context.query_person(person_id)?;
    println!("Hello, {}", person.name);
    Ok(())
}
```

By specifying the trait bound `Context: PersonQuerier<Error=Error>`, we state that the `greet` function works with any generic type `Error`, provided that `Context::Error` is the same as `Error`. With the explicit binding, we are able to have greet return `Result<(), Error>` instead of `Result<(), Context::Error>`.

> 通过指定trait约束`Context: PersonQuerier<Error=Error>`，我们声明`greet`函数可以与任何泛型类型`Error`一起使用，只要`Context::Error`与`Error`相同。使用显式绑定，我们可以让`greet`返回`Result<(), Error>`而不是`Result<(), Context::Error>`。

There are sometimes benefits when we bind the associated types to an explicit generic type parameter. For one, the inferred type shown in IDEs like Rust Analyzer would be simpler, as they are shown as `Error` instead of the fully qualified syntax `<Context as HasError>::Error`. As we will see later, explicit type parameters also help us by providing a way to specify the additional trait bounds of the associated types.

> 将相关类型绑定到显式的泛型类型参数中有时会带来好处。首先，IDE中显示的推断类型（例如 Rust Analyzer）会更简单，因为它们显示为`Error`，而不是完全限定的语法`<Context as HasError>::Error`。正如我们将在后面看到的那样，显式类型参数还可以通过提供指定相关类型的其他trait约束的方式来帮助我们。

Aside from that, it is up to the programmer to decide whether to bind the associated types to explicit type parameters. The key thing to understand here is that the explicit bindings are optional, and we can choose to omit such parameters whenever it is appropriate.

> 除此之外，是否将相关类型绑定到显式类型参数取决于程序员的决策。在这里需要理解的关键是，显式绑定是可选的，我们可以在适当的时候选择省略此类参数。

## 泛型 Persion （Generic Person）

Right now, our `Error` type has become generic, but our `Person` type is still concrete. We may also want to make the `Person` type generic in a similar way, so that the `greet` function can work with any other person types.

> 现在，我们的`Error`类型已经变成了泛型，但是`Person`类型仍然是具体的。我们可能还想以类似的方式使`Person`类型成为泛型，这样`greet`函数就可以与任何其他人物类型一起使用。

This may be essential for reasons such as performance. For instance, depending on where `greet` is called, it may be desirable to load all details about a person from the database so that it can be cached, or conversely, it might be desirable to load minimal details in order to save bandwidth.

> 这可能是出于性能等原因而必需的。例如，根据`greet`的调用位置，可能希望从数据库中加载有关一个人的所有详细信息，以便可以进行缓存，或者反过来，可能希望加载最少的详细信息以节省带宽。

From the perspective of `greet`, it does not matter what fields a `Person` type has, as long as it can extract the name of the person as a string. So we can generalize the `Person` type as follows:

> 从`greet`的角度来看，`Person`类型有哪些字段并不重要，只要它可以将人的名称提取为字符串即可。因此，我们可以将`Person`类型概括如下：

```rust
trait NamedPerson {
    fn name(&self) -> &str;
}

trait HasError {
    type Error;
}

trait PersonContext {
    type PersonId;
    type Person: NamedPerson;
}

trait PersonQuerier: PersonContext + HasError {
    fn query_person(
        &self,
        person_id: &Self::PersonId,
    ) -> Result<Self::Person, Self::Error>;
}

fn greet<Context>(
    context: &Context,
    person_id: &Context::PersonId,
) -> Result<(), Context::Error>
where
    Context: PersonQuerier,
{
    let person = context.query_person(person_id)?;
    println!("Hello, {}", person.name());
    Ok(())
}
```

We first define a `NamedPerson` trait, with a `name` method to extract a string out from the person. We also define a `PersonContext` trait that has two associated types: `PersonId` and `Person`. The `PersonId` type is completely generic, as we don't care whether it is a string, an integer, or a `UUID`. The associated type `Person` is also generic, but we also add a trait bound that the type must implement `NamedPerson`.

> 我们首先定义了一个`NamedPerson`trait ，其中包含一个`name`方法，用于从`Person`中提取一个字符串。我们还定义了一个`PersonContext`trait ，其中有两个相关类型：`PersonId`和`Person`。`PersonId`类型是完全泛型的，因为我们不关心它是字符串、整数还是`UUID`。关联类型`Person`也是泛型的，但我们还添加了一个trait约束，即该类型必须实现`NamedPerson`trait。

The `PersonQuerier` is now defined with `PersonContext` as another of its supertraits. With that, the `query_person` method becomes completely abstract. A generic consumer would only know that, given an abstract type `Context::PersonId`, it can query the `Context` and either get back an abstract type `Context::Person` that implements `NamedPerson`, or get back an abstract error `Context::Error`.

> 现在，`PersonQuerier`被定义为另一个supertrait `PersonContext`。这样，`query_person`方法变得完全抽象。通用的使用者只知道，给定一个抽象类型`Context::PersonId`，它可以查询`Context`，然后要么得到实现`NamedPerson`的抽象类型`Context::Person`，要么得到抽象的错误`Context::Error`。
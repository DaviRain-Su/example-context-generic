# 查询消费者（Querier Consumer）

```rust
struct SimpleGreeter;

impl<Context> Greeter<Context> for SimpleGreeter
where
    Context: PersonContext + HasError,
    KvStorePersonQuerier: PersonQuerier<Context>,
{
    fn greet(&self, context: &Context, person_id: &Context::PersonId)
        -> Result<(), Context::Error>
    {
        let person = KvStorePersonQuerier::query_person(context, person_id)?;
        println!("Hello, {}", person.name());
        Ok(())
    }
}
```

## 通用查询消费者（Generic Querier Consumer）

Now that we have a context-generic implementation of `KvStorePersonQuerier`, we can try to use it from `SimpleGreeter`. To do that, `SimpleGreeter` has to somehow get `KvStorePersonQuerier` from Context and use it as a `PersonQuerier`.

> 现在我们已经有了一个上下文通用的`KvStorePersonQuerier`的实现，我们可以尝试从`SimpleGreeter`中使用它。为了实现这一点，`SimpleGreeter`必须以某种方式从`Context`中获取`KvStorePersonQuerier`并将其用作`PersonQuerier`。

Recall that `KvStorePersonQuerier` itself is not a context (though it does implement `PersonQuerier<Context>` in order to query a context), and therefore it does not implement other context traits like `PersonContext`. What we need instead is for concrete contexts like `AppContext` to specify that their implementation of `PersonQuerier` is `KvStorePersonQuerier`. We can do that by defining a `HasPersonQuerier` trait as follows:

> 回想一下，`KvStorePersonQuerier`本身不是一个上下文（尽管它实现了`PersonQuerier<Context>`以查询上下文），因此它不实现其他上下文特性，如`PersonContext`。我们需要的是具体的上下文，如`AppContext`，将其`PersonQuerier`实现指定为`KvStorePersonQuerier`。我们可以通过定义`HasPersonQuerier`trait来实现这一点：

```rust
trait PersonQuerier<Context>
where
    Context: PersonContext + HasError,
{
     fn query_person(context: &Context, person_id: &Context::PersonId)
         -> Result<Context::Person, Context::Error>;
}

trait HasPersonQuerier:
    PersonContext + HasError + Sized
{
    type PersonQuerier: PersonQuerier<Self>;

    fn query_person(&self, person_id: &Self::PersonId)
        -> Result<Self::Person, Self::Error>
    {
        Self::PersonQuerier::query_person(self, person_id)
    }
}
```

While the `PersonQuerier` trait is implemented by component types like `KvStorePersonQuerier`, the `HasPersonQuerier` trait is implemented by context types like `AppContext`. Compared to the earlier design of `PersonQuerier`, the context is now offering a component for querying for a person that will work in the current context.

> 将 `PersonQuerier` trait 的实现分配给组件类型，如 `KvStorePersonQuerier`，而 `HasPersonQuerier` trait 的实现分配给上下文类型，如 `AppContext`。与先前的 `PersonQuerier` 设计相比，现在上下文提供了一个在当前上下文中运行的查询个人信息的组件。

We can see that the `HasPersonQuerier` trait has `PersonContext` and `HasError` as its supertraits, indicating that the concrete context also needs to implement these two traits first. Due to quirks in Rust, the trait also requires the `Sized` supertrait, which is already implemented by most types other than `dyn Trait` types, so that we can use Self inside other generic parameters.

> 我们可以看到，`HasPersonQuerier` trait 有 `PersonContext` 和 `HasError` 作为它的父trait，表明具体的上下文还需要先实现这两个trait。由于 Rust 的一些怪异之处，该trait还需要 Sized 作为父trait，这个trait已经被大多数类型实现了，除了 `dyn Trait` 类型以外，因此我们可以在其他泛型参数中使用 Self。

In the body of `HasPersonQuerier`, we define a `PersonQuerier` associated type, which implements the trait `PersonQuerier<Self>`. This is because we want to have the following constraints satisfied:

- `AppContext: HasPersonQuerier` - AppContext implements the trait HasPersonQuerier.
- `AppContext::PersonQuerier`: `PersonQuerier<AppContext>` - The associated type `AppContext::PersonQuerier` implements the trait `PersonQuerier<AppContext>`.
- `KvStorePersonQuerier`: `PersonQuerier<AppContext>` - The type `KvStorePersonQuerier`, which we defined earlier, should implement `PersonQuerier<AppContext>`.
- `AppContext`: `HasPersonQuerier<PersonQuerier=KvStorePersonQuerier>` - We want to set the associated type - `AppContext::PersonQuerier` to be `KvStorePersonQuerier`.

> 在 `HasPersonQuerier` 的函数体中，我们定义了一个 `PersonQuerier` 关联类型，它实现了 `PersonQuerier<Self>` trait。这是因为我们希望满足以下约束条件：
>
> - `AppContext: HasPersonQuerier` - AppContext 实现了 `HasPersonQuerier` trait。
> - `AppContext::PersonQuerier: PersonQuerier<AppContext>` - 关联类型 `AppContext::PersonQuerier` 实现了 `PersonQuerier<AppContext>` trait。
> - `KvStorePersonQuerier: PersonQuerier<AppContext>` - 我们之前定义的类型 `KvStorePersonQuerier` 应该实现 `PersonQuerier<AppContext>` trait。
> - `AppContext: HasPersonQuerier<PersonQuerier=KvStorePersonQuerier>` - 我们希望将关联类型 `AppContext::PersonQuerier` 设置为 `KvStorePersonQuerier`。

In general, since we want any type Ctx that implements `HasPersonQuerier` to have the associated type `Ctx::PersonQuerier` to implement `PersonQuerier<Ctx>`. Hence inside the trait definition, we define the associated type as type PersonQuerier: `PersonQuerier<Self>`, where Self refers to the `Ctx` type.

> 通常情况下，我们希望任何实现了 `HasPersonQuerier` 的类型 Ctx，都具有关联类型 `Ctx::PersonQuerier` 实现 `PersonQuerier<Ctx>`。因此，在该 trait 的定义中，我们将关联类型定义为 `type PersonQuerier: PersonQuerier<Self>`，其中 Self 指的是 Ctx 类型。

This may look a little self-referential, as the context is providing a type that is referencing back to itself. But with the dependency injection mechanism of the traits system, this in fact works most of the time as long as there are no actual cyclic dependencies.

> 这可能看起来有点自我引用，因为上下文提供了一个引用回自身的类型。但是在 trait 系统的依赖注入机制下，只要没有实际的循环依赖，这通常是有效的。

Inside `HasPersonQuerier`, we also implement a `query_person` method with a `&self`, which calls `Self::PersonQuerier::query_person` to do the actual query. This method is not meant to be overridden by implementations. Rather, it is a convenient method that allows us to query from the context directly using `context.query_person()`.

> 在 `HasPersonQuerier` 中，我们还使用 `&self` 实现了 `query_person` 方法，该方法调用 `Self::PersonQuerier::query_person` 来进行实际的查询。该方法并不是用于被实现重写的。相反，它是一个方便的方法，允许我们直接使用 `context.query_person()` 从上下文中进行查询。

Now inside the Greet implementation for `SimpleGreeter`, we can require the generic `Context` to implement `HasPersonQuerier` as follows:

> 现在在 `SimpleGreeter` 的 `Greet` 实现中，我们可以如下定义泛型 `Context` 需要实现 `HasPersonQuerier`:

```rust
struct SimpleGreeter;

impl<Context> Greeter<Context> for SimpleGreeter
where
    Context: HasPersonQuerier,
{
    fn greet(&self, context: &Context, person_id: &Context::PersonId)
        -> Result<(), Context::Error>
    {
        let person = context.query_person(person_id)?;
        println!("Hello, {}", person.name());
        Ok(())
    }
}
```

Inside the `greet` method, we can call `context.query_person()` and pass in the context as the first argument to query for the person details.

> 在 greet 方法内部，我们可以调用 `context.query_person()` 方法并将上下文作为第一个参数传递来查询个人详细信息。

In summary, what we achieved at this point is as follows:

- We define a context-generic component for `PersonQuerier` as `KvStorePersonQuerier`.
- We define another context-generic component for Greet as `SimpleGreeter`, which depends on a `PersonQuerier` component provided from the context.
- The Rust trait system resolves the dependency graph, constructs a `KvStorePersonQuerier` using its indirect dependencies from the context, and passes it as the PersonQuerier dependency to `SimpleGreeter`.
  
> 总结一下，我们目前已经实现了以下几点：
>
> - 我们定义了一个泛型组件`PersonQuerier`作为上下文的一部分，即`KvStorePersonQuerier`。
> - 我们定义了另一个泛型组件`SimpleGreeter`，它依赖于上下文提供的`PersonQuerier`组件。
> - Rust的trait系统解决了依赖关系，使用上下文中的间接依赖项构造了一个`KvStorePersonQuerier`，并将其作为`SimpleGreeter`的`PersonQuerier`依赖项传递。
  
By using dependency injection, we don't need to know about the fact that in order to build `SimpleGreeter`, we need to first build `KvStorePersonQuerier`, but in order to build `KvStorePersonQuerier`, we need to first build `FsKvStore`.

> 通过使用依赖注入，我们不需要知道要构建 `SimpleGreeter`，我们需要首先构建 `KvStorePersonQuerier`，但是为了构建 `KvStorePersonQuerier`，我们需要首先构建 `FsKvStore`的事实。

By leveraging dependency injection, we don't need to know that building `SimpleGreeter` requires first building `KvStorePersonQuerier`, which itself requires first building `FsKvStore`. The compiler resolves all of these dependencies at compile time for free, and we do not even need to pay for the cost of doing such wiring at run time.

> 通过利用依赖注入，我们无需知道构建 `SimpleGreeter` 需要首先构建 `KvStorePersonQuerier`，而构建 `KvStorePersonQuerier` 又需要首先构建 `FsKvStore` 的事实。编译器在编译时免费解析所有这些依赖关系，我们甚至不需要为在运行时进行此类连接付出代价。

## 无自我组件（Selfless Components）

```rust
trait PersonQuerier<Context>
where
    Context: PersonContext + HasError,
{
     fn query_person(&self, context: &Context, person_id: &Context::PersonId)
         -> Result<Context::Person, Context::Error>;
}

trait HasPersonQuerier:
    PersonContext + HasError + Sized
{
    type PersonQuerier: PersonQuerier<Self>;

    fn person_querier(&self) -> &Self::PersonQuerier;
}

struct SimpleGreeter;

impl<Context> Greeter<Context> for SimpleGreeter
where
    Context: HasPersonQuerier,
{
    fn greet(&self, context: &Context, person_id: &Context::PersonId)
        -> Result<(), Context::Error>
    {
        let person = context.person_querier().query_person(context, person_id)?;
        println!("Hello, {}", person.name());
        Ok(())
    }
}
```
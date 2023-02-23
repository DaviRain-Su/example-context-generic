# Simple Greeter

Let's say we want to write a simple greeter program that greets a person. The simplest way is to write a function like follows:

当我们想要编写一个简单的问候程序来问候一个人时，最简单的方法是编写一个如下的函数：

```rust
fn greet(name: String) {
    println!("Hello, {}!", name);
}
```

When calling the `greet` function from a larger context, we may want to pass a `Person` struct with a `name` attribute on it, so that the caller does not have to know how to get the person's name.

当我们在一个更大的上下文中调用`greet`函数时，我们可能希望传递一个具有`name`属性的`Person`结构体作为参数，以便调用者不需要知道如何获取个人的名字。

```rust
struct Person {
    name: String,
    address: String,
    // ...
}

fn greet(person: &Person) {
    println!("Hello, {}!", person.name);
}
```

But the caller of the `greet` function might not have the person's information on hand, as it may be stored in a database. So we might want to implement a `greet` function that accepts a `PersonId` and a database handler, so that it can load the person's information from the database, and then greets them.

但是，调用`greet`函数的程序可能没有个人的信息，因为这些信息可能存储在数据库中。因此，我们可能需要实现一个`greet`函数，它接受`PersonId`和数据库处理程序作为参数，以便它可以从数据库加载个人的信息，然后向他们致以问候。


```rust
struct PersonId(String);
struct Person {
    id: PersonId,
    name: String,
    address: String,
    // ...
}

struct Database { /* ... */ }
struct DbError { /* ... */ }

impl Database {
    fn query_person(&self, person_id: &PersonId) -> Result<Person, DbError> {
        unimplemented!() // stub
    }
}

fn greet(db: &Database, person_id: &PersonId) -> Result<(), DbError> {
    let person = db.query_person(person_id)?;
    println!("Hello, {}", person.name);
    Ok(())
}
```

As the application grows, we can see that the complexity creeps in pretty quickly even with such a simple example:

- The full details of the `Person` struct must be fetched regardless of whether the `greet` function needs it.
- The concrete implementation of `Database` is exposed to the greet function, making it difficult to work with other databases.
- The concrete error `DbError` from the database query is leaked into the `greet` function implementation.

随着应用程序的增长，即使在这样一个简单的示例中，我们也可以看到复杂性很快就会悄悄蔓延：

- 必须获取`Person`结构的完整细节，而不管`greet`函数是否需要。
- 数据库的具体实现暴露给`greet`函数，使得难以使用其他数据库。
- 数据库查询的具体错误`DbError`泄漏到了`greet`函数的实现中。

When the application is still in its early stages, it might be tempting to leave these concerns aside and not worry about them too much. But eventually, we will reach a point where we need our application to work with different implementations. For example:

- We may want a caching layer to cache the person's information instead of querying directly from the database all the time.
- We may want to have different database implementations, such as a mocked-up database or an in-memory database.
- We may want to have multiple concrete person types, so that the database only fetches the essential information. e.g. `PersonWithName`, `PersonWithFullDetails`, `PersonWithRoles` etc.


当应用程序还处于早期阶段时，放弃这些问题并不去太担心它们可能很诱人。但最终，我们将到达一个需要使我们的应用程序与不同实现一起工作的点。例如：

- 我们可能希望有一个缓存层来缓存人的信息，而不是直接从数据库中查询。
- 我们可能希望有不同的数据库实现，比如一个模拟数据库或一个内存数据库。
- 我们可能希望有多个具体的人员类型，以便数据库只获取必要的信息，例如：`PersonWithName`、`PersonWithFullDetails`、`PersonWithRoles`等。

## Comparison with Dynamic Typing

One thing worth noting with our `greet` example in Rust is that many of the problems mentioned are applicable because we are programming in a statically-typed language. If we were to re-implement the `greet` function in a dynamically- typed language like JavaScript, many of these problems go away:

需要注意的是，我们在Rust中的`greet`示例中提到的许多问题是适用的，因为我们正在使用静态类型语言进行编程。如果我们在动态类型语言（例如JavaScript）中重新实现`greet`函数，许多这些问题将不复存在：

```js
function greet(db, personId) {
    const person = db.queryPerson(personId)
    console.log(`Hello, ${person.name}!`)
}
```

Thanks to dynamic typing, the JavaScript `greet` function above is general in several ways:

- The function can work with any `db` value, as long as it provides a valid `query_person` method.
- The function can work with any `person` value returned from `db.query_person`, as long as it contains a `name` field that can be converted into a string.
- The error can be thrown implicitly by `db.query_person` as an exception.

由于动态类型的特性，上面的JavaScript `greet`函数具有以下几个通用性：

- 只要`db`值提供了有效的`query_person`方法，该函数就可以与任何`db`值一起使用。
- 只要从`db.query_person`返回的任何`person`值包含可以转换为字符串的`name`字段，该函数就可以与任何person值一起使用。
- 错误可以由`db.query_person`隐式地作为异常抛出。

On the upside, the dynamic nature of the `greet`s function means that it can easily be reused across multiple database and person implementations. On the downside, since there is no type information, it is easy to accidentally call `greet` with invalid implementations and only discover the errors late during runtime execution.

从好的方面来看，`greet`函数的动态特性意味着它可以轻松地在多个数据库和人员实现中重复使用。但从不好的方面来看，由于没有类型信息，很容易在无意中使用无效的实现来调用`greet`，并且只会在运行时执行时才会发现错误。

Ideally, we would like to have the same benefits of writing generalized programs in dynamically-typed contexts, but still enjoy the benefits of type checking when there are mismatches in the specialized implementation.

理想情况下，我们希望在动态类型环境中编写通用程序时拥有相同的好处，但在专业化实现存在不匹配时仍然可以享受类型检查的好处。

## Dynamic Context

The first thing to notice when writing generalized functions is that there are usually contextual values in the surrounding environment that are needed for the program to execute successfully.

编写通用函数时需要注意的第一件事是，通常需要周围环境中的上下文值才能使程序成功执行。

In our dynamic `greet` example, we can generalize the `db` value and think of it as a `context` value, which may contain other environment parameters such as what kind of greeting is used.

在动态的`greet`示例中，我们可以将`db`值泛化，并将其视为上下文值，该上下文值可能包含其他环境参数，例如使用哪种问候语。

```js
function greet(context, personId) {
    const person = context.queryPerson(personId)
    const greeting = context.getGreeting()
    console.log(`${greeting}, ${person.name}!`)
}
```

In the OOP world, the `context` value is typically referred to as a `this` or `self` value. However, for clarity and for more structured composition, it is better to think of it as a fully abstract value with unknown type. This allows the context value to be augmented in a functional way, without having to resort to using any OOP class hierarchy.

在面向对象编程的世界中，上下文值通常被称为`this`或`self`值。但是，为了清晰起见并且为了更结构化的组合，最好将其视为具有未知类型的完全抽象值。这允许通过函数式的方式增强上下文值，而无需使用任何面向对象的类层次结构。
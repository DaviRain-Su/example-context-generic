# Greeter Component

The `greet` function that we have defined at this point can now work with any context type that implements the required traits. In practice, we may also want to implement different versions of the `greet` function so that they can be consumed generically by other components.

到目前为止，我们定义的`greet`函数现在可以与任何实现所需trait的上下文类型一起使用。在实践中，我们可能还希望实现不同版本的`greet`函数，以便它们可以被其他组件通用地使用。

With the generic context design, we define a `Greeter` interface that is parameterized by a generic context, and can be used by any other components that also share the same context. This can be defined as follows:

使用泛型上下文设计，我们定义了一个通过泛型上下文参数化的`Greeter`接口，可以被与其共享相同上下文的任何其他组件使用。这可以定义如下：

```rust
trait Greeter<Context>
where
    Context: PersonContext + HasError,
{
    fn greet(
        &self,
        context: &Context,
        person_id: &Context::PersonId,
    ) -> Result<(), Context::Error>;
}

struct SimpleGreeter;

impl<Context> Greeter<Context> for SimpleGreeter
where
    Context: PersonQuerier,
{
    fn greet(
        &self,
        context: &Context,
        person_id: &Context::PersonId,
    ) -> Result<(), Context::Error>
    {
        let person = context.query_person(person_id)?;
        println!("Hello, {}", person.name());
        Ok(())
    }
}
```

The `Greeter` trait is defined to be parameterized by a generic `Context` type, which is required to implement both `PersonContext` and `HasError`. The greet method is then defined without generic parameters, as these have been captured in the trait definition. We then define an empty struct `SimpleGreeter`, which is there only to implement the Greeter trait for any `Context` type that implements `PersonQuerier`.

`Greeter trait`被定义为以泛型`Context`类型参数化，该类型需要实现`PersonContext`和`HasError`。`greet`方法则定义了没有泛型参数，因为这些已被捕获在trait定义中。然后，我们定义了一个空的`SimpleGreeter`结构体，仅用于为实现`PersonQuerier`的任何`Context`类型实现`Greeter`trait。

It is worth noticing here that in the main `Greeter` trait definition, the `Context` type, is only required to implement `PersonContext` and `HasError`, but there is no mention of the `PersonQuerier` trait bound. On the other hand, the concrete Greeter implementation for SimpleGreeter can require the additional trait bound Context: PersonQuerier in its impl definition.

值得注意的是，在`Greeter trait`的主要定义中，`Context`类型只需要实现`PersonContext`和`HasError`，没有提到`PersonQuerier`特质约束。另一方面，`SimpleGreeter`的具体实现可以在其impl定义中要求额外的trait约束`Context: PersonQuerier`。

This demonstrates the benefits of separating the `PersonQuerier` from the `PersonContext trait`: From the perspective of a consumer that uses the `Greeter` component, it does not need to know whether the generic context type implements `PersonQuerier`. This means that from the trait bounds alone, we can tell whether a piece of code can call `query_person` directly, or whether it can only call the greet method to `greet` a person without knowing how the greeter determined the person's name.

这证明了将`PersonQuerier`与`PersonContext` trait 分离的好处：从使用`Greeter`组件的消费者的角度来看，它不需要知道泛型上下文类型是否实现`PersonQuerier`。这意味着仅从trait 约束就可以知道一个代码片段是否可以直接调用`query_person`，或者它是否只能调用`greet`方法来向一个人问候，而不知道问候者是如何确定这个人的名字的。

## Greeter Instance

In the previous chapter, we defined AppContext as a concrete implementation for the context traits HasError, PersonContext, and PersonQuerier. Based on the earlier definition of SimpleGreeter and its Greeter implementation, we can deduce that SimpleGreeter should implement `Greeter<AppContext>`.

在之前的章节中，我们为上下文特质`HasError`、`PersonContext`和`PersonQuerier`定义了`AppContext`的具体实现。根据早期定义的`SimpleGreeter`和其`Greeter`实现，我们可以推断`SimpleGreeter`应该实现`Greeter<AppContext>`。

But how can we ensure that we implemented `AppContext` correctly to be used by `SimpleGreeter`? If we forgot to implement any dependency that is required by `SimpleGreeter`, such as `PersonQuerier`, we would get a compile-time error when trying to use SimpleGreeter as `Greeter<AppContext>`. Worse, if we try to use `SimpleGreeter` within another generic component, the error messages may become too incomprehensible to find out what went wrong.

但是，我们如何确保我们已正确实现了`AppContext`以供`SimpleGreeter`使用？如果我们忘记实现`SimpleGreeter`所需的任何依赖项，例如`PersonQuerier`，则在尝试将`SimpleGreeter`用作`Greeter<AppContext>`时会得到编译时错误。更糟糕的是，如果我们尝试在另一个泛型组件内使用`SimpleGreeter`，则错误消息可能会变得过于难以理解，以至于无法找出错误所在。

In test-driven development, it is common practice that we would write tests that check that our code satisfies certain requirements and constraints. Following the same principle, we would want to write tests that check that `SimpleGreeter` does implements `Greeter<AppContext>`. But instead of writing dynamic tests that checks for the program behavior at runtime, we can write static tests that checks that the program requirements are satisfied at compile time.

在测试驱动开发中，通常会编写测试以检查我们的代码是否满足某些要求和约束。遵循相同的原则，我们希望编写测试以检查`SimpleGreeter`是否实现了`Greeter<AppContext>`。但是，我们可以编写静态测试来检查在编译时是否满足程序要求，而不是编写动态测试以在运行时检查程序行为。

Our test would be implemented as an app_greeter function which checks at compile time that SimpleGreeter implements `Greeter<AppContext>`:

我们的测试将被实现为一个`app_greeter`函数，该函数在编译时检查`SimpleGreeter`是否实现了`Greeter<AppContext>`：

```rust
fn app_greeter() -> impl Greeter<AppContext> {
    SimpleGreeter
}
```

Our `app_greeter` function accepts nothing and returns `impl Greeter<AppContext>`. This indicates that the function can return a value with an existential type that implements `Greeter<AppContext>`. Inside the function body, we simply return a SimpleGreeter value. From the surface, it may look like this is effectively the same as the function with signature `fn app_greeter() -> SimpleGreeter`. But by returning `impl Greeter<AppContext>`, we force the Rust compiler to check here that SimpleGreeter must implement `Greeter<AppContext>`.

我们的`app_greeter`函数不接受任何参数，并返回`impl Greeter<AppContext>`。这表明该函数可以返回一个存在类型的值，该类型实现了`Greeter<AppContext>`。在函数体内，我们只需返回一个`SimpleGreeter`值。从表面上看，它可能看起来实际上与签名为`fn app_greeter() -> SimpleGreeter`的函数相同。但是，通过返回`impl Greeter<AppContext>`，我们强制Rust编译器在此处检查`SimpleGreeter`是否必须实现`Greeter<AppContext>`。

Having `app_greeter` compiled successfully is sufficient to prove that `SimpleGreeter` can always be used as a `Greeter<AppContext>`. Hence, we can say that app_greeter is a proof that `SimpleGreeter`: `Greeter<AppContext>`.

如果`app_greeter`编译成功，就足以证明`SimpleGreeter`始终可以用作`Greeter<AppContext>`。因此，我们可以说`app_greeter`是一个证明`SimpleGreeter：Greeter<AppContext>`的证明。

We call static tests like `app_greeter` as proofs, as it works similar to writing mathematical proofs as programs in dependent-typed programming. In this style of programming,the type-checking of a program alone is sufficient to prove that a given requirement is always satisfied, without us having to execute the program. This is much more efficient as compared to dynamic checking, which can only check for requirements at runtime and raise errors when the requirements are not satisfied.

我们像`app_greeter`这样的静态测试称为证明，因为它类似于在依赖类型编程中编写数学证明作为程序。在这种编程风格中，仅程序的类型检查就足以证明给定的要求始终得到满足，而无需我们执行程序。这比动态检查更有效，后者只能在运行时检查要求并在不满足要求时引发错误。

## Compile-Time Dependency Injection

The `app_greeter` function demonstrates a form of dependency injection done at compile time. This is because for any code to use a type implementing `Greeter<Context>`, they only need to know that `Context` implements `HasError` and `PersonContext`. But to make SimpleGreeter implement `Greeter<Context>`, it also needs `Context` to implement `PersonQuerier`.

`app_greeter`函数展示了一种在编译时进行的依赖项注入形式。这是因为对于任何要使用实现了`Greeter<Context>`的类型的代码，它们只需要知道`Context`实现了`HasError`和`PersonContext`。但是，要使`SimpleGreeter`实现`Greeter<Context>`，它还需要`Context`实现`PersonQuerier`。

When we return `SimpleGreeter` inside `app_greeter`, the Rust compiler figures out that `SimpleGreeter` requires `AppContext` to implement `PersonQuerier`. It would then try to automatically resolve the dependency by searching for an implementation of `PersonQuerier` for `AppContext`. Upon finding the implementation, Rust "binds" that implementation with `SimpleGreeter` and returns it as an existential type that implements `Greeter<AppContext>`. As a result, we can treat the type returned from app_greeter as an abstract type, and "forget" the fact that `AppContext` implements `PersonQuerier`.

当我们在`app_greeter`中返回`SimpleGreeter`时，Rust编译器会发现`SimpleGreeter`要求`AppContext`实现`PersonQuerier`。然后它会尝试通过搜索`AppContext`的`PersonQuerier`实现来自动解决依赖关系。找到实现后，Rust会将该实现与`SimpleGreeter`“绑定”，并将其作为实现`Greeter<AppContext>`的存在类型返回。因此，我们可以将从`app_greeter`返回的类型视为抽象类型，并“遗忘”了`AppContext`实现了`PersonQuerier`的事实。

This pattern of making use of Rust's trait system for dependency injection efficiently solves the context and capabilities problem in Rust. Without it, we would have to rely on more exotic language features that are not available in Rust, or resort to manually passing dependencies around by hand.

在 Rust 中，利用 trait 系统进行依赖注入的这种模式有效地解决了上下文和能力问题。如果没有这种模式，我们可能会依赖于更为奇特的语言特性，而这些特性在 Rust 中并不可用，或者不得不手动传递依赖项。通过使用 trait 和关联类型，我们可以定义与环境中满足一定要求的任何上下文一起使用的通用组件，同时保持高度的类型安全性和模块化。这种方法允许我们使用最少的样板代码和重复来构建复杂和可扩展的应用程序。

For example, we could perform manual binding for an implementation similar to SimpleGreeter as a purely generic function as follows:

我们可以手动绑定实现类似于SimpleGreeter的纯泛型函数，例如:

```rust
fn make_simpler_greeter<Context, PersonId, Person, Error>(
    query_person: impl Fn(&Context, &PersonId) -> Result<Person, Error>,
) -> impl Fn(&Context, &PersonId) -> Result<(), Error>
where
    Person: NamedPerson,
{
    move | context, person_id | {
        let person = query_person(context, person_id)?;
        println!("Hello, {}", person.name());
        Ok(())
    }
}
```

As we can see, the ad hoc function make_simpler_greeter that we defined is much more verbose than the earlier trait-based implementation. We would have to explicitly track 4 generic type parameters, and we would have to manually pass in dependencies like query_person and return nested closures.

When we delegate the management of the context dependencies to Rust's trait system, the Rust compiler handles the binding of dependencies automatically in a manner similar to what is being done in the example above. The binding process is not only automated, the code that the compiler generates is also much more efficient. Because the binding is done at compile time, the Rust compiler is able to perform many further optimizations, such as code inlining.

As we will see later, this process of automatic resolution can be applied to nested dependencies. Thanks to how the trait system works, we can specify complex dependencies for our components and have the Rust compiler figure out how to stitch together the dependencies and construct the combined components that we need.


正如我们所见，我们定义的临时函数 `make_simpler_greeter` 比之前基于 trait 的实现要冗长得多。我们必须显式跟踪 4 个泛型类型参数，并手动传递依赖项，如 `query_person`，并返回嵌套闭包。

当我们将上下文依赖关系的管理委托给 Rust 的 trait 系统时，Rust 编译器会自动处理依赖关系的绑定，类似于上面的示例。绑定过程不仅是自动的，编译器生成的代码也更加高效。因为绑定是在编译时完成的，Rust 编译器能够执行许多进一步的优化，例如代码内联。

正如我们稍后将看到的那样，这种自动解析的过程可以应用于嵌套依赖项。由于 trait 系统的工作方式，我们可以为组件指定复杂的依赖关系，并让 Rust 编译器找出如何将依赖关系拼合在一起并构造我们需要的组合组件。
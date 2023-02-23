# 组件组合 （Component Composition）

Now that we have both `SimpleGreeter` and `DaytimeGreeter` implemented, we can look at how we can define a full application context that satisfies the constraints of both `greeters`. To better structure our application, we also separate out different parts of the code into separate modules.

> 现在我们已经实现了 `SimpleGreeter` 和 `DaytimeGreeter`，接下来我们看一下如何定义一个完整的应用程序上下文，以满足两个 `greeter` 的约束。为了更好地组织我们的应用程序，我们还将不同部分的代码分成不同的模块。

First, we put all the abstract traits into a traits module:

> 首先，我们将所有抽象特征放入一个 traits 模块中：

```rust
mod app {
    mod traits {
        pub trait NamedPerson {
            fn name(&self) -> &str;
        }

        pub trait SimpleTime {
            fn is_daytime(&self) -> bool;
        }

        pub trait HasError {
            type Error;
        }

        pub trait PersonContext {
            type PersonId;
            type Person: NamedPerson;
        }

        pub trait HasTime {
            type Time;

            fn now(&self) -> Self::Time;
        }
    }

    // ...
}
```

This module does not contain any concrete type definitions, and thus has minimal dependencies on external crates.

> 这个模块不包含任何具体的类型定义，因此对外部 crate 的依赖非常小。

In practice, the trait definitions can be placed in different sub-modules so that we can have more fine grained control over which traits a component depends on.

> 在实践中，trait定义可以放置在不同的子模块中，这样我们就可以更精细地控制组件依赖的trait。

Next, we define SimpleGreeter and DaytimeGreeter in separate modules.

> 接下来，我们在不同的模块中定义 `SimpleGreeter`和`DaytimeGreeter`：

```rust
mod app {
    mod traits {
        // ...
    }

    mod simple_greeter {
        use super::traits::{Greeter, NamedPerson, PersonQuerier};

        pub struct SimpleGreeter;

        impl<Context> Greeter<Context> for SimpleGreeter
        where
            Context: PersonQuerier,
        {
            fn greet(&self, context: &Context, person_id: &Context::PersonId)
                -> Result<(), Context::Error>
            {
                let person = context.query_person(person_id)?;
                println!("Hello, {}", person.name());
                Ok(())
            }
        }
    }

    mod daytime_greeter {
        use super::traits::{
            Greeter, HasError, PersonContext,
            HasTime, SimpleTime,
        };

        pub struct DaytimeGreeter<InGreeter>(pub InGreeter);

        pub struct ShopClosedError<Time> { time: Time }

        impl<Context, InGreeter, Time, Error, PersonId>
            Greeter<Context> for DaytimeGreeter<InGreeter>
        where
            InGreeter: Greeter<Context>,
            Context: HasError<Error=Error>,
            Context: PersonContext<PersonId=PersonId>,
            Context: HasTime<Time=Time>,
            Time: SimpleTime,
            Error: From<ShopClosedError<Time>>,
        {
            fn greet(&self, context: &Context, person_id: &PersonId)
                -> Result<(), Error>
            {
                let now = context.now();
                if now.is_daytime() {
                    self.0.greet(context, person_id)
                } else {
                    Err(ShopClosedError { time: now }.into())
                }
            }
        }
    }

    // ...
}
```

The two `greeter` components do not depend on each other, but they all depend on the traits crate to make use of the abstract definitions. Since these components do not depend on other crates, they are also abstract components that can be instantiated with any context types that satisfy the trait bounds.

> 这两个 `greeter` 组件不相互依赖，但它们都依赖于 traits 模块，以利用抽象定义。由于这些组件不依赖于其他 crate，它们也是抽象组件，可以用满足 trait 约束的任何上下文类型实例化。

Next, we define our concrete AppContext struct that implements all context traits:

> 我们接下来定义一个具体的`AppContext`结构体，它实现了所有上下文特性：

```rust
mod app {
    mod traits {
        // ...
    }

    mod simple_greeter {
        // ...
    }

    mod daytime_greeter {
        pub struct ShopClosedError<Time> { time: Time }
        // ...
    }

    mod context {
        use super::traits::*;
        use super::daytime_greeter::ShopClosedError;

        #[derive(Copy, Clone, PartialEq, Eq)]
        pub enum DummyTime {
            DayTime,
            NightTime,
        }

        pub struct BasicPerson {
            name: String,
        }

        pub struct AppContext {
            database: Database,
            time: DummyTime,
        }

        // Database stubs
        struct Database;
        struct DbError;

        pub enum AppError {
            Database(DbError),
            ShopClosed(ShopClosedError<DummyTime>),
            // ...
        }

        impl HasError for AppContext {
            type Error = AppError;
        }

        impl PersonContext for AppContext {
            type PersonId = String;
            type Person = BasicPerson;
        }

        impl HasTime for AppContext {
            type Time = DummyTime;

            fn now(&self) -> DummyTime {
                self.time
            }
        }

        impl PersonQuerier for AppContext {
            fn query_person(&self, person_id: &Self::PersonId)
                -> Result<Self::Person, Self::Error>
            {
                unimplemented!() // database stub
            }
        }

        impl NamedPerson for BasicPerson {
            fn name(&self) -> &str {
                &self.name
            }
        }

        impl SimpleTime for DummyTime {
            fn is_daytime(&self) -> bool {
                self == &DummyTime::DayTime
            }
        }

        impl From<ShopClosedError<DummyTime>> for AppError {
            fn from(err: ShopClosedError<DummyTime>) -> Self {
                Self::ShopClosed(err)
            }
        }
    }
}
```

Compared to before, we define a `DummyTime` struct that mocks the current time with either day time or night time. We then implement HasTime for AppContext, with `DummyTime` being the Time type. We also add `ShopClosedError<DummyTime>` as a variant to AppError and define a From instance for it.

> 在这个例子中，我们定义了一个 `DummyTime` 结构体，来模拟当前时间是白天还是晚上。我们接着为 `AppContext` 实现了 `HasTime` trait，其中的 `Time` 类型是 `DummyTime`。我们还为 AppError 添加了 `ShopClosedError<DummyTime>` 变体，并定义了相应的 From 实例。

As we can see in this exercise, by having all types used by the `greeter` components as abstract types, it becomes very easy to mock up dependencies such as time functionality without having to commit to a specific time library. The explicit dependencies also help us better understand what features are really needed from the concrete types. If we know that our application only needs the `SimpleTime` trait, then there are more options out there that we can try out and we can easily swap between them.

> 我们可以看到，将所有 `greeter` 组件使用的类型都定义为抽象类型后，我们就可以很容易地模拟诸如时间功能这样的依赖关系，而无需针对特定的时间库进行编写。明确的依赖关系还有助于我们更好地了解哪些功能真正需要来自具体类型。如果我们知道我们的应用程序只需要 `SimpleTime` trait，那么就有更多的选择可以尝试，并且我们可以轻松地在它们之间进行切换。


It is also worth noting that it doesn't matter whether the concrete types `AppContext` and `BasicPerson` have private or public fields. Since the components do not have access to the concrete types, all concrete fields are essentially private and can only be exposed via trait methods.

> 另外，值得注意的是，无论具体类型 `AppContext` 和 `BasicPerson` 的字段是私有的还是公开的，都没有关系。由于组件无法访问具体类型，所有具体字段都可以看作是私有的，只能通过 trait 方法公开。

Finally, we define an instances module to encapsulate the witness of satisfying all dependencies required from `AppContext` to implement the `Greeter` components:

> 最后，我们定义了一个 `instances` 模块，用于封装满足从 `AppContext` 到实现 `Greeter` 组件所需的所有依赖关系的证明。

```rust
mod app {
    mod traits {
        // ...
    }

    mod simple_greeter {
        // ...
    }

    mod daytime_greeter {
        // ...
    }

    mod context {
        // ...
    }

    mod instances {
        use super::traits::Greeter;
        use super::context::AppContext;
        use super::simple_greeter::SimpleGreeter;
        use super::daytime_greeter::DaytimeGreeter;

        pub fn base_greeter() -> impl Greeter<AppContext> {
            SimpleGreeter
        }

        pub fn app_greeter() -> impl Greeter<AppContext> {
            DaytimeGreeter(base_greeter())
        }
    }
}
```

We first have a `base_greeter` function which witnesses that `SimpleGreeter` implements `Greeter<AppContext>`. We then define an `app_greeter` function which witnesses that `DaytimeGreeter<SimpleGreeter>` also implements `Greeter<AppContext>`.

> 我们首先有一个 `base_greeter` 函数，该函数证明了 `SimpleGreeter` 实现了 `Greeter<AppContext>`。接下来，我们定义了一个 `app_greeter` 函数，该函数证明了 `DaytimeGreeter<SimpleGreeter>`   实现了 `Greeter<AppContext>`。

Notice that in the `app_greeter` body, we construct the greeter with `DaytimeGreeter(base_greeter())` instead of `DaytimeGreeter(SimpleGreeter)`. In theory, both expressions are valid and have the same effect, but calling `base_greeter` inside `app_greeter` implies that `app_greeter` does not care what the concrete type of `base_greeter` is; all that matters is that it implements `Greeter<AppContext>`.

请注意，在 `app_greeter` 函数的主体中，我们使用 `DaytimeGreeter(base_greeter())` 构造了 `greeter`，而不是 `DaytimeGreeter(SimpleGreeter)`。理论上，这两个表达式都是有效的，具有相同的效果，但在 `app_greeter` 中调用 `base_greeter` 意味着 `app_greeter` 不关心 `base_greeter` 的具体类型；重要的是它实现了 `Greeter<AppContext>`

Having separate witness functions can also help us debug any errors that arise in dependencies much more easily. Let's say that we forgot to implement `PersonQuerier` for `AppContext` such that the dependency for `SimpleGreeter` would not be satisfied; we would get a type error in base_greeter. However, no errors would crop up in `app_greeter`, because it doesn't care that base greeter implements `SimpleGreeter`.

> 分别有不同的函数来作为见证函数也可以帮助我们更轻松地调试依赖项引起的任何错误。假设我们忘记为`AppContext`实现`PersonQuerier`以使得`SimpleGreeter`的依赖项不被满足；我们会在`base_greeter`中得到类型错误。然而，`app_greeter`中不会出现任何错误，因为它不关心`base_greeter`是否实现了`SimpleGreeter`。

If we were to write the complex expression in one go, like `DaytimeGreeter(SimpleGreeter)`, it would be less clear which part of the expression caused the type error. Things would get worse if we introduced more complex component composition. Therefore, it is always a good practice to define the component instantiation in multiple smaller functions so that it is clear to the reader whether the dependencies are being resolved correctly.

> 如果我们一次性编写复杂的表达式，比如 `DaytimeGreeter(SimpleGreeter)`，那么很难清楚地看出哪个部分导致了类型错误。如果引入更复杂的组件组合，情况会变得更糟。因此，将组件实例化定义为多个较小函数总是一个好习惯，这样读者可以清楚地看到依赖项是否被正确解决。

## 组件图可视化 （Component Graph Visualization）

## Reader单子 （Reader Monad）

Readers coming from a functional programming background might notice that the context pattern looks similar to the `reader monad` pattern. This is correct, as we are defining a global Context type and passing it around as a function argument to all code that requires the context. Additionally, we make use of the   trait (typeclass)    system in Rust for compile-time dependency injection, and the same pattern can be applied for the context type used in `reader monads`.

> 从函数式编程的角度，读者可能会注意到上下文模式看起来类似于 `reader monad` 模式。这是正确的，因为我们定义了一个全局的 Context 类型，并将其作为函数参数传递给所有需要上下文的代码。此外，我们利用 Rust 的 t`rait (typeclass)` 系统进行编译时依赖注入，而这种模式也可以应用于 `reader monad` 中使用的上下文类型。

For Rust readers, the main difference of the pattern described here with the `reader monad` is that we are passing the context value as an explicit argument without making use of any monadic constructs. Doing it this way is slightly more verbose, but the upside is that we still get to enjoy much of the benefits of the `reader monad` pattern without requiring Rust programmers to learn what a monad really is (though if you're comfortable with using `Result` and `Option`, you've already been making use of `monads`).

> 对于 Rust 读者来说，这种模式与 `reader monad` 的主要区别在于，我们将上下文值作为显式参数传递，而没有使用任何单子结构。这样做稍微有点冗长，但好处是我们仍然可以享受 `reader monad` 模式的许多好处，而无需要求 Rust 程序员学习什么是 `monad`（虽然如果您熟悉使用 `Result` 和 `Option`，您已经在使用 `monad` 了）。
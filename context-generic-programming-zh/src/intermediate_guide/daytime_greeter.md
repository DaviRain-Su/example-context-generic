# Daytime Greeter

Now suppose that we want to extend our greeter component such that it only greets a person at day time during office hours. We could directly modify the Greeter implementation for SimpleGreeter to do that, but that may complicate the implementation and makes it more difficult to understand the core logic. Alternatively, we could define a new DaytimeGreeter component that wraps around the original SimpleGreeter.

现在假设我们想要扩展我们的问候组件，使其仅在办公时间内的白天问候人。我们可以直接修改`SimpleGreeter`的`Greeter`实现来实现，但是那样可能会使实现变得更加复杂，并且更难以理解核心逻辑。或者，我们可以定义一个新的`DaytimeGreeter`组件，它包装在原始的`SimpleGreeter`组件周围。

This new DaytimeGreeter component would need to know how to get the current time of the system, as well as how to tell whether a given time value is at daytime. Following the context pattern we learned, we will also define a HasTime trait for getting the time:

这个新的`DaytimeGreeter`组件需要知道如何获取系统的当前时间，以及如何判断给定的时间值是否在白天。遵循我们所学的上下文模式，我们还将为获取时间定义一个`HasTime trait`:

```rust
trait SimpleTime {
    fn is_daytime(&self) -> bool;
}

trait HasTime {
    type Time;

    fn now(&self) -> Self::Time;
}
```

For demonstration purposes, we first define a SimpleTime trait that provides an is_daytime method to tell whether the current time value is considered daytime. Following that, we define a HasTime trait that provides a now method to fetch the current time from the context. Notice that the associated type Time does not implement SimpleTime. This is so that we can learn how to inject the SimpleTime constraint as an indirect dependency using the same dependency injection technique.

为了演示，我们首先定义一个`SimpleTime trait` ，提供一个 `is_daytime`方法，用于判断当前时间值是否被视为白天。接着，我们定义了一个`HasTime trait`，提供一个`now`方法，从上下文中获取当前时间。请注意，关联类型`Time`并未实现`SimpleTime`。这是为了我们能够学习如何使用同样的依赖注入技术将SimpleTime约束作为间接依赖注入。

We then define the DaytimeGreeter component as follows:

然后我们定义 `DaytimeGreeter` 组件，如下所示：

```rust
struct DaytimeGreeter<InGreeter>(InGreeter);

impl<Context, InGreeter> Greeter<Context> for DaytimeGreeter<InGreeter>
where
    InGreeter: Greeter<Context>,
    Context: HasTime + PersonContext + HasError,
    Context::Time: SimpleTime,
{
    fn greet(
        &self,
        context: &Context,
        person_id: &Context::PersonId,
    ) -> Result<(), Context::Error>
    {
        let now = context.now();
        if now.is_daytime() {
            self.0.greet(context, person_id)?;
        } else {
            println!("Sorry, the shop has closed now!");
        }
        Ok(())
    }
}
```

We define the`DaytimeGreeter` with an `InGreeter` type parameter, which would act as the inner `Greeter` component. We then define a generic implementation of `Greeter<Context>` for `DaytimeGreeter<InGreeter>`. In the trait bounds, we require the inner greeter InGreeter to also implement `Greeter<Context>`, since the core logic is implemented over there.

我们为`DaytimeGreeter`定义了一个`InGreeter`类型参数，它将作为内部`Greeter`组件。然后，我们为`DaytimeGreeter<InGreeter>`定义了一个通用的`Greeter<Context>`实现。在trait约束中，我们要求内部`Greeter InGreeter`也实现了`Greeter<Context>`，因为核心逻辑是在那里实现的。

Aside from `PersonContext` and `HasError`, we also require `Context` to implement `HasTime` for `DaytimeGreeter` to fetch the current time. Other than that, we also explicitly require that the associated type Context::Time implements SimpleTime.

除了 `PersonContext` 和 `HasError` 之外，我们还要求 `Context` 实现 `HasTime`，这是为了让 `DaytimeGreeter` 获取当前时间。除此之外，我们还显式要求 `Context::Time` 实现 `SimpleTime`。

By specifying `SimpleTime` as an explicit dependency, we relax the requirement of how the HasTime trait can be used by other components. So if SimpleTime is only ever used by DaytimeGreeter, and if an application does not need DaytimeGreeter, then a concrete context can skip implementing SimpleTime for its time type, even if the trait HasTime is used by other components.

通过指定`SimpleTime`为显式依赖项，我们放宽了`HasTime trait`可供其他组件使用的要求。因此，如果`SimpleTime`仅由`DaytimeGreeter`使用，并且应用程序不需要`DaytimeGreeter`，则具体上下文可以跳过为其时间类型实现`SimpleTime`，即使`HasTime trait`被其他组件使用。

## Error Injection

In our earlier implementation of `DaytimeGreeter`, the greeter simply prints out that the shop has closed, and then returns successfully without calling the inner greeter. But what if we want `DaytimeGreeter` to return an error during night time? Since the associated type Error in `HasError` is abstract, there is no obvious way we can construct an error value of type Error.

在之前的 `DaytimeGreeter` 实现中，当商店关闭时，这个 `Greeter` 只是简单地打印一条消息，然后成功地返回，没有调用内部 Greeter。但是，如果我们想让 `DaytimeGreeter` 在晚上返回一个错误怎么办？由于 `HasError` 中的关联类型 `Error` 是抽象的，我们没有明显的方法可以构造类型为 `Error` 的错误值。

On the other hand, we learned in the previous section that we can specify an additional trait bound that `Context::Time` implements `SimpleTime`. Similarly, we can also specify additional trait bounds for `Context::Error` so that we gain additional knowledge of how to construct an error value.

与此相反，在前面的章节中，我们学习了可以指定其他的 trait bound，以便  `Context::Time` 实现 `SimpleTime`。同样地，我们还可以为 `Context::Error` 指定其他的 trait bound，以便获取构造错误值的额外信息。

We can do this by defining a custom `ShopClosedError` struct and require that `Context::Error` implement a From instance for conversion from `ShopClosedError`:

我们可以通过定义一个自定义的 `ShopClosedError` 结构体，并要求 `Context::Error` 实现一个 From 实例，以便从 `ShopClosedError` 进行转换，从而实现这一点。

```rust
struct ShopClosedError<Time> { time: Time }

struct DaytimeGreeter<InGreeter>(InGreeter);

impl<Context, InGreeter> Greeter<Context> for DaytimeGreeter<InGreeter>
where
    InGreeter: Greeter<Context>,
    Context: HasTime + PersonContext + HasError,
    Context::Time: SimpleTime,
    Context::Error: From<ShopClosedError<Context::Time>>,
{
    fn greet(
        &self,
        context: &Context,
        person_id: &Context::PersonId,
    ) -> Result<(), Context::Error>
    {
        let now = context.now();
        if now.is_daytime() {
            self.0.greet(context, person_id)
        } else {
            Err(ShopClosedError { time: now }.into())
        }
    }
}
```

The `ShopClosedError` is parameterized by a generic Time type so that it can provide details about the time that caused `ShopClosedError` to be raised. In the `Greeter` implementation for `DaytimeGreeter` we add an additional trait bound to require `Context::Error` to implement `From<ShopClosedError<Context::Time>>`. With that, if the time returned by `context.now()` is not considered daytime, we can construct a `ShopClosedError` and turn it into `Context::Error` using the into method.

`ShopClosedError` 是一个泛型结构体，它的类型参数 `Time` 用于提供引发 `ShopClosedError` 错误的时间详细信息。在 `DaytimeGreeter` 的 `Greeter` 实现中，我们增加了一个额外的 trait bound，要求 `Context::Error` 实现 `From<ShopClosedError<Context::Time>>`。这样，如果 `context.now()` 返回的时间不被视为白天，我们就可以构造一个 `ShopClosedError` 并使用 into 方法将其转换为 `Context::Error`。

What we have done above is essentially specifying an error injection method for injecting a sub-error type into the main error type. With this, individual components do not need to know about the concrete application error and all the possible errors that can be raised. But they can still inject specific errors into the main error type by requiring additional From constraints.

上面的内容实际上是为注入错误类型定义了一个错误注入方法。借助这种方法，各个组件无需了解具体的应用程序错误和可能引发的所有错误类型，但是它们仍然可以通过需要额外的 From 约束将特定错误注入到主错误类型中。

For instance, `DaytimeGreeter` does not need to be aware of whether the inner greeter component would raise a database error. From the impl definition, we can be confident that `DaytimeGreeter` itself cannot raise any sub-error other than `ShopClosedError`.

例如，`DaytimeGreeter` 不需要知道内部 Greeter 组件是否会引发数据库错误。从 impl 的定义中，我们可以确信 `DaytimeGreeter` 本身不能引发任何除 `ShopClosedError` 之外的子错误。

## Multiple Type Bindings

When specifying the constraints for indirect dependencies, we have to keep using the Context:: prefix to access associated types like Context::Error. Worse, once we start using nested associated types, we have to resort to using fully qualified syntax like `<Context::Foo as Foo>::Bar; Context::Foo::Bar` doesn't work.

在指定间接依赖项的约束时，我们必须继续使用`Context::`前缀来访问关联类型，如`Context::Error`。更糟糕的是，一旦开始使用嵌套关联类型，我们必须使用完全限定的语法，例如`<Context::Foo as Foo>::Bar`;，而`Context::Foo::Bar`将不起作用。

To help simplify the trait bounds for components like `DaytimeGreeter`, we can use the explicit associated type bindings we learned about earlier:

我们可以使用之前学习的显式关联类型绑定来简化 `DaytimeGreeter` 组件的特质约束：

```rust
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
```

In our new `Greeter` implementation, we introduce the generic parameters `Time`, Error, and PersonId. We then bind the types to the associated types of the context traits, such as `HasError<Error=Error>`. With the bindings in place we can have simpler trait bounds like `Time: SimpleTime` to be specified in place of the more verbose `Context::Time: SimpleTime`.


在我们的新的`Greeter`实现中，我们引入了泛型参数`Time`、`Error`和`PersonId`。然后，我们将这些类型绑定到上下文特质的关联类型上，例如`HasError<Error=Error>`。有了绑定，我们可以使用更简单的trait bounds，例如`Time: SimpleTime`，以代替更冗长的`Context::Time: SimpleTime`。

## Dynamic-Typed Interpretation

```js
function daytime_greeter(in_greeter) {
    return function(context, person_id) {
        const now = context.now()
        if now.is_daytime() {
            return in_greeter.greet(context, person_id)
        } else {
            throw new ShopeClosedError({ time: now })
        }
    }
}
function build_daytime_greeter_class(deps) {
    return Class {
        constructor(in_greeter) {
            this.in_greeter = in_greeter
        }

        greet(context, person_id) {
            const now = deps.HasTime.prototype.now.call(context)
            if deps.HasTime.Time.prototype.is_daytime.call(now) {
                return deps.InGreeter.prototype.greet.call(
                    this.in_greeter, context, person_id)
            } else {
                throw deps.HasError.Error.from(
                    new ShopeClosedError({ time: now }))
            }
        }
    }
}
const DaytimeGreeter = build_daytime_greeter_class({
    InGreeter: ...,
    HasError: {
        Error: {
            from: function(err) { ... }
        },
    },
    PersonContext: {
        PersonId: ...,
        Person: ...,
    },
    HasTime: {
        Time: {
            prototype: {
                is_daytime: function() { ... }
            }
        }
    },
})
function build_daytime_greeter_class(deps) {
    const {
        HasTime,
        HasError,
        InGreeter,
    } = deps

    const { Time } = HasTime
    const { Error } = HasError

    return Class {
        constructor(in_greeter) {
            this.in_greeter = in_greeter
        }

        greet(context, person_id) {
            const now = HasTime.prototype.now.call(context)

            if Time.prototype.is_daytime.call(now) {
                return InGreeter.prototype.greet.call(
                    this.in_greeter, context, person_id)
            } else {
                throw Error.from(
                    new ShopeClosedError({ time: now }))
            }
        }
    }
}
```
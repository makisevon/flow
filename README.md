# DAG Flow

DAG Flow is a simple DAG workflow engine. It aims to provide a flexible scheduling framework for asynchronous tasks. Currently, it is still an experimental project, so feel free to PR.

## Examples

Let's start with a simple example.

Assume that there are 3 tasks `A`, `B` and `C`:

- `A`:
  1. Do something (1 s)
  2. Output
- `B`:
  1. Do something (3 s)
  2. Output
- `C`:
  1. Do something with `A`'s output (1 s)
  2. Do something with `B`'s output (0 s)
  3. Output

For convenience, assume that these tasks use the type `Bytes` for output:

```rust
type Bytes = Vec<u8>;
```

We can define these tasks by implementing the trait `dag_flow::task::Task`:

```rust
use std::collections::HashMap;
use std::time::Duration;

use dag_flow::task::Input;
use dag_flow::task::Task;
use futures_timer::Delay;
```

```rust
struct A;

impl Task<String, Bytes> for A {
    fn id(&self) -> String {
        "A".into()
    }

    async fn run(&self, _: HashMap<String, Input<'_, Bytes>>) -> Option<Bytes> {
        // do something
        Delay::new(Duration::from_secs(1)).await;

        // output
        Some("A's output".into())
    }
}
```

```rust
struct B;

impl Task<String, Bytes> for B {
    fn id(&self) -> String {
        "B".into()
    }

    async fn run(&self, _: HashMap<String, Input<'_, Bytes>>) -> Option<Bytes> {
        // do something
        Delay::new(Duration::from_secs(3)).await;

        // output
        Some("B's output".into())
    }
}
```

```rust
struct C;

impl Task<String, Bytes> for C {
    fn id(&self) -> String {
        "C".into()
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["A".into(), "B".into()]
    }

    async fn run(&self, inputs: HashMap<String, Input<'_, Bytes>>) -> Option<Bytes> {
        // do something with `A`'s output
        let _output_a = inputs["A"].clone().await;
        Delay::new(Duration::from_secs(1)).await;

        // do something with `B`'s output
        let _output_b = inputs["B"].clone().await;

        // output
        Some("C's output".into())
    }
}
```

Note that the outputs of `A` and `B` are passed to `C` as inputs via futures. Why is it designed this way?

In fact, such a design allows DAG Flow to run all tasks simultaneously, rather than running tasks in layers according to dependency order. Meanwhile, we can `await` dependent inputs at any time within a task, which means we don't have to break down some cohesive tasks for better concurrency. In most cases, such breakdowns make tasks hard to maintain.

After defining these tasks, we can run them through the engine:

```rust
use std::time::Instant;

use dag_flow::context::Context;
use dag_flow::engine::Engine;
use futures::executor;
```

```rust
fn main() {
    let builder = Engine::builder();
    builder.add_task(A).add_task(B).add_task(C);

    let engine = builder.build().unwrap();
    let context = Context::new();

    let now = Instant::now();
    executor::block_on(engine.run(context.clone()));
    assert_eq!(now.elapsed().as_secs(), 3);

    assert_eq!(
        executor::block_on(async { context.get(&"C".to_string()).unwrap().await }),
        Some("C's output".into())
    )
}
```

The workflow is as follows:

|       |     `A`      |     `B`      |                   `C`                    |
| :---: | :----------: | :----------: | :--------------------------------------: |
|  0 s  | Do something | Do something |           `await` `A`'s output           |
|       | Do something | Do something |           `await` `A`'s output           |
|  1 s  |    Output    | Do something |           `await` `A`'s output           |
|       |              | Do something |      Do something with `A`'s output      |
|  2 s  |              | Do something |           `await` `B`'s output           |
|       |              | Do something |           `await` `B`'s output           |
|  3 s  |              |    Output    |           `await` `B`'s output           |
|       |              |              | Do something with `B`'s output<br>Output |

So far, everything is fine. The engine can schedule and run tasks in dependency order. But what if there are weak dependencies? Specifically, in the above example, if we are not concerned with `B`'s output, how can we make `C` weakly dependent on `B` while minimizing the impact on concurrency?

There is obviously a simple solution: add some code to `B` to check whether it is a dependency of `C` in the running workflow. The problem with this solution is that the code added to `B` should logically belong to `C`, and coding in this way may make it hard to decouple tasks.

Is there a better solution? Let's go back to the definition of `C`. `B`'s output is passed to `C` via a future, and this future is actually designed to be `B` itself. In other words, we can let `C` rather than the engine drive `B` to start running.

To achieve this, we can redefine `B` and `C` as follows:

```rust
impl Task<String, Bytes> for B {
    /* -- snip -- */
    fn is_auto(&self) -> bool {
        false
    }
}
```

```rust
impl Task<String, Bytes> for C {
    /* -- snip -- */
    async fn run(&self, inputs: HashMap<String, Input<'_, Bytes>>) -> Option<Bytes> {
        futures::join!(
            async {
                // do something with `A`'s output
                let _output_a = inputs["A"].clone().await;
                Delay::new(Duration::from_secs(1)).await;
            },
            async {
                // check whether it depends on `B`
                if rand::random_bool(0.5) {
                    // do something with `B`'s output
                    let _output_b = inputs["B"].clone().await;
                }
            }
        );

        // output
        Some("C's output".into())
    }
}
```

Try it now:

```rust
fn main() {
    // -- snip --
    let now = Instant::now();
    executor::block_on(engine.run(context.clone()));

    let elapsed = now.elapsed().as_secs();
    assert!(elapsed == 2 || elapsed == 3);
    // -- snip --
}
```

The workflow is as follows when `C` depends on `B`:

|       |     `A`      |     `B`      |                                    `C`                                    |
| :---: | :----------: | :----------: | :-----------------------------------------------------------------------: |
|  0 s  | Do something |              | `await` `A`'s output<br>`await` `B`'s output (Drive `B` to start running) |
|       | Do something | Do something |               `await` `A`'s output<br>`await` `B`'s output                |
|  1 s  |    Output    | Do something |               `await` `A`'s output<br>`await` `B`'s output                |
|       |              | Do something |          Do something with `A`'s output<br>`await` `B`'s output           |
|  2 s  |              | Do something |                           `await` `B`'s output                            |
|       |              | Do something |                           `await` `B`'s output                            |
|  3 s  |              |    Output    |                           `await` `B`'s output                            |
|       |              |              |                 Do something with `B`'s output<br>Output                  |

## Issues

### How to `dyn` async traits?

For this issue, these crates provide some procedural macros:

- [dtolnay/async-trait](https://github.com/dtolnay/async-trait)
- [spastorino/dynosaur](https://github.com/spastorino/dynosaur)

DAG Flow is powered by dynosaur, and it also tried async-trait before.

### How to handle tasks with different output types?

A common solution is to use serialization/deserialization, but sometimes it can be a bit overkill. In such cases, it may be a good idea to use enums or trait like `std::any::Any`. See [examples](https://github.com/makisevon/dag-flow/tree/main/examples) for more details.

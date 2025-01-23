# Replica Language

# 1. Basic Language Design

- Swift/Java-like object-oriented syntax
- All objects are actors, no classes needed
- Actors are distributed actors by default
- Use single actor when forcing execution on a single node
- Parallel processing utilizing async
- Distributed data management (distributed var)
- Efficient execution on WebAssembly

# 2. Primitive Types

Replica has the following primitive types:

- `Bool`: Boolean type
- `Int`: Integer type
- `Double`: Floating-point type 64-bit
- `String`: String type

## Advanced Types

- `Array`: Array type
- `Dictionary`: Dictionary type
- `Set`: Set type
- `Tuple`: Tuple type
- `Optional`: Optional type
- `Result`: Result type

# 2. Basic Syntax

Replica has syntax similar to object-oriented languages like Swift and Java. The basic writing style is very similar, making it easy for programmers familiar with these languages to write code.

```swift
actor User {
    var id: String
    var name: String

    init(id: String, name: String) {
        self.id = id
    }

    func greet() -> String {
        return "Hello, \(name)!"
    }
}
```

# 3. Actor Model
What makes Replica significantly different from other object-oriented languages is that it makes the actor model standard. There is no class syntax common in typical programming languages, and all objects are treated as asynchronous actors by default.
This approach adopts a complete actor model, making it very suitable for distributed system design. In particular, it has major benefits in terms of asynchronicity, parallelism, and ease of distributed data processing.

```swift
actor Counter {
    var value: Int = 0

    func increment() async {
        value += 1
    }

    func getValue() async -> Int {
        return value
    }
}
```

Additionally, all actors are `distributed actor` by default. This means that actor initialization and method execution are executed on distributed systems by default.

### 3.1 immediate Initializer
The immediate initializer forces synchronous initialization of actors rather than asynchronous initialization. While all objects in Replica are handled asynchronously, this is suitable for use with lightweight objects that undergo frequent value updates.

```swift
actor Point {
    let x: Int
    let y: Int

    immediate init(x: Int, y: Int) {
        self.x = x
        self.y = y
    }
}
```

#### Synchronous Processing with Mutable Variables

The immediate initializer allows instant instantiation without asynchronicity. However, there are points to note. As shown in the code below, synchronous processing applies to `let` (immutable variables), but for `var` (mutable variables), while initialization is immediate, updates are forced to be `async`.

```swift
actor User {
    let id: String
    var name: String

    immediate init(id: String, name: String) {
        self.id = id
        self.name = name
    }

    func updateName(newName: String) async {
        self.name = newName
    }
}
```

#### Summary of immediate Initializer

Constraints and implementation rules for immediate init

**(A) What immediate init can do**

✅ Can store data immediately in let variables (optimization of immutable data)
✅ var can also be initialized immediately, but changes require async (preventing data races)
✅ Objects initialized with immediate init can interoperate with asynchronous actors

**(B) What immediate init cannot do**

❌ Cannot execute processes containing await (synchronous processing only)
❌ Cannot directly incorporate network communication or external data access
❌ Data declared with immediate init cannot be modified (requires async for changes)

### 3.2 single actor Syntax

In Replica, `actor` objects are executed on a Node selected by the system in the CoreOverlay distributed environment, and their execution state is maintained distributedly. However, for very lightweight processing or processing suited for local execution, distributed execution can create overhead and potentially negatively impact execution speed.

`single actor` is syntax that can handle such situations. `single actor` objects are not placed in a distributed environment and can force actor execution on a single node.

```swift
actor Worker {}  // `distributed actor` by default
single actor LocalWorker {}  // operates only on a single node
```

Additionally, the characteristic of executing on a single Node works well with the aforementioned `immediate init` and can be used in combination. Not only actor initialization but also method execution content can be executed entirely on a single node for actors suited for single-node processing.

```swift
single actor Logger {
    let name: String

    immediate init(name: String) {
        self.name = name
    }

    func log(message: String) {
        print("[\(name)] \(message)")
    }
}
```

`single actor` basically guarantees thread safety and ensures that it is not executed in parallel (operates within a single thread).
This allows for safe data management without thread contention.
The following cache object is an example where `single actor` is appropriate:

```swift
single actor Cache {
    var data: [String: String] = [:]

    func store(key: String, value: String) {
        data[key] = value
    }

    func fetch(key: String) -> String? {
        return data[key]
    }
}
```

#### Converting from `single actor` to `distributed actor`

While `single actor` easily guarantees execution on a single node and allows writing without much consideration for asynchronous processing within the actor, communication between `single actor` and `actor` is not possible when you want to pass data from a `single actor` to an `actor` suited for distributed processing.
Therefore, data can be handed over by copying from a single actor to a distributed actor.

`copyToDistributed()` is a special function that can be used in such scenarios. This method allows conversion of a synchronous single actor to Replica's standard `distributed actor`.

```swift
single actor Config {
    let appName: String
    let version: String

    immediate init(appName: String, version: String) {
        self.appName = appName
        self.version = version
    }

    func copyToDistributed() -> DistributedConfig {
        return DistributedConfig(appName: self.appName, version: self.version)
    }
}

actor DistributedConfig {
    let appName: String
    let version: String

    init(appName: String, version: String) {
        self.appName = appName
        self.version = version
    }
}
```

However, there are points to note. A distributed actor copied from a single actor is treated as a separate independent instance. Therefore, a distributed actor cannot directly reference a single actor, and vice versa.

| Feature | Specification |
|:----------|:----------|
|Copy Direction    |single actor → distributed actor only |
|Mutable Data (var)    | Copied as a snapshot at the time of copyToDistributed() execution  |
| Changes to distributed actor    | Copied distributed actor operates independently |
| Object Reference |single actor and distributed actor are treated as separate instances|

#### `single actor` Summary
✅ This actor is not executed in parallel
✅ Achieves synchronous data management
✅ Enables fast processing as memory consistency doesn't need to be guaranteed

| Feature  | Possible with single actor?  |
|:----------|:----------|
|Communication with other distributed actors | ❌ |
|Execution of remote func| ❌ |
|Use of distributed var| ❌ |
|State management on a single node| ✅ |
|Use of immediate init| ✅ |

## 4. Ownership

Replica adopts a paradigm called **Ownership** that explicitly manages object ownership and prevents data races by controlling moves and copies. This is very similar to the concept adopted by Rust and can efficiently manage execution state without using complex GC. This characteristic works well with autonomous distributed systems, and programs executed on CoreOverlay can expect better performance.

✅ By default, actor instances have ownership
✅ When ownership is moved, the original instance is invalidated
✅ Copy is only available when transitioning from single actor to distributed actor
✅ Use shared when sharing data between multiple distributed actors

**Keywords indicating ownership**

| Keyword  | Description  |
|:----------|:----------|
| move    | Moves ownership to another variable or actor |
| copy    | Copies state from single actor to distributed actor    |
| shared    | Shares data between distributed actors    |

### Moving Ownership with move

By default, actor instances have ownership, so move must be used when assigning to another variable.

```swift
actor DataHolder {
    var data: String

    init(data: String) {
        self.data = data
    }
}

func process(holder: move DataHolder) {
    print("Processing: \(holder.data)")
}

let holder = DataHolder(data: "Hello")
process(holder)  // Ownership moves to `process`
// print(holder.data)  // ← Error here: ownership has been moved
```

### Converting from single actor → distributed actor using copy
✅ With copy, single actor data is copied to distributed actor
✅ Original single actor data is retained (not invalidated as it's copy, not move)

```swift
single actor LocalConfig {
    let settings: [String: String]

    immediate init(settings: [String: String]) {
        self.settings = settings
    }

    func copyToDistributed() -> copy DistributedConfig {
        return DistributedConfig(settings: self.settings)
    }
}

distributed actor DistributedConfig {
    let settings: [String: String]

    init(settings: [String: String]) {
        self.settings = settings
    }
}
```

### Summary

| Operation | single actor | distributed actor |
|--------|--------------|------------------|
| move | ❌ (always owns) | ✅ (can move ownership) |
| copy | ✅ (single actor → distributed actor) | ❌ |
| shared | ❌ | ✅ (share data between distributed actors) |

✅ Introduces Rust-like ownership management (move, copy, shared)
✅ By default, single actors have ownership, and distributed actors can move ownership
✅ Safely execute single actor → distributed actor conversion using copy
✅ Enable data sharing between distributed actors using shared

### Chronological Guarantee of Asynchronous Tasks
While Replica's async model allows parallel execution by default, predictable behavior is needed for processes where chronological order is important.
Therefore, we consider the following approaches:

1. Mechanism to guarantee chronological order

With default async, the order of processing is not guaranteed, potentially executing in unintended order in specific situations.
To solve this, the following techniques can be introduced:

✅ (A) Introduce sequential task queue
✅ (B) Control using priority
✅ (C) Dependency management using task groups

(A) Introduction of sequential tasks

To guarantee tasks executed in chronological order, introduce the sequential modifier and provide a task queue that executes serially.

```swift
actor Logger {
    sequential func logMessage(_ message: String) async {
        print("[LOG]: \(message)")
    }
}

async func main() {
    let logger = Logger()

    await logger.logMessage("Start")
    await logger.logMessage("Processing")
    await logger.logMessage("End")
}
```

✅ With sequential, tasks are processed in chronological order
✅ Even with async, order is maintained within the specified actor

(B) Control using priority

By specifying task priority, high-urgency processes can be executed preferentially.

```swift
actor Scheduler {
    func processTask(id: Int) async priority(high) {
        print("Processing high-priority task: \(id)")
    }

    func processTask(id: Int) async priority(low) {
        print("Processing low-priority task: \(id)")
    }
}
```

✅ priority(high) ensures high-priority tasks are executed first
✅ Task scheduling optimization is possible

(C) Dependency management using task groups

```swift
actor DataPipeline {
    taskgroup func processData() async {
        let rawData = await fetchData()
        let transformedData = await transformData(rawData)
        await storeData(transformedData)
    }

    private func fetchData() async -> String {
        return "raw data"
    }

    private func transformData(_ data: String) async -> String {
        return "transformed data"
    }

    private func storeData(_ data: String) async {
        print("Stored: \(data)")
    }
}
```

✅ Task group makes task dependencies clear
✅ Can execute in parallel while guaranteeing data processing order

| Task Type | Characteristics | Use Cases |
|:--|:--|:--|
| async task | Default asynchronous processing | Tasks that can be executed in parallel |
| sequential task | Guarantees chronological order | Log recording, event processing |
| priority task | Can specify high/low priority | Emergency processing, real-time processing |
| task group | Parallel processing while managing dependencies | Data pipeline, multiple request processing |

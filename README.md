# xscan

## 0. 成品预览

本文将基于`Rust`构建一个常见的网络工具，端口扫描器。

按照惯例，还是和之前实现的文本编辑器一样，我给这个工具起名为`X-SCAN`,它的功能很简单，通过命令行参数的方式对指定`IP`进行扫描，扫描结束之后返回该`IP`地址中处于开放状态的端口号,学完本文，你将自己实现一个如下效果的端口扫描工具(截图以`CSDN`平台的`IP`地址的扫描结果为例)

![image-20240518085144512](https://images.waer.ltd/notes/image-20240518085144512.png)

---

## 1. 相关依赖

```toml
tokio = {version = "1.37.0",features = ["full"]}
bpaf = {version = "0.9.12",features = ["derive","bright-color"]}
ansi_term = "0.12.1"
prettytable-rs = "0.10.0"
```

- `Tokio `:用于异步编程
- `bpaf `：一个简化命令行实现的库
- `ansi_term`：美化终端字符
- `prettytable-rs`：将数据进行表格化打印

> 上面这些依赖都是在后续的代码中需要用到的，后面会在针对每一个依赖库进行简单的入门讲解，便于理解最终要实现的端口扫描工具。过多的还是建议去官方文档学习。

----

## 2. 基本实现原理

通过异步请求对目标`IP`的端口进行`tcp`链接扫描，一旦连接建立成功，将本次连接的端口号返回，以此类推，直到全部扫描结束,打印扫描的结果即可。

![image-20240517212712200](https://images.waer.ltd/notes/image-20240517212712200.png)



---

## 3. 几个依赖库的快速入门

> 这小节会对上面列出来的几个依赖库进行简单的入门，为后续编码扫清障碍。

### 3.1 tokio

- [Tokio官网](https://tokio.rs/)

在`tokio`中，实现异步编程的两大核心

- `async`
- `await`

如果某个函数需要异步执行，可以通过`async`关键字实现，比如下面`connect`函数的定义

```rust
use mini_redis::Result;
use mini_redis::client::Client;
use tokio::net::ToSocketAddrs;

pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
    // ...
}
```

- 这个异步函数的定义看起来像一个普通的同步函数，但实际上是以异步方式运行的。这意味着在代码编写时，异步函数的语法和结构与同步函数类似，使得编写异步代码更加直观和易于理解。
- `Rust `编译器会对异步函数进行转换和优化，以便在运行时能够以异步的方式执行。

- 当异步函数内部遇到 .await 关键字时，它会暂时挂起当前操作，将控制权交还给线程，从而允许线程执行其他任务。
- 当异步操作在后台进行时，线程并不会被阻塞，而是可以继续执行其他任务，从而提高程序的效率和并发性能。

```rust
async fn say_hi() {
    println!("Tokio");
}

#[tokio::main]
async fn main() {
    let op = say_hi();

    println!("hello");

    op.await;
}
```

> - 使用`#[tokio::main]`宏将主函数标记为异步。
>   - 运行时包含异步任务调度器，提供事件 I/O、计时器等。运行时不会自动启动，因此需要 main 函数启动它。
> - 对于异步函数，它的调用方式和普通的`Rust`函数类似，无需其他冗余操作；
> - 当异步函数被调用时，函数体不会立即执行，而是会返回一个表示操作的值，类似于返回一个尚未执行的操作描述标识；
> - 这个概念类似于返回一个零参数的闭包，闭包本身不会立即执行，而是等待进一步的操作；
> - 要执行异步函数代表的操作，这就需要用到了另外一个关键字:`await`,它作用在操作返回值上，用来触发异步操作；

依据上面的描述，示例代码会打印：

> hello Tokio

### 3.2 bpaf

这是一个多功能且易用的命令行参数解析工具。通过借助这个`lib`可以快速高效的编写命令行程序，由于我们的端口扫描器需要手动通过命令行输入IP和端口范围等参数，因此这无疑是一个不错的选择。

- [仓库地址](https://crates.io/crates/bpaf)

- 基本用法

```rust
// 导入 bpaf crate 中的 Bpaf trait
use bpaf::Bpaf;

// 定义一个结构体 SpeedAndDistance，自动实现 Clone、Debug 和 Bpaf trait
#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)] // 添加额外属性 options 和 version
struct SpeedAndDistance {
    speed: f64,     // 速度
    distance: f64,  // 距离
}

fn main() {
    // 解析命令行参数并返回选项
    let opts = speed_and_distance().run();
    
    // 打印解析得到的选项信息
    println!("Options: {:?}", opts);
}
```

> - 通过结构体的方式定义了两个属性，分别是速度和距离；
> - 由于我们需要将这两个字段作为命令行输入的参数，因此这里使用了`#[bpaf(options,version)]`

示例代码中定义了两个参数，在运行时，通过下面的命令即可指定参数值执行程序

```rust
cargo run -- --speed 20.0 --distance 100.0
```

![image-20240518093613844](https://images.waer.ltd/notes/image-20240518093613844.png)

需要注意的是，这个crate有两种不同的用法，过多内容请移步文档。

> 下面两个`crate`对本项目的实质性功能不会产生影响并且使用也相对简单，这里就只做简单的介绍，具体的示例就不再写了，感兴趣的可以自己学习一下。

### 3.3 ansi_term

这个小工具用于美化字段字符的。虽然它的有无并不会影响我们项目的实际功能，但是通过这个工具，我们可以给自己的项目画一个有颜色的炫酷字符图案`logo`,这看起来是一件很酷的事情。

### 3.4 prettytable-rs

用于将输出构建成终端表格的形式进行打印，并且可以指定表格颜色等信息。美化和规范输出。

----

## 4. 步入正题

开始正式编码之前，先分析一下大致的实施步骤。

我们的`X-SCAN`大致可以分为三个小块。

- 命令行参数的定义解析：负责解析命令行参数
- 端口扫描的函数：负责完成扫描的核心任务
- `Rust`主函数：调用扫描函数并将结果组织返回

> 基于此，这里将按照这个步骤依次展开讲解；

### 4.1 参数定义

我们的`X-SCAN`一共需要三个参数,分别是：

1. `IP`地址:`Address`
2. 起始端口号:`start_port`
3. 结束端口号:`end_port`

```rust
// 命令行参数定义
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Argument {
    #[bpaf(long, short, argument("Address"), fallback(IPFALLBACK))]
    /// 想要嗅探的地址，必须是有效的IPv4地址。将回退到127.0.0.1
    pub address: IpAddr,
    #[bpaf(
        long("start"),
        short('s'),
        guard(start_port_guard, "必须大于0"),
        fallback(1u16)
    )]
    pub start_port: u16,

    #[bpaf(
        long("end"),
        short('e'),
        guard(end_port_guard, "必须小于或等于65535"),
        fallback(MAX)
    )]
    pub end_port: u16,
}
```

> - 这里主要用到了`bpaf`,这个上面讲过了，但是这里有一些东西需要提一下；
> - 这里用到了`guard`作为字段的条件约束，指明该参数应该满足的规则，它需要指定一个校验函数；
> - 引入了`long`和`short`两个属性，用来指定参数的长格式和短格式两种风格；
> - `fallback`用来指定参数默认值，在用户没有显式指定参数时，它的值将用作默认值；

上面的代码中大概也注意到了，在指定`IP`地址参数时，我们用到了两个默认值的常量，下面是他们的定义：

```rust
const IPFALLBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const MAX: u16 = 65535;
```

> - 常量（ `MAX` 和 `IPFALLBACK` ）：这些是用作默认值的预定义值。 `MAX` 设置结束端口的最大值，确保它不超过允许的最大端口号 (65535)。 
> - `IPFALLBACK` 提供默认 `IP `地址（`127.0.0.1`，这是本地主机），以防用户未指定 IP 地址。

---

### 4.2 扫描函数

这个`scan` 函数是一个异步函数，旨在检查给定 `IP `地址上的特定端口是否打开。

```rust
/// 异步函数：扫描指定地址和端口
async fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr) {
    match TcpStream::connect(format!("{}:{}", addr, start_port)).await {
        Ok(_) => {
            print!(".");
            io::stdout().flush().unwrap();
            tx.send(start_port).unwrap();
        }
        Err(_) => {}
    }
}
```

> - 函数签名： `async fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr)` ：这定义了一个名为 `scan` 的异步函数，它采用三个参数：
>   - `tx` ： `Sender<u16>` 类型，用于将数据（在本例中为端口号）发送到程序的另一部分。
>   - `start_port` ：要检查的端口号;
>   - `addr` ：要检查端口的 `IP `地址。
> - `TcpStream::connect(format!("{}:{}", addr, start_port)).await` ：此行尝试建立到指定 `addr` 和 `start_port` 的 TCP 连接。使用 `await` 关键字是因为 `TcpStream::connect` 是一个异步操作，您需要等待它完成才能继续,这一点之前也说过了；
> - 使用`match`表达式来处理返回的不同结果，具体如下：
>   - `Ok(_)`：连接成功，捕获一个开放端口；
>     - `print!(".")`这里用来在扫描过程中打印`...`，作为正在扫描的视觉提示；
>     - `io::stdout().flush().unwrap()`:通过刷新标准输出缓冲区确保点立即显示在屏幕上,达到实时加载的视觉效果；
>     - `tx.send(start_port).unwrap()`:通过 `tx` 通道返回开放端口号，以由程序的其他部分处理或记录。
>   - `Err(_)`:连接失败，表示本次连接的端口为关闭状态，不做任何操作；
> - 利用异步编程有效地处理可能长时间运行的网络操作，而不会阻止程序其他部分的执行。允许同时扫描多个端口，从而加快扫描过程。

---

### 4.3 结果处理

`main` 函数设置异步环境、收集参数并生成用于扫描指定范围内的每个端口的任务。通过表格整理返回结果并打印 ；

```rust
#[tokio::main]
async fn main() {
    print_infos();
    let opts = argument().run();

    let (tx, rx) = channel();

    for i in opts.start_port..opts.end_port {
        let tx = tx.clone();
        task::spawn(async move { scan(tx, i, opts.address).await });
    }
    let mut open_ports = vec![];

    drop(tx);

    for p in rx {
        open_ports.push(p);
    }

    println!("");
    open_ports.sort();

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Port").style_spec("Fg=blue"),
        Cell::new("Status").style_spec("Fg=blue"),
    ]));

    for port in open_ports {
        table.add_row(Row::new(vec![
            Cell::new(&port.to_string()),
            Cell::new("is open"),
        ]));
    }

    table.printstd();
}
```

> - `#[tokio::main]` ：该属性宏将常规 `main` 函数转换为异步主函数。它设置 `Tokio `运行时，这是运行异步代码所必需的。
> - `let opts = arguments().run();` ：此行调用 `arguments()` 函数。该函数构造并解析命令行参数，返回 `Arguments` 结构体存储在 `opts` 中。
> - `let (tx, rx) = channel();` ：这里创建了生产者、单消费者 通道。 `tx` 是发送者， `rx` 是接收者。该通道用于异步任务之间的通信。
> - 接着就是端口扫描的一个循环处理：
>   - `第10行` ：为每个端口生成一个新的异步任务。使用当前端口号 `i` 、克隆的发送者 `tx` 和目标 `IP `地址 `opts.address` 调用 `scan` 函数。每个任务将尝试连接到其分配的端口并通过通道将结果发送回。
> - `drop(tx);` ：显式删除原始发件人。这很重要，因为它标识将不再在此通道上发送消息，从而允许接收者在处理所有发送的消息后退出循环。
> - 对于结果的处理，这里创建了一个`vec`数组，此循环从通道接收消息。每条消息代表一个开放端口号并将其存入`vec`之中；
> - 对于`23-27`行,使用`prettytable-rs`提供的方法构建表格的表头，包括端口`Port`和开放状态`Status`;
> - `29-36`行则是将结果添加到表格中并打印在终端。

---

### 4.4 打印版本信息

对于图案信息，大家可以去这个网站生成之后复制过来.[https://patorjk.com/software/taag/](https://patorjk.com/software/taag/)

我们新增一个函数，用来在重新启动时打印`X-SCAN`的字符`LOGO`和版本等信息：

```rust
fn print_infos() {
    println!(
        "{}",
        Red.paint(
            r#"
         __   __            _____    _____              _   _ 
         \ \ / /           / ____|  / ____|     /\     | \ | |
          \ V /   ______  | (___   | |         /  \    |  \| |
           > <   |______|  \___ \  | |        / /\ \   | . ` |
          / . \            ____) | | |____   / ____ \  | |\  |
         /_/ \_\          |_____/   \_____| /_/    \_\ |_| \_|
                                                              
        author:代号0408
        version:0.1.0                                                      
        "#
        )
    );
}
```

> 别忘了在`main`函数中调用；

---

## 5. 使用方式

```rust
cargo run -- --address 8.137.10.104 --start 1 --end 8888
```

> - address参数：指定要扫描的`IP`地址
> - start 参数：指定起始端口
> - end参数：指定结束端口

当然，你也可以对参数使用短格式来执行程序：

```rust
cargo run -- --address 49.232.219.30 -s 1 -e 10000
```

>  注意，使用参数的短格式形式时参数前面的短横线也需要调整为一条短横线(`-`)，长格式参数使用两条(`--`)；

假设我们不指定IP地址，那么它将会默认扫描本地`127.0.0.1`;

```rust
cargo run --   -s 1 -e 10000
```

----

- 项目地址:[https://github.com/08820048/X-SCAN](https://github.com/08820048/X-SCAN)

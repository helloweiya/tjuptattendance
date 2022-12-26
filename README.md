# tjuptattendance

AzureQAQ's Blog: [3MoreDays](https://azureqaq.github.io)

目前已经完成基础功能，欢迎使用

如果使用上有什么问题或对本项目的建议，欢迎 [新建issue](https://github.com/azureqaq/tjuptattendance/issues/new)

如果你也对 `rust` 感兴趣，并且想提交代码，欢迎 **PR**


## 简介
- 使用[rust-lang](https://www.rust-lang.org/) 开发
- 支持所有主流平台 *部分平台需要自行编译*
- 保证准确率
- TOP10签到 *TODO*


## 目标

开发进度请查看: [changelog](./CHANGELOG.md)

- [x] 命令行配置
- [x] cookies 保存
- [x] 配置文件解析(高级功能)
- [x] 实现签到
- [ ] 更详细的配置文件及自定义功能
  - [x] 邮件功能
  - [ ] 定时功能(TOP10签到)
  - [ ] 信息查询


## 安装方式
Windows 推荐使用 Release 的方式

其他平台，推荐自行编译的方式(Release可能会有依赖问题)，如果有人知道如何解决，欢迎 **PR** 或者 **Issue**，将不胜感激

### 通过 Release
- 访问 [发行版](https://github.com/azureqaq/tjuptattendance/releases) 页面，下载适合平台的最新发行版 **注意，可用的版本号 >= 1.0.0**
- 从命令行启动: `./tjuptatt --help`
- 根据提示使用，更详细的介绍见下节

### 自行编译 Source
- 安装 `Rust-lang` 及其工具链(包含`cargo`): [安装方法](https://www.rust-lang.org/tools/install)
- 克隆本仓库并切换到 `master` 分支: `git clone https://github.com/azureqaq/tjuptattendance.git` `cd tjuptattendance` `git switch master`
- 编译: `cargo build --release`
- 运行: `cargo run --release -- --help` 或者复制二进制文件到其他地方，编译后的路径：`./target/release/tjuptatt` 或者 `target/release/tjuptatt.exe`

## 使用方法(手动)
### 临时使用
将不会保存 *cookies* 文件，相对于 *自动* 方式来说，稍微复杂一些，优点就是不会产生任何额外的文件

1. 将得到的可执行文件(`tjuptatt`/`tjuptatt.exe`)：放到方便的位置
2. 打开命令行：切换到英文输入法，同时按下键盘上的 *Win + R* 键，打开 *CMD* 窗口，拖入 *可执行文件* 并在后边输入命令，比如：`C:\Users\user\Desktop\tjuptatt --help` 来查看帮助信息
3. 执行命令：`C:\Users\user\Desktop\tjuptatt -u "name" "password"` 来运行
4. 查看命令行输出

### 使用配置文件
将保存配置及cookies到本地文件，方便使用 (以默认配置文件位置为例)

1. 初始化：`tjuptatt --init`
2. 按照实例编辑配置文件，配置文件路径在第一步时已经显示出来 *以toml格式存储的*
3. 运行：`tjuptatt`，无需任何参数
4. **可选-使用自定义位置的配置文件**：`tjuptatt -f 配置文件位置` *操作方式：可以先拖入tjuptatt再写-f，再拖入配置文件*


## 使用方法(自动)
通过创建定时任务来自动运行，仅以 *Windows* 为例

请自行搜索或查看引用的教程：[计划任务](https://www.xitongcheng.com/jiaocheng/win10_article_47796.html)

**注意**：创建计划任务时，要添加参数的话，比如 `--email` 请在 *新建操作* 时，*程序或脚本* 填写 `tjuptatt` 的路径，*添加参数* 填写 `--email`

## 命令参数
**最简单的方式**: `tjuptatt -u "name" "password"`

### 参数
- `--init`: 初始化，创建默认配置文件及其父文件夹，创建保存cookie的文件夹
- `--uninstall`: 卸载，删除由`--init`所创建的文件和文件夹
- `--user`: 从命令行获取用户信息运行，格式: `--user id1 pwd1 --user id2 pwd2 ...` 此种方式不需要 `--init` 即可正常使用，不会留下任何文件
- `--retry`: 签到重试次数，必须与 `--user` 一起使用 *暂时不推荐使用，因为豆瓣api得有一段时间冷却*
- `--file`: 使用配置文件的参数来进行签到，如果不指定则使用默认值，如果要使用自定义位置: `tjuptatt config -f CONFIG_PATH`，如果直接运行不加任何参数则效果如同: `tjuptatt -f DEFAULT_CONFIG_PATH`
- `--email`: 是否启用邮件通知，必须与配置文件一起使用 `--file`，同时要求开启的 *user* 填写了 `email` 字段

### 子命令 - config - 配置文件快速操作
- `--file`: 指定要操作的配置文件，如果不指定则使用默认值
- `--show`: 显示结果的配置文件简要信息
- `--adduser`: 快速添加用户，格式: `--adduser id1 pwd 1 --adduser id2 pwd2`
- `--rmuser`: 快速删除用户，格式: `--rmuser id1 --rmuser id2`

## 配置文件格式

可以参考配置文件模版: [配置文件模版](https://github.com/azureqaq/tjuptattendance/blob/master/config_template.toml)

```toml
# 实例配置文件, 展示高级设置

# 在此填写用户信息，可以指定多个
[[users]]
# 是否开启，如果关闭则不会对此用户进行签到
# 从命令行快速添加的默认是 true
enable = false
# 邮箱地址
# 用来发送邮件提醒
email = "asd@qq.com"
# 用户的登录名
id = "user_id"
# 用户的密码
pwd = "user_pwd"

# 全局设置
[global]
retry = 1

# 邮件设置
# 用来发送邮件提醒
[global.emailconf]
# 登陆名
user = "登录名"
# 发件人，可以不指定
sender = "发件人"
# 登陆密码
pwd = "pwd"

# smtp设置，如果不指定则是 smtp.qq.com
host = "smtp.qq.com"
# 端口，如果不指定则是465
port = 465
```

- `[[users]]`: 用来设置用户信息
- `[global]`: 全局配置信息

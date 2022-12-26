# tjuptattendance

AzureQAQ's Blog: [3MoreDays](https://azureqaq.github.io)

目前已经完成基础功能，欢迎使用

如果有什么问题，欢迎 [新建issue](https://github.com/azureqaq/tjuptattendance/issues/new)

如果你也对 `rust` 感兴趣，并且想提交代码，欢迎 **PR**


## 简介
- 使用[rust-lang](https://www.rust-lang.org/) 开发
- 支持所有主流平台 *部分平台需要自行编译*
- 保证准确率
- 定时功能 *TODO*
- TOP10签到 *TODO*


## 目标

开发进度请查看: [changelog](./CHANGELOG.md)

- [x] 命令行配置
- [x] cookies 保存
- [x] 配置文件解析(高级功能)
- [x] 实现签到
- [ ] 更详细的配置文件及自定义功能
  - [ ] 定时功能(TOP10签到)
  - [ ] 邮件功能


## 安装方法 - Release
- 访问 [发行版](https://github.com/azureqaq/tjuptattendance/releases) 页面，下载适合平台的最新发行版 **注意，可用的版本号 >= 1.0.0**
- 从命令行启动: `./tjuptatt --help`
- 根据提示使用，更详细的介绍见下节

## 安装方法 - Source
- 安装 `Rust-lang` 及其工具链(包含`cargo`): [安装方法](https://www.rust-lang.org/tools/install)
- 克隆本仓库并切换到 `master` 分支: `git clone https://github.com/azureqaq/tjuptattendance.git` `git switch master`
- 编译: `cargo build --release`
- 运行: `cargo run --release -- --help` 或者复制二进制文件到其他地方 `cp target/release/tjuptatt NEWP_ATH` `cd NEW_PATH` `tjuptatt --help`

## 命令参数
**最简单的方式**: `tjuptatt -u "name" "password"`

### 参数
- `--init`: 初始化，创建默认配置文件及其父文件夹，创建保存cookie的文件夹
- `--uninstall`: 卸载，删除由`--init`所创建的文件和文件夹
- `--user`: 从命令行获取用户信息运行，格式: `--user id1 pwd1 --user id2 pwd2 ...` 此种方式不需要 `--init` 即可正常使用，不会留下任何文件
- `--retry`: 签到重试次数，必须与 `--user` 一起使用 *暂时不推荐使用，因为豆瓣api得有一段时间冷却*
- `--file`: 使用配置文件的参数来进行签到，如果不指定则使用默认值，如果要使用自定义位置: `tjuptatt config -f CONFIG_PATH`，此参数只能单独使用，如果直接运行不加任何参数则效果如同: `tjuptatt -f DEFAULT_CONFIG_PATH`

### 字命令 - config - 配置文件快速操作
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

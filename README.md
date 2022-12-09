# tjuptattendance

AzureQAQ's Blog: [3MoreDays](https://azureqaq.github.io)

## 尚未完成!
- 进度请查看 [Changelog](/CHANGELOG.md)

## 简介
- 使用[rust-lang](https://www.rust-lang.org/) 开发
- 支持所有主流平台 *部分平台需要自行编译*


## 目标
- [x] 命令行配置
- [x] 命令行定时签到
- [x] cookie 保存
- [x] 配置文件解析(高级功能)
- [ ] 签到实现的默认方式
- [ ] 可自定义的签到实现 
- [ ] 更详细的配置文件及自定义功能


## 安装方法 - Release
- 访问 [发行版](https://github.com/azureqaq/tjuptattendance/releases) 页面，下载适合平台的最新发行版
- 从命令行启动: `./tjuptatt --help`
- 根据提示使用，更详细的介绍见下节

## 安装方法 - Source
- 安装 `Rust-lang` 及其工具链(包含`cargo`): [安装方法](https://www.rust-lang.org/tools/install)
- 克隆本仓库并切换到 `master` 分支: `git clone https://github.com/azureqaq/tjuptattendance.git` `git switch master`
- 编译: `cargo build --release`
- 运行: `cargo run --release -- --help` 或者复制二进制文件到其他地方 `cp target/release/tjuptatt NEWP_ATH` `cd NEW_PATH` `tjuptatt --help`

## 命令参数
### 参数
- `--init`: 初始化，创建默认配置文件及其父文件夹，创建保存cookie的文件夹
- `--uninstall`: 卸载，删除由`--init`所创建的文件和文件夹
- `--user`: 从命令行获取用户信息运行，格式: `--user id1 pwd1 --user id2 pwd2 ...` 此种方式不需要 `--init` 即可正常使用，不会留下任何文件
- `--retry`: 签到重试次数，必须与 `--user` 一起使用
- `--time`: 签到时间点，必须与 `--user` 一起使用，如果设定的时间点已经过去了，则会等待到明天的时间点，格式: `--time HOUR MIN SEC`
- `--delay`: 网络延迟(ms)，可以按照经验适当尝试增加，注意：请不要太大！默认是0ms，实际签到时间应该为 `--time` 设定的时间之前毫秒数的时间，此参数必须与 `--time` 一起使用
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
# 网络延迟(ms) 范围 0~1000ms
delay = 50
# 签到时间点
# 如果不指定则是马上签到，如果指定则会等到对应时间
# 可以设置多个时间点，但是只有接下来最近的那一个会执行
# 格式：时分秒纳秒 二十四小时制
points_in_time = [ [0,0,0,0], [6,0,0,0] ]

# 全局设置
[global]
# 网络延迟，如果用户设置中未指定，则会使用此值
# 范围 0~1000ms
network_delay = 50

# 邮件设置
# 用来发送邮件提醒
[global.emailconf]
# 是否开启邮件提醒
# 注意：就算开启了但是用户未填写邮箱，还是不会发送的
# 如果想要使用此功能，请提前进行测试
enable = false
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

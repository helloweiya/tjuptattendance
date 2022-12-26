//! 命令行参数解析

use crate::command::DIRS;
use anyhow::{anyhow, Result};
use clap::{
    crate_authors, crate_description, crate_name, crate_version, value_parser, Arg, ArgAction,
    ArgMatches, Command,
};

/// 使用`clap`来解析命令行参数
pub fn cli_parser() -> Result<ArgMatches> {
    let Some(config_path) = DIRS.config_path().to_str() else {
        return Err(anyhow!("无法获取配置文件位置"));
    };
    Ok(Command::new(crate_name!())
        .about(crate_description!())
        .long_about("北洋园PT(TJUPT <https://tjupt.org/>) 签到工具")
        .author(crate_authors!())
        .version(crate_version!())
        .args_conflicts_with_subcommands(true)
        .before_help("更多信息请查看 https://github.com/azureqaq/tjuptattendance")
        .before_long_help(
            "\
AzureQAQ's Blog: https://www.3moredays.com/
本工具仅作为学习交流使用，切勿用作违规用途!!!
详细规则请访问: https://tjupt.org/rules.php
更多信息请访问: https://github.com/azureqaq/tjuptattendance

使用本工具所造成的一切后果自负",
        )
        .help_template(
            "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}tjuptatt [<subcommand>] [<option> <arg> ...]

{all-args}{after-help}",
        )
        .arg_required_else_help(false)
        .arg(
            Arg::new("init")
                .long("init")
                .help("初始化")
                .long_help(
                    "\
初始化，创建必须的文件夹
如果不进行初始化，无法使用部分功能
包括：创建默认位置的配置文件、状态文件",
                )
                .action(ArgAction::SetTrue)
                .num_args(0)
                .exclusive(true),
        )
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .help("删除相关文件夹及包含的文件")
                .long_help(
                    "\
删除所有相关文件夹及包含的文件
注意，此操作无法撤销",
                )
                .action(ArgAction::SetTrue)
                .num_args(0)
                .exclusive(true),
        )
        .arg(
            Arg::new("user")
                .long("user")
                .short('u')
                .help("指定用户, <userid> <userpwd>")
                .long_help(
                    "\
指定本次使用哪个用户
格式为: <userid> <userpwd>
可以通过 '-u <id1> <pwd1> -u <id2> <pwd2> -u ...' 指定多个用户
使用此方式来覆盖配置文件",
                )
                .action(ArgAction::Append)
                .num_args(2),
        )
        .arg(
            Arg::new("retry")
                .long("retry")
                .short('r')
                .help("重试次数")
                .long_help("在进行签到过程中, 遇到错误时，重复尝试的次数 >=1")
                .action(ArgAction::Set)
                .num_args(1)
                .value_parser(value_parser!(u8))
                .default_value("1")
                .requires("user")
                .conflicts_with("file"),
        )
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .help("使用配置文件")
                .long_help(
                    "\
使用指定位置的配置文件
如果不指定则使用默认值",
                )
                .action(ArgAction::Set)
                .default_value(config_path)
                .num_args(1)
                .conflicts_with("user"),
        )
        .arg(
            Arg::new("email")
                .long("email")
                .short('e')
                .help("是否开启邮件提醒")
                .long_help(
                    "\
是否开启邮件提醒，默认不使用邮件功能，
此选项启用后，会使用配置文件中的邮件配置，
必须与 `-f` 一起使用",
                )
                .action(ArgAction::SetTrue)
                .num_args(0)
                .conflicts_with("user"),
        )
        .subcommand(
            Command::new("config")
                .about("配置文件相关操作")
                .long_about(
                    "\
配置文件相关操作
如果不指定配置文件，将使用默认位置
更多配置文件信息: https://github.com/azureqaq/tjuptattendance",
                )
                .help_template("\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}tjuptatt config [<option> <arg> ...]

{all-args}{after-help}")
                .arg_required_else_help(true)
                .args_conflicts_with_subcommands(true)
                .arg(
                    Arg::new("show")
                        .long("show")
                        .short('s')
                        .help("查看配置文件信息")
                        .long_help("查看配置文件信息，可以通过`--file`来自定义")
                        .action(ArgAction::SetTrue)
                        .num_args(0),
                )
                .arg(
                    Arg::new("file")
                        .short('f')
                        .long("file")
                        .help("自定义配置文件路径")
                        .long_help(
                            "\
指定自定义的配置文件位置
否则使用默认值",
                        )
                        .default_value(config_path)
                        .action(ArgAction::Set)
                        .num_args(1),
                )
                .arg(
                    Arg::new("adduser")
                        .long("adduser")
                        .short('a')
                        .help("增加用户")
                        .long_help(
                            "\
增加一个默认启用的用户
格式: <userid> <userpwd>
如果要配置邮箱等，可以去对应配置文件修改
注意：这可能覆盖已经存在的用户配置",
                        )
                        .action(ArgAction::Append)
                        .num_args(2),
                )
                .arg(
                    Arg::new("rmuser")
                        .long("rmuser")
                        .help("删除对应配置文件中的用户")
                        .long_help(
                            "\
删除对应配置文件中的用户
格式: <userid>
可以通过 -r <id> -r <id> ... 来同时指定多个",
                        )
                        .action(ArgAction::Append)
                        .num_args(1)
                        .conflicts_with("adduser"),
                ),
        )
        .get_matches())
}

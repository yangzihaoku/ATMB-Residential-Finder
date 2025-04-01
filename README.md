# ATMB 非CMRA地址筛选工具

这是一个用于爬取Anytime Mailbox地址并筛选出非CMRA地址的工具。本项目基于 [starccy/atmb-us-non-cmra](https://github.com/starccy/atmb-us-non-cmra) 进行改进，添加了Web界面支持，使工具更易于使用。

## 功能特点

- 自动爬取Anytime Mailbox上的所有地址
- 使用Smarty Streets API验证地址并检查CMRA状态
- 支持选择特定州进行爬取
- 提供CLI和Web界面两种使用方式
- 结果保存为CSV格式，方便后续处理

## 安装与使用

### 必备条件

- Rust环境 (https://www.rust-lang.org/tools/install)
- Smarty Streets API凭证 (https://www.smarty.com/docs/cloud/authentication)

### 运行方式

#### 1. 命令行模式

```bash
# 设置API凭证
export CREDENTIALS=your-auth-id=your-auth-token

# 运行程序
cargo run
```

#### 2. Web界面模式

```bash
# 启动Web服务器
cargo run -- web
```

然后在浏览器中打开 http://127.0.0.1:8080 访问Web界面。

## 使用Web界面

1. 在"设置API凭证"部分输入您的Smarty Streets API凭证
2. 在"选择州"部分选择您想要爬取的州（可以全选）
3. 点击"开始爬取"按钮启动爬取过程
4. 等待处理完成，结果将保存在`result/mailboxes.csv`文件中

## API凭证格式

API凭证的格式为：`ID=密钥`，如果有多个凭证，用逗号分隔：

```
id1=secret1,id2=secret2
```

## 注意事项

- 爬取过程可能需要一些时间，取决于所选州的数量和网络状况
- Smarty Streets API有使用限制，请确保您的凭证有足够的配额
- 结果文件默认保存在`result/mailboxes.csv`

## 致谢

本项目基于 [starccy/atmb-us-non-cmra](https://github.com/starccy/atmb-us-non-cmra) 进行改进。感谢原作者 [starccy](https://github.com/starccy) 的开源贡献！

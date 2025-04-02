# ATMB 非CMRA地址筛选工具

这是一个用于爬取Anytime Mailbox地址并筛选出非CMRA地址的工具。本项目基于 [starccy/atmb-us-non-cmra](https://github.com/starccy/atmb-us-non-cmra) 进行改进，添加了Web界面支持，使工具更易于使用。

## 功能特点

- 自动爬取Anytime Mailbox上的所有地址
- 使用Smarty Streets API验证地址并检查CMRA状态
- 支持选择特定州进行爬取
- 提供CLI和Web界面两种使用方式
- 结果保存为CSV格式，方便后续处理
- 支持在Web界面中直接查看和搜索结果
- 支持多个API凭证，自动轮换使用
- 实时显示处理进度和结果统计

## 安装指南

### 1. 安装Rust环境

1. 访问 [Rust官网](https://www.rust-lang.org/tools/install)
2. 下载并运行安装程序：
   - Windows: 下载并运行 `rustup-init.exe`
   - macOS/Linux: 在终端中运行以下命令：
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```
3. 安装完成后，打开新的终端窗口，运行以下命令验证安装：
   ```bash
   rustc --version
   cargo --version
   ```

### 2. 获取Smarty Streets API凭证

1. 访问 [Smarty官网](https://www.smarty.com/)
2. 注册一个免费账号
3. 在控制台中获取您的API凭证（Auth ID和Auth Token）
4. 注意：免费账号每月有1000次查询限制，而ATMB目前有1700多个美国地址，建议注册2-3个账号

### 3. 下载项目

1. 打开终端（命令提示符）
2. 进入您想要存放项目的目录
3. 运行以下命令克隆项目：
   ```bash
   git clone https://github.com/yangzihaoku/ATMB-Residential-Finder.git
   cd ATMB-Residential-Finder
   ```

## 使用说明

### 方式一：使用Web界面（推荐）

1. 在终端中进入项目目录
2. 运行以下命令启动Web服务器：
   ```bash
   cargo run -- web
   ```
3. 打开浏览器，访问 http://127.0.0.1:8080
4. 在Web界面中：
   - 输入您的Smarty Streets API凭证（可以添加多个）
   - 选择要爬取的州（可以全选）
   - 点击"开始爬取"按钮
   - 等待处理完成，结果会直接显示在网页中
   - 可以使用搜索框和排序功能筛选结果
   - 结果同时会保存在`result/mailboxes.csv`文件中

Web界面功能说明：
- 支持添加多个API凭证，系统会自动轮换使用
- 可以搜索和排序结果
- 显示每个地址的详细信息，包括：
  - 名称
  - 地址
  - 城市
  - 州
  - 邮编
  - 价格
  - CMRA状态
  - RDI类型
- 可以直接点击"查看"链接访问ATMB网站
- 实时显示处理进度和统计信息

### 方式二：使用命令行

1. 设置API凭证环境变量：
   ```bash
   # Windows (命令提示符)
   set CREDENTIALS=your-auth-id=your-auth-token

   # Windows (PowerShell)
   $env:CREDENTIALS="your-auth-id=your-auth-token"

   # macOS/Linux
   export CREDENTIALS=your-auth-id=your-auth-token
   ```

2. 运行程序：
   ```bash
   cargo run
   ```

3. 等待程序运行完成，结果将保存在`result/mailboxes.csv`文件中

## API凭证格式

API凭证的格式为：`ID=密钥`，如果有多个凭证，用逗号分隔：

```
id1=secret1,id2=secret2
```

## 注意事项

- 爬取过程可能需要一些时间，取决于所选州的数量和网络状况
- Smarty Streets API有使用限制，请确保您的凭证有足够的配额
- 结果文件默认保存在`result/mailboxes.csv`
- 如果遇到问题，请检查：
  1. Rust环境是否正确安装
  2. API凭证是否正确设置
  3. 网络连接是否正常
  4. 是否有足够的磁盘空间

## 常见问题

### 1. 安装Rust时遇到问题

- 确保您的系统已安装最新的更新
- 如果下载速度慢，可以尝试使用国内镜像源：
  ```bash
  export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
  export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
  ```

### 2. 程序无法启动

- 检查是否在正确的目录下运行命令
- 确保没有其他程序占用8080端口
- 检查是否有足够的系统权限

### 3. 爬取失败

- 检查API凭证是否正确
- 确认网络连接正常
- 查看是否有足够的API调用配额

## 致谢

本项目基于 [starccy/atmb-us-non-cmra](https://github.com/starccy/atmb-us-non-cmra) 进行改进。感谢原作者 [starccy](https://github.com/starccy) 的开源贡献！


# 原力小镇
- 构建一个AI驱动的在ICP网络上运行的社交应用
- 验证ICP网络运行社交应用的可行性
-[ Web App访问地址](https://tsoel-7yaaa-aaaai-alcva-cai.icp0.io/)

## Introduction
PartyBoard原力小镇项目web portal的代码，服务于web前端app，调用canister的接口完成应用的社交功能。

Highlight some features:
- 话题社交。
- 和不同的AI角色聊天。
- 和不同的真人角色聊天。
 
![架构图](https://github.com/MetaPowerMatrix/pabOnICP/blob/master/MetaPowerICP%E6%9E%B6%E6%9E%84%E5%9B%BE.jpg)

## Installation

### Prerequisites
需要先安装Rust开发工具

```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install
下载和编译

```bash
$ git clone [<GitHub repo>](https://github.com/MetaPowerMatrix/metapower_service_icp)
$ cd metapower_service_icp
$ export RUSTFLAGS="-A unused"
$ cargo build --release
```

## Usage
部署完成之后, 继续部署前端App，修改前端代码的后端服务地址即可使用这个后端portal服务


## Documentation
PartyBoard原力小镇项目web portal的代码，服务于web前端app，调用canister的接口完成应用的社交功能。

## Roadmap
Describe the project roadmap, this could be the grant milestones, but it could also be the team's broader project roadmap.

- [x] 移植Matrix和Agent服务，部署两个canister
- [x] 移植Pato智能体服务，部署一个canister 
- [x] 增加向量数据库使用，部署一个canister. 
- [x] 部署前端App到canister，可以通过canister域名访问应用的功能.
- [ ] 添加更多的系统AI角色.
- [ ] 构建数据量足够大的社交应用向量数据库供检索.

## License
This project is licensed under the MIT license, see LICENSE.md for details. See CONTRIBUTE.md for details about how to contribute to this project. 


## References
- [Internet Computer](https://internetcomputer.org)


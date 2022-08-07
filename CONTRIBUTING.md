# 协作开发说明

本文档用于对本仓库协作开发的流程及其规范进行粗略的说明。

## 分支管理

本节对仓库分支的管理方式做出约定。若无特别说明，名称以 `master`、`dev`、`release`、`feature`、`hotfix` 为首的分支应该按照 Git Flow 的方式进行管理。对其他分支则没有强制的要求或者限制。

简而言之，在实现某一个新功能时，应该从 `dev` 分支 Fork 出一个 `feature/*` 分支（如 `feature/api_draft`）并在这个分支上进行开发。在完成开发后，应该向 `dev` 分支发起一个 [PR](https://github.com/kaaass/bc-hostcall/compare)，若通过 Code Review 再以 No fast forward 方式并入 `dev` 分支。发起 PR 前，建议对代码进行格式化，并且通过相关 `cargo test`。

特例：为了适应快节奏的开发，现阶段若有 Commit 的内容的确过于简单，无需 Fork `feature` 分支即可完成实现，则可以直接向 `dev` 分支进行 Commit。

## Commit 规范

本节对仓库主要分支（受 Git Flow 管理的）中的 Commit 的消息格式、内容做出约定，对于其他分支则无强制要求，只需要在并入主要分支时的提交上符合规范即可。若无特别说明，这些分支中的提交应该参考 [约定式提交 v1.0.0-beta.4](https://www.conventionalcommits.org/zh-hans/v1.0.0-beta.4/)，并符合下文中的规范。

简而言之，一个 Commit 应该具有如下格式的消息：

```
<类型>(作用域): <简述>

<正文>
```

其中，“作用域”与“正文”并不是必须的。如果添加了作用域，则应该是一个单一的名称，通常是当前 crate 的名称，比如：`lowlevel`。此外，在本项目中，简述与正文应该使用中文描述。

类型应该选择如下类型其一，或者其他语义明确的类型：

- feat: 新特性
- fix: 修复 BUG
- docs: 文档更改
- refactor: 代码结构重构
- test: 测试用例相关修改
- chore: 其余类型更新

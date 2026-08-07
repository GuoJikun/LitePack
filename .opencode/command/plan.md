---
description: 加载 LitePack 开发计划并从当前里程碑继续执行
agent: build
---

读取 `.opencode/PLAN.md` 中的 LitePack 开发计划，然后：

1. 根据 PLAN.md 第 10 节的里程碑勾选状态，报告当前进度
2. 从第一个未完成的里程碑开始继续实施
3. 每完成一个里程碑，更新 PLAN.md 中对应勾选项为 `[x]`
4. 严格遵循 PLAN.md 中锁定的依赖版本、目录结构与 API 设计

$ARGUMENTS

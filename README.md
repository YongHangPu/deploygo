# deploygo

> 面向静态站点的桌面端发布工具，把零停机部署、版本管理与回滚收进一套可直接使用的产品体验。

deploygo 让前端发布回到一条清晰流程：

- 在业务项目中完成构建
- 在页面根组件接入对应框架的版本更新通知组件
- 在 deploygo 中完成服务器配置、发布、回滚与历史追踪

## 接入方式

### 1. 挂载更新通知组件

从 `templates/` 目录选择对应框架的组件放入业务项目，并挂载到应用根组件。组件会轮询线上 `version.json`，在发现新版本后提示用户刷新页面获取最新资源。

支持 Vue / React / Angular / Svelte / 纯 JavaScript 五种版本：

| 框架 | 文件 |
|------|------|
| Vue 3 (TypeScript) | `templates/VersionUpdateNotification.vue` |
| Vue 3 (JavaScript) | `templates/VersionUpdateNotification.vue.js` |
| Vue 2 (TypeScript) | `templates/VersionUpdateNotification.vue2.ts` |
| Vue 2 (JavaScript) | `templates/VersionUpdateNotification.vue2` |
| React | `templates/VersionUpdateNotification.tsx` |
| Angular | `templates/VersionUpdateNotification.angular.ts` |
| Svelte | `templates/VersionUpdateNotification.svelte` |
| 纯 JS（零依赖） | `templates/VersionUpdateNotification.js` |

### 2. 保持业务项目原有构建方式

业务项目只需要按平常方式构建（`npm run build` / `pnpm build` / `yarn build` 等），无需为 deploygo 调整构建流程。

## deploygo 自动完成什么

在桌面端中填写服务器信息、项目名称、本地构建产物目录、线上发布目录和保留版本数后，deploygo 会自动完成：

1. 识别真实构建目录
2. 自动生成 `version.json`
3. 自动生成 release 名称
4. 在本地准备发布目录或压缩包
5. 根据 `live_root_path` 自动推导同级 `releases` 目录
6. 远端自动创建 `live_root`、`releases_root`、备份目录
7. 上传发布内容到远端 release 目录
8. 执行内置零停机发布流程
9. 清理旧版本
10. 写入部署历史，并支持一键回滚
11. 拦截重复发布，避免同一份产物被重复部署

## 使用流程

```
1. 在业务项目中先执行构建
2. 打开 deploygo，添加服务器
3. 添加项目，填写本地构建产物目录和线上发布目录
4. 点击一键部署
5. 如需回滚，到部署历史中选择目标版本
```

### 服务器信息

需要填写：

- 服务器名称
- 主机地址、端口
- 用户名
- 认证方式（密码 或 SSH 密钥）
- 可选：跳板机 / 堡垒机配置

### 项目信息

需要填写：

- 项目名称
- 本地构建产物目录（支持目录选择器）
- 关联服务器
- 线上发布目录
- 保留版本数

deploygo 会自动根据线上发布目录推导同级 `releases` 目录，并在远端完成创建与维护。

## 适用范围

deploygo 适合：

- Vue / React / Angular / Svelte / 纯 HTML 等所有 SPA / 静态站点
- 构建产物包含 hash 资源文件的前端项目
- 线上通过 Nginx 或类似静态文件服务托管的项目
- 没有专门 DevOps 流水线，但又希望发版稳定可回滚的团队

以下场景暂不适合：

- SSR 服务端渲染项目
- 不生成 hash 资源名的静态站点
- 发布时必须立即清空旧资源的场景

## 发布语义

deploygo 的内置发布流程遵循固定顺序：

```text
[1/3] 先同步静态资源（排除 index.html / version.json）
[2/3] 再替换 index.html
[3/3] 最后替换 version.json
```

这样可以保证：

- 旧页面仍然能继续访问已有资源，不会在发布瞬间白屏
- 新用户会逐步进入新版本入口
- 已打开页面的用户会在检测到新版本后收到刷新提示，而不是被强制打断

## 缓存建议

如果页面没有按预期提示新版本，优先检查线上缓存策略：

- `index.html` 不要强缓存
- `version.json` 不要强缓存
- 带 hash 的静态资源可以长缓存

只要项目具备稳定的 hash 构建产物，deploygo 就能提供完整的版本发布体验。

## 演示站

仓库中还提供了 Web 演示页，用于展示：

- 产品介绍与接入说明
- 桌面端使用流程
- 多框架更新通知组件下载

本地运行：

```bash
git clone https://github.com/YongHangPu/deploygo.git
cd deploygo
pnpm install
pnpm dev
```

## 下载桌面端

从 [GitHub Releases](https://github.com/YongHangPu/deploygo/releases) 下载最新版本。

也可以自行构建：

```bash
pnpm install
pnpm tauri:build
```

## License

[MIT](./LICENSE)

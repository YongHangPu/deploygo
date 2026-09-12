#!/usr/bin/env node
// 发布说明拼装脚本
//
// 用途：发布 Release 时，从 CHANGELOG.md 提取当前版本小节，
//   填充 .github/release-template.md 模板，输出最终 Release 说明到 stdout。
//
// 用法：
//   node scripts/build-release-notes.mjs <版本号>            # 输出到 stdout
//   node scripts/build-release-notes.mjs <版本号> > notes.md # 供 gh release create --notes-file 使用
//
// 维护规则：每次发版前在 CHANGELOG.md 顶部（`---` 之后）添加 `## [x.y.z] - 日期` 小节。
//   找不到对应小节时脚本不失败，输出兜底文案并打印警告。

import { existsSync, readFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const version = process.argv[2]

if (!version || !/^\d+\.\d+\.\d+/.test(version)) {
  console.error('用法: node scripts/build-release-notes.mjs <版本号，如 1.0.0>')
  process.exit(1)
}

// 从 CHANGELOG.md 提取 `## [版本号]` 小节正文（到下一个 `## [` 或文件结束）
const changelogPath = join(root, 'CHANGELOG.md')
let section = ''
if (existsSync(changelogPath)) {
  const parts = readFileSync(changelogPath, 'utf8').split(/^## \[(.+?)\][^\n]*\n/m)
  for (let i = 1; i < parts.length; i += 2) {
    if (parts[i] === version) {
      section = parts[i + 1].trim()
      break
    }
  }
}
if (!section) {
  console.warn(`警告: CHANGELOG.md 中未找到 [${version}] 小节，使用兜底文案（建议发版前补充更新记录）`)
  section = '- 详见 [提交历史](https://github.com/YongHangPu/deploygo/commits/main)。'
}

const template = readFileSync(join(root, '.github', 'release-template.md'), 'utf8')
process.stdout.write(template.replaceAll('{{VERSION}}', version).replace('{{CHANGELOG}}', section) + '\n')

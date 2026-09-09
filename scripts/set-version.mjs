/**
 * 一键更新版本号脚本
 * 同步更新 tauri.conf.json、Cargo.toml、package.json 的 version
 *
 * 使用: node scripts/set-version.mjs "1.0.0"
 */

import { readFile, writeFile } from 'node:fs/promises'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')

const newVersion = process.argv[2]
if (!newVersion || !/^\d+\.\d+\.\d+/.test(newVersion)) {
  console.error('用法: node scripts/set-version.mjs <版本号>')
  console.error('示例: node scripts/set-version.mjs 1.2.0')
  process.exit(1)
}

// ── 1. tauri.conf.json ──
const confPath = resolve(root, 'src-tauri', 'tauri.conf.json')
let conf = await readFile(confPath, 'utf8')
conf = conf.replace(/"version":\s*"[^"]+"/, `"version": "${newVersion}"`)
await writeFile(confPath, conf)
console.log(`[1/3] tauri.conf.json  → ${newVersion}`)

// ── 2. Cargo.toml ──
const cargoPath = resolve(root, 'src-tauri', 'Cargo.toml')
let cargo = await readFile(cargoPath, 'utf8')
cargo = cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${newVersion}"`)
await writeFile(cargoPath, cargo)
console.log(`[2/3] Cargo.toml       → ${newVersion}`)

// ── 3. package.json ──
const pkgPath = resolve(root, 'package.json')
let pkg = JSON.parse(await readFile(pkgPath, 'utf8'))
pkg.version = newVersion
await writeFile(pkgPath, JSON.stringify(pkg, null, 2) + '\n')
console.log(`[3/3] package.json     → ${newVersion}`)

console.log('\n三个文件版本号已同步更新！')

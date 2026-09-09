/**
 * 将原生 Tauri 应用复制到 release/，用于本地分发。
 * 支持宿主机构建，也支持通过 --target <triple> 指定 Rust 目标。
 */

import {
  chmodSync,
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  rmSync,
} from 'node:fs'
import { dirname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = dirname(fileURLToPath(import.meta.url))
const projectRoot = resolve(scriptDir, '..')
const releaseDir = resolve(projectRoot, 'release')

function println(message) {
  console.log(`[build-release] ${message}`)
}

function readTargetArgument() {
  const index = process.argv.indexOf('--target')
  if (index === -1) return process.env.TAURI_TARGET || ''
  const value = process.argv[index + 1]
  if (!value || value.startsWith('--')) {
    throw new Error('--target requires a Rust target triple')
  }
  return value
}

function targetPlatform(target) {
  if (target.includes('windows')) return 'windows'
  if (target.includes('apple-darwin')) return 'macos'
  if (target.includes('linux')) return 'linux'
  if (process.platform === 'win32') return 'windows'
  if (process.platform === 'darwin') return 'macos'
  return 'linux'
}

function assertInsideProject(path) {
  const pathFromRoot = relative(projectRoot, resolve(path))
  if (pathFromRoot.startsWith('..') || pathFromRoot === '') {
    throw new Error(`Refusing to operate outside a project subdirectory: ${path}`)
  }
}

function findAppBundle(bundleDir) {
  if (!existsSync(bundleDir)) return null
  const appName = readdirSync(bundleDir).find((name) => name.endsWith('.app'))
  return appName ? join(bundleDir, appName) : null
}

const target = readTargetArgument()
const platform = targetPlatform(target)
const targetReleaseDir = join(
  projectRoot,
  'src-tauri',
  'target',
  ...(target ? [target] : []),
  'release',
)

assertInsideProject(releaseDir)
assertInsideProject(targetReleaseDir)
mkdirSync(releaseDir, { recursive: true })
for (const name of readdirSync(releaseDir)) {
  rmSync(join(releaseDir, name), { recursive: true, force: true })
}

if (platform === 'windows') {
  const source = join(targetReleaseDir, 'app.exe')
  if (!existsSync(source)) throw new Error(`Windows executable not found: ${source}`)
  const destination = join(releaseDir, 'deploygo.exe')
  copyFileSync(source, destination)
  println(`Windows executable: ${destination}`)
} else if (platform === 'macos') {
  const source = findAppBundle(join(targetReleaseDir, 'bundle', 'macos'))
  if (!source) throw new Error(`macOS .app bundle not found under: ${targetReleaseDir}`)
  const destination = join(releaseDir, 'deploygo.app')
  cpSync(source, destination, { recursive: true })
  println(`macOS app bundle: ${destination}`)
} else {
  const candidates = ['app', 'deploygo'].map((name) => join(targetReleaseDir, name))
  const source = candidates.find(existsSync)
  if (!source) throw new Error(`Linux executable not found under: ${targetReleaseDir}`)
  const destination = join(releaseDir, 'deploygo')
  copyFileSync(source, destination)
  chmodSync(destination, 0o755)
  println(`Linux executable: ${destination}`)
}

println(`Release artifact ready for ${target || process.platform}`)

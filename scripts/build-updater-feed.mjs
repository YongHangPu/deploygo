/**
 * 收集 Tauri 更新器产物并生成静态 latest.json 清单。
 * 清单和重命名后的更新包可以一起发布到任意 HTTPS 主机。
 */

import {
  copyFileSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from 'node:fs'
import { basename, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const PLATFORM_ARTIFACTS = {
  'windows-x86_64': {
    directory: '/release-windows/',
    // Tauri v2 的 updater 产物即 NSIS 安装包本身（伴随 -setup.exe.sig 签名文件），
    // 不再是 v1 时代的 .nsis.zip
    suffix: '-setup.exe',
  },
  'darwin-x86_64': {
    directory: '/release-macos-x86_64/',
    suffix: '.app.tar.gz',
  },
  'darwin-aarch64': {
    directory: '/release-macos-aarch64/',
    suffix: '.app.tar.gz',
  },
  'linux-x86_64': {
    directory: '/release-linux/',
    // Tauri v2 的 updater 产物即 AppImage 本身（伴随 .AppImage.sig 签名文件），
    // 不再是 v1 时代的 .AppImage.tar.gz
    suffix: '.AppImage',
  },
}

function parseArguments(argv) {
  const options = {}
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index]
    const value = argv[index + 1]
    if (!key?.startsWith('--') || value === undefined) {
      throw new Error(`Invalid argument near ${key || '<end>'}`)
    }
    options[key.slice(2)] = value
  }
  return options
}

function requireOption(options, name) {
  const value = options[name]?.trim()
  if (!value) throw new Error(`--${name} is required`)
  return value
}

function listFiles(root) {
  const files = []
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const path = join(root, entry.name)
    if (entry.isDirectory()) files.push(...listFiles(path))
    else if (entry.isFile()) files.push(path)
  }
  return files
}

function httpsUrl(value, label) {
  const url = new URL(value)
  if (url.protocol !== 'https:') throw new Error(`${label} must use HTTPS`)
  return url
}

function selectPayload(files, platform, definition) {
  const matches = files.filter((path) => {
    const normalized = path.replaceAll('\\', '/')
    return normalized.includes(definition.directory) && normalized.endsWith(definition.suffix)
  })
  if (matches.length !== 1) {
    throw new Error(`Expected one ${platform} updater payload, found ${matches.length}`)
  }
  const payload = matches[0]
  const signature = `${payload}.sig`
  if (!files.includes(signature)) {
    throw new Error(`Missing updater signature for ${payload}`)
  }
  return { payload, signature }
}

export function buildUpdaterFeed(options) {
  const artifactsRoot = resolve(requireOption(options, 'artifacts'))
  const outputRoot = resolve(requireOption(options, 'output'))
  const version = requireOption(options, 'version').replace(/^v/, '')
  const endpoint = httpsUrl(requireOption(options, 'endpoint'), '--endpoint')
  const explicitDownloadBase = options['download-base']?.trim()
  const downloadBase = explicitDownloadBase
    ? httpsUrl(explicitDownloadBase, '--download-base')
    : new URL('.', endpoint)
  if (!downloadBase.pathname.endsWith('/')) downloadBase.pathname += '/'

  const files = listFiles(artifactsRoot)
  const platforms = {}
  mkdirSync(outputRoot, { recursive: true })

  for (const [platform, definition] of Object.entries(PLATFORM_ARTIFACTS)) {
    const { payload, signature } = selectPayload(files, platform, definition)
    const outputName = `${platform}-${basename(payload)}`
    const outputPayload = join(outputRoot, outputName)
    const outputSignature = `${outputPayload}.sig`
    copyFileSync(payload, outputPayload)
    copyFileSync(signature, outputSignature)

    const signatureText = readFileSync(signature, 'utf8').trim()
    if (!signatureText) throw new Error(`Updater signature is empty: ${signature}`)
    // .sig 文件本身是 base64 包装格式（解码后为 minisign 四行格式），
    // tauri-plugin-updater 运行时会对 signature 做 base64 解码后再按 minisign 解析，
    // 因此 signature 字段直接使用 .sig 文件内容，无需再次编码。
    platforms[platform] = {
      signature: signatureText,
      url: new URL(encodeURIComponent(outputName), downloadBase).toString(),
    }
  }

  const manifest = {
    version,
    notes: options.notes?.trim() || `Deploygo ${version}`,
    pub_date: options['pub-date']?.trim() || new Date().toISOString(),
    platforms,
  }
  const manifestPath = join(outputRoot, 'latest.json')
  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8')
  return manifestPath
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : ''
if (invokedPath === fileURLToPath(import.meta.url)) {
  const manifestPath = buildUpdaterFeed(parseArguments(process.argv.slice(2)))
  console.log(`[updater-feed] created: ${manifestPath}`)
}

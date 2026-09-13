import { writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), '..')
const outputPath = join(projectRoot, 'src-tauri', 'tauri.updater.generated.json')
const endpoint = process.env.DEPLOYGO_UPDATER_ENDPOINT?.trim()
const publicKey = process.env.DEPLOYGO_UPDATER_PUBLIC_KEY?.trim()

let config = {}
if (endpoint || publicKey) {
  if (!endpoint || !publicKey) {
    throw new Error('DEPLOYGO_UPDATER_ENDPOINT and DEPLOYGO_UPDATER_PUBLIC_KEY must be set together')
  }
  if (!process.env.TAURI_SIGNING_PRIVATE_KEY?.trim()) {
    throw new Error('TAURI_SIGNING_PRIVATE_KEY is required when signed updater artifacts are enabled')
  }
  const url = new URL(endpoint)
  if (url.protocol !== 'https:') {
    throw new Error('DEPLOYGO_UPDATER_ENDPOINT must use HTTPS')
  }
  config = {
    bundle: { createUpdaterArtifacts: true },
    plugins: {
      updater: {
        endpoints: [endpoint],
        pubkey: publicKey,
      },
    },
  }
  console.log(`[updater-config] enabled: ${url.origin}`)
} else {
  console.log('[updater-config] disabled: updater endpoint/public key not configured')
}

writeFileSync(outputPath, `${JSON.stringify(config, null, 2)}\n`, 'utf8')

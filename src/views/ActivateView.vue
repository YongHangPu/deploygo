<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  NButton, NInput, NTag, NAlert, NSpin, NSelect, NInputNumber
} from 'naive-ui'
import { useNotify } from '../composables/useNotify'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { save } from '@tauri-apps/plugin-dialog'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import type { LicenseInfo } from '../types'

const notify = useNotify()

// 许可证状态
const license = ref<LicenseInfo | null>(null)
const loading = ref(true)
const activating = ref(false)
const licenseKeyInput = ref('')

// 开源纯壳构建：不含部署引擎，展示引导下载官方完整版
const isShell = computed(() => license.value?.engine === 'shell')

// 开发模式
const isDev = ref(false)

// 生成器状态（仅开发模式）
const genSecret = ref('')
const genPublic = ref('')
const genExpiryValue = ref(1)
const genExpiryUnit = ref<'days' | 'months' | 'quarters' | 'years'>('months')
const genCount = ref(10)
const genKeys = ref<string[]>([])
const generatedDurationLabel = ref('')
const generating = ref(false)
const genKeypairLoading = ref(false)


const tierOptions = [
  { label: '专业版（¥99/年）', value: 'pro' },
]

async function loadLicense() {
  loading.value = true
  try {
    license.value = await invoke<LicenseInfo>('get_license_status')
    isDev.value = await invoke<boolean>('is_dev_mode')
  } catch (e) {
    console.error(e)
  } finally { loading.value = false }
}

async function handleActivate() {
  const key = licenseKeyInput.value.trim()
  if (!key) { notify.warning('请输入授权密钥'); return }
  activating.value = true
  try {
    const result = await invoke<LicenseInfo>('activate_license', { key })
    if (result.is_valid) {
      license.value = result
      licenseKeyInput.value = ''
    notify.success('激活成功！当前版本：专业版')
    } else {
      notify.error(result.reason || '激活失败')
    }
  } catch (e) {
    notify.error(`激活失败: ${e}`)
  } finally { activating.value = false }
}

// ---- 许可证生成器 ----
async function handleGenerateKeypair() {
  genKeypairLoading.value = true
  try {
    const [secret, pub] = await invoke<[string, string]>('generate_license_keypair')
    genSecret.value = secret
    genPublic.value = pub
    notify.warning('新密钥对已生成。正式版构建请将公钥配置为 DEPLOYGO_LICENSE_PUBLIC_KEY，并妥善保存私钥。')
  } catch (e) {
    notify.error(`生成失败: ${e}`)
  } finally { genKeypairLoading.value = false }
}

async function handleGenerateKeys() {
  if (!genSecret.value) { notify.warning('请先生成或填入私钥'); return }
  if (!Number.isInteger(genExpiryValue.value) || genExpiryValue.value < 1) {
    notify.warning('激活后有效期必须大于 0'); return
  }
  if (!Number.isInteger(genCount.value) || genCount.value < 1) {
    notify.warning('生成数量必须大于 0'); return
  }
  generating.value = true
  try {
    const keys: string[] = []
    for (let i = 0; i < genCount.value; i++) {
      const key = await invoke<string>('sign_license_key', {
        tier: 'pro',
        durationValue: genExpiryValue.value,
        durationUnit: genExpiryUnit.value,
        secretHex: genSecret.value,
      })
      keys.push(key)
    }
    genKeys.value = keys
    const unitLabel = { days: '天', months: '个月', quarters: '个季度', years: '年' }[genExpiryUnit.value]
    generatedDurationLabel.value = `${genExpiryValue.value} ${unitLabel}`
    notify.success(`已生成 ${keys.length} 个 Key`)
  } catch (e) {
    notify.error(`生成失败: ${e}`)
  } finally { generating.value = false }
}

async function handleExport() {
  if (!genKeys.value.length) return
  try {
    const path = await save({
      defaultPath: 'license-keys.txt',
      filters: [{ name: '文本文件', extensions: ['txt'] }],
    })
    if (path) {
      await writeTextFile(path, genKeys.value.join('\n'))
      notify.success(`已导出到 ${path}`)
    }
  } catch (e) {
    notify.error(`导出失败: ${e}`)
  }
}

onMounted(() => loadLicense())
</script>

<template>
  <div class="activate-page">
    <div class="activate-container" :class="{ 'has-dev': isDev }">
      <NSpin :show="loading">
        <!-- 页面头部 -->
        <header class="page-header">
          <h2>授权管理</h2>
          <p>管理你的 deploygo 许可证</p>
        </header>

        <div class="content-grid">
        <!-- 左侧主栏 -->
          <div class="main-col">
            <!-- 开源纯壳构建说明 -->
            <NAlert
              v-if="isShell"
              type="info"
              :bordered="false"
              class="section"
            >
              <template #header>开源展示版（不含部署引擎）</template>
              当前构建来自开源仓库，仅包含界面与方案展示，不包含部署引擎与授权系统。
              如需实际部署能力，请从
              <a class="link" @click="open('https://github.com/YongHangPu/deploygo/releases')">GitHub Releases</a>
              下载官方完整版（提供 90 天免费试用）。
            </NAlert>

            <!-- 试用期/过期提示 -->
            <NAlert
              v-else-if="license && !license.is_valid"
              :type="license.trial_remaining_days > 0 ? 'warning' : 'error'"
              :bordered="false"
              class="section"
            >
              <template #header>
                <template v-if="license.tier !== 'free'">授权已失效</template>
                <template v-else>
                  {{ license.trial_remaining_days > 0 ? `90 天试用 · 剩余 ${license.trial_remaining_days} 天` : '试用期已结束' }}
                </template>
              </template>
              {{ license.reason || '部署功能已暂停，请购买授权后继续使用。' }}
            </NAlert>

            <!-- 当前有效许可证详情 -->
            <section v-if="license?.is_valid" class="info-card section">
              <div class="info-card-header">
                <span>当前授权</span>
                <NTag type="success" size="small" round>专业版</NTag>
              </div>
              <div class="info-grid">
                <div class="info-item">
                  <span class="info-label">到期日</span>
                  <span class="info-value">{{ license.expires_at }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">状态</span>
                  <span class="info-value info-valid">● 有效</span>
                </div>
              </div>
            </section>

            <!-- 激活表单 -->
            <section v-if="!isShell" class="form-card section">
              <h3 class="form-card-title">
                {{ license?.is_valid ? '升级 / 续费授权' : '激活授权' }}
              </h3>
              <div class="activate-row">
                <NInput
                  v-model:value="licenseKeyInput"
                  type="text"
                  placeholder="输入授权密钥"
                  size="large"
                  clearable
                  :input-props="{ spellcheck: false }"
                  @keyup.enter="handleActivate"
                />
                <NButton type="primary" size="large" class="activate-btn" :loading="activating" @click="handleActivate">
                  激活
                </NButton>
              </div>
              <div class="form-card-footer">
                <span>
                  还没有授权？
                  <a class="link" @click="open('https://mbd.pub/o/deploygo')">前往购买 →</a>
                </span>
              </div>
            </section>
          </div>

        <!-- 右侧开发栏：许可证生成器 -->
          <div v-if="isDev" class="dev-col">
            <section class="form-card section">
              <h3 class="form-card-title title-with-tag">
                🔑 密钥生成器
                <NTag type="warning" size="tiny" round>开发模式</NTag>
              </h3>

              <div class="gen-grid">
              <!-- 私钥 -->
                <div class="gen-field gen-field-wide">
                  <span class="gen-label">私钥</span>
                  <div class="gen-keypair-row">
                    <NInput v-model:value="genSecret" type="text" placeholder="64 位 hex 私钥" clearable />
                    <NButton type="primary" size="small" :loading="genKeypairLoading" @click="handleGenerateKeypair">
                      生成新密钥对
                    </NButton>
                  </div>
                </div>

              <!-- 公钥（生成后显示） -->
                <div v-if="genPublic" class="gen-field gen-field-wide">
                  <span class="gen-label">公钥（Release 构建用）</span>
                  <div class="gen-keypair-row">
                    <NInput :value="genPublic" type="text" readonly placeholder="生成后自动填入" />
                    <NButton size="small" @click="() => { writeText(genPublic); notify.success('公钥已复制') }">复制</NButton>
                  </div>
                </div>

              <!-- 套餐等级 -->
                <div class="gen-field">
                  <span class="gen-label">版本</span>
                  <NSelect :value="'pro'" :options="tierOptions" disabled />
                </div>

              <!-- 有效期 -->
                <div class="gen-field">
                  <span class="gen-label">激活后有效期</span>
                  <div class="gen-expiry-row">
                    <NInputNumber v-model:value="genExpiryValue" :min="1" :max="3650" />
                    <NSelect
                      :value="genExpiryUnit"
                      :options="[
                        { label: '天', value: 'days' },
                        { label: '月', value: 'months' },
                        { label: '季度', value: 'quarters' },
                        { label: '年', value: 'years' },
                      ]"
                      class="gen-unit-select"
                      @update:value="(v: any) => genExpiryUnit = v"
                    />
                  </div>
                </div>

              <!-- 数量 -->
                <div class="gen-field">
                  <span class="gen-label">数量</span>
                  <NInputNumber v-model:value="genCount" :min="1" :max="500" />
                </div>
              </div>

              <div class="gen-actions">
                <NButton type="primary" :loading="generating" @click="handleGenerateKeys">
                  生成 {{ genCount }} 个 Key
                </NButton>
                <NButton v-if="genKeys.length" @click="handleExport">导出文件</NButton>
              </div>

              <div v-if="genKeys.length" class="gen-result">
                <div class="gen-result-header">
                  <span>已生成 {{ genKeys.length }} 个 Key · 激活后有效 {{ generatedDurationLabel }}</span>
                  <NButton text size="tiny" @click="() => { writeText(genKeys.join('\n')); notify.success('已复制到剪贴板') }">
                    全部复制
                  </NButton>
                </div>
                <div class="gen-list">
                  <div v-for="(key, i) in genKeys" :key="i" class="gen-key-row">
                    <code>{{ key }}</code>
                    <NButton text size="tiny" @click="() => { writeText(key); notify.success('已复制') }">复制</NButton>
                  </div>
                </div>
              </div>
            </section>
          </div>
        </div>
      </NSpin>
    </div>
  </div>
</template>

<style scoped>
/* ── 外层布局 ── */
.activate-page {
  height: 100%;
  display: flex;
  justify-content: center;
  padding: clamp(16px, 3vw, 40px) clamp(12px, 2.5vw, 32px) 60px;
  overflow-y: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.activate-page::-webkit-scrollbar { display: none; }

.activate-container {
  width: 100%;
  max-width: 1300px;
  min-width: 0;
}

/* ── 页面头部（始终占满宽度） ── */
.page-header {
  text-align: center;
  margin-bottom: clamp(20px, 3vw, 32px);
}
.page-header h2 {
  margin: 0;
  font-size: clamp(18px, 3vw, 24px);
  font-weight: 700;
}
.page-header p {
  margin: 4px 0 0;
  font-size: clamp(12px, 1.4vw, 14px);
  color: var(--text-muted);
}

/* ── 双栏网格 ── */
.content-grid {
  display: grid;
  gap: clamp(16px, 2.5vw, 28px);
}

/* 宽屏：显示开发工具时并排展示 */
@media (min-width: 1000px) {
  .activate-container.has-dev .content-grid {
    grid-template-columns: 1fr 1fr;
    align-items: start;
  }
}

/* ── 公共样式 ── */
.section { margin-bottom: clamp(14px, 2vw, 20px); }

/* ── 许可证信息卡片 ── */
.info-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 14px;
  padding: clamp(14px, 2.5vw, 24px);
  box-shadow: 0 2px 12px rgba(0,0,0,0.05);
}
.info-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: clamp(13px, 1.6vw, 16px);
  font-weight: 600;
  margin-bottom: clamp(10px, 1.8vw, 18px);
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: clamp(10px, 1.8vw, 16px) clamp(14px, 2vw, 24px);
}
.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.info-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.info-value {
  font-size: clamp(13px, 1.6vw, 15px);
  font-weight: 500;
  word-break: break-all;
}
.info-valid { color: var(--accent-green); }

/* ── 表单卡片 ── */
.form-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 14px;
  padding: clamp(14px, 2.5vw, 24px);
  box-shadow: 0 2px 12px rgba(0,0,0,0.05);
}
.form-card-title {
  margin: 0 0 clamp(10px, 1.5vw, 16px);
  font-size: clamp(14px, 1.7vw, 16px);
  font-weight: 600;
}
.title-with-tag {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

/* 激活行：输入框 + 按钮 */
.activate-row {
  display: flex;
  gap: 10px;
}
.activate-row > :first-child { flex: 1; }
.activate-btn { min-width: 80px; white-space: nowrap; }

.form-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: clamp(10px, 1.5vw, 16px);
  font-size: 13px;
  color: var(--text-muted);
}

/* ── 许可证生成器 ── */
.gen-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: clamp(10px, 1.5vw, 14px) clamp(12px, 1.8vw, 16px);
}
.gen-field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
}
.gen-field-wide { grid-column: 1 / -1; }
.gen-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.gen-keypair-row {
  display: flex;
  gap: 8px;
}
.gen-keypair-row > :first-child { flex: 1; }

.gen-expiry-row {
  display: flex;
  gap: 8px;
}
.gen-expiry-row > :first-child { flex: 1; }
.gen-unit-select { width: 85px; flex-shrink: 0; }

.gen-actions {
  display: flex;
  gap: 10px;
  margin-top: clamp(12px, 2vw, 18px);
}
.gen-actions > :first-child { flex: 1; }

.gen-result { margin-top: clamp(10px, 1.5vw, 16px); }
.gen-result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 4px;
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 8px;
}
.gen-list {
  background: var(--bg-code);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  padding: 8px;
  max-height: 240px;
  overflow-y: auto;
}
.gen-key-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  border-radius: 4px;
}
.gen-key-row:hover { background: var(--sidebar-hover); }
.gen-key-row code {
  flex: 1;
  font-size: clamp(10px, 1.2vw, 11px);
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  color: var(--text-secondary);
  word-break: break-all;
}

/* ── 其他样式 ── */
.link {
  color: var(--accent-blue);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 3px;
  font-weight: 500;
}
.link:hover { opacity: 0.75; }

/* ── 响应式断点 ── */

/* 窄屏：所有内容垂直堆叠 */
@media (max-width: 540px) {
  .activate-row {
    flex-direction: column;
  }
  .activate-btn {
    width: 100%;
  }

  .gen-keypair-row {
    flex-direction: column;
  }

  .gen-expiry-row {
    flex-direction: column;
  }
  .gen-unit-select {
    width: 100%;
  }

  .info-grid {
    grid-template-columns: 1fr;
  }

  .gen-grid {
    grid-template-columns: 1fr;
  }
  .gen-field-wide { grid-column: 1; }
}

/* 中等屏幕：允许适度并排，同时保持合理布局 */
@media (min-width: 541px) and (max-width: 999px) {
  .info-grid {
    grid-template-columns: 1fr 1fr;
  }
  .gen-grid {
    grid-template-columns: 1fr 1fr;
  }
}

/* 非开发宽屏：单栏展示，并限制可读的最大宽度 */
@media (min-width: 1000px) {
  .activate-container:not(.has-dev) .content-grid {
    max-width: 720px;
    margin: 0 auto;
  }
}
</style>

<style scoped>
.activate-page {
  background: transparent;
}

.activate-container {
  max-width: var(--content-max-width);
  gap: 18px;
}

.page-header {
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-subtle);
}

.page-header h2 {
  font-weight: 750;
}

.page-header h2::before {
  content: none;
  display: block;
  margin-bottom: 4px;
  color: var(--accent-blue);
  font: 700 10px/1.2 'JetBrains Mono', monospace;
  letter-spacing: 0.11em;
}

.info-card,
.form-card {
  border-radius: 8px;
  background: var(--bg-panel);
  border-color: rgba(var(--accent-blue-rgb), 0.16);
  box-shadow: 0 1px 2px rgba(27, 34, 47, 0.035);
  transition:
    transform 260ms cubic-bezier(0.22, 1, 0.36, 1),
    border-color 260ms cubic-bezier(0.22, 1, 0.36, 1),
    box-shadow 260ms cubic-bezier(0.22, 1, 0.36, 1);
}

.info-card {
  border-top: 2px solid var(--accent-blue);
}

.info-card:hover,
.form-card:hover {
  transform: translateY(-1px);
  border-color: rgba(var(--accent-blue-rgb), 0.3);
  box-shadow: 0 10px 26px rgba(27, 34, 47, 0.06);
}

.info-card-header,
.form-card-title {
  font-weight: 750;
}

.info-item {
  border-bottom-color: var(--border-subtle);
}

.info-label,
.gen-label {
  font: 650 10px/1.4 'JetBrains Mono', monospace;
  letter-spacing: 0.04em;
}

.activate-btn,
.gen-actions :deep(.n-button),
.form-card-footer :deep(.n-button) {
  border-radius: 5px;
  font-weight: 700;
}

.gen-key-row,
.gen-result,
.gen-list {
  border-radius: 5px;
}
</style>

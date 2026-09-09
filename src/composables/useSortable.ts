import { onMounted, onUnmounted, watch, nextTick, type Ref, type WatchSource } from 'vue'
import Sortable from 'sortablejs'

export interface UseSortableOptions<T> {
  /** 拖拽手柄的 CSS 选择器；省略时整个项目都可拖拽。 */
  handle?: string
  /** SortableJS 在拖拽结束时读取的响应式列表，用于计算新顺序。 */
  items: Ref<T[]>
  /**
   * 可选的自定义监听源，用于控制何时重建 Sortable。
   * 默认直接监听 `items`，因为筛选或搜索会改变 DOM。
   * 如果列表只关心添加/删除，可传入 `() => items.value.length`，
   * 避免每次内部排序都销毁并重建 Sortable。
   */
  watchSource?: WatchSource
  /** 拖拽完成后回调，并传入重新排序后的项目 id。 */
  onReorder: (ids: string[]) => void
  /** 从项目中提取唯一 id。 */
  getId: (item: T) => string
}

export function useSortable<T>(containerRef: Ref<HTMLElement | null>, opts: UseSortableOptions<T>) {
  let sortable: any = null

  const setup = () => {
    const el = containerRef.value
    if (!el) return
    if (sortable) sortable.destroy()

    sortable = Sortable.create(el, {
      animation: 150,
      handle: opts.handle,
      forceFallback: true,
      ghostClass: 'sortable-ghost',
      chosenClass: 'sortable-chosen',
      dragClass: 'sortable-drag',
      onEnd: (evt: { oldIndex?: number; newIndex?: number }) => {
        if (evt.oldIndex === evt.newIndex) return
        const items = [...opts.items.value]
        const [moved] = items.splice(evt.oldIndex!, 1)
        items.splice(evt.newIndex!, 0, moved)
        opts.onReorder(items.map(opts.getId))
      },
    })
  }

  // 监听需要重建 Sortable 的变化（数据加载、搜索筛选，或按 watchSource 决定的添加/删除）。
  watch(opts.watchSource ?? opts.items, async () => {
    await nextTick()
    setup()
  })

  // DOM 就绪后完成首次初始化。
  onMounted(async () => {
    await nextTick()
    setup()
  })

  onUnmounted(() => {
    if (sortable) {
      sortable.destroy()
      sortable = null
    }
  })
}

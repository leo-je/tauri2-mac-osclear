import type { JunkScanTargetId, JunkScanTargetOption } from '../types'

export const JUNK_SCAN_TARGETS: JunkScanTargetOption[] = [
  {
    id: 'user_cache',
    label: '用户缓存',
    description: '扫描 ~/Library/Caches 中的应用缓存文件。'
  },
  {
    id: 'logs',
    label: '日志文件',
    description: '扫描 ~/Library/Logs 中的应用和系统日志。'
  },
  {
    id: 'tmp',
    label: '临时文件',
    description: '扫描 /tmp 中的临时内容。'
  },
  {
    id: 'system_cache',
    label: '系统缓存',
    description: '扫描 /Library/Caches 中的系统级缓存。'
  },
  {
    id: 'xcode_derived_data',
    label: 'Xcode 派生数据',
    description: '扫描 Xcode DerivedData，适合开发环境释放空间。'
  },
  {
    id: 'application_support',
    label: '应用支持文件',
    description: '扫描 ~/Library/Application Support 下较大的支持文件。'
  },
  {
    id: 'downloads',
    label: '下载文件',
    description: '扫描下载目录中的大文件条目。'
  },
  {
    id: 'trash',
    label: '废纸篓',
    description: '扫描当前用户废纸篓中的内容。'
  }
]

export const DEFAULT_JUNK_SCAN_TARGET_IDS: JunkScanTargetId[] = JUNK_SCAN_TARGETS.map(target => target.id)
